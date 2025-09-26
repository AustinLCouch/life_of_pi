use axum::{
    extract::State,
    response::{Html, Json},
    routing::{get, Router},
    serve,
};
use serde::{Deserialize, Serialize};
use std::{
    env, fs,
    net::SocketAddr,
    process::Command,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use sysinfo::{Disks, Networks, System};
use tokio::{net::TcpListener, time::interval};
use tower_http::{cors::CorsLayer, services::ServeDir};
use tracing::info;

// System metrics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SystemSnapshot {
    timestamp: u64,
    cpu_usage: f32,
    cpu_temp: f32,
    memory_total: u64,
    memory_used: u64,
    memory_percent: f32,
    disk_total: u64,
    disk_used: u64,
    disk_percent: f32,
    network_rx: u64,
    network_tx: u64,
    // System information
    hostname: String,
    os_name: String,
    kernel_version: String,
    uptime: u64, // seconds
    load_avg_1m: f64,
    load_avg_5m: f64,
    load_avg_15m: f64,
    current_user: String,
    local_ips: Vec<String>,
    pi_model: Option<String>,
}

#[derive(Clone)]
struct AppState {
    latest_snapshot: Arc<tokio::sync::RwLock<SystemSnapshot>>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    info!("🥧 Life of Pi - Starting Raspberry Pi Monitor");

    // Create initial state
    let app_state = AppState {
        latest_snapshot: Arc::new(tokio::sync::RwLock::new(get_system_snapshot())),
    };

    // Start background metrics collection
    let state_clone = app_state.clone();
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(2));
        loop {
            interval.tick().await;
            let snapshot = get_system_snapshot();
            *state_clone.latest_snapshot.write().await = snapshot;
        }
    });

    // Create router
    let app = Router::new()
        .route("/", get(dashboard))
        .route("/api/metrics", get(get_metrics))
        .nest_service("/static", ServeDir::new("static"))
        .layer(CorsLayer::permissive())
        .with_state(app_state);

    // Start server
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()?;
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    info!("Starting server on http://{}", addr);
    info!("Dashboard: http://localhost:{}", port);
    info!("API: http://localhost:{}/api/metrics", port);

    let listener = TcpListener::bind(addr).await?;
    serve(listener, app).await?;

    Ok(())
}

// Get current system metrics
fn get_system_snapshot() -> SystemSnapshot {
    let mut sys = System::new_all();
    sys.refresh_all();

    // CPU usage (global usage)
    let cpu_usage = sys.global_cpu_usage();

    // Memory
    let memory_total = sys.total_memory();
    let memory_used = sys.used_memory();
    let memory_percent = if memory_total > 0 {
        (memory_used as f32 / memory_total as f32) * 100.0
    } else {
        0.0
    };

    // Disk (use root filesystem)
    let mut disk_total = 0;
    let mut disk_used = 0;
    let disks = Disks::new_with_refreshed_list();
    for disk in &disks {
        if disk.mount_point().to_str().unwrap_or("") == "/" {
            disk_total = disk.total_space();
            disk_used = disk_total - disk.available_space();
            break;
        }
    }
    let disk_percent = if disk_total > 0 {
        (disk_used as f32 / disk_total as f32) * 100.0
    } else {
        0.0
    };

    // Network (sum all interfaces)
    let mut network_rx = 0;
    let mut network_tx = 0;
    let networks = Networks::new_with_refreshed_list();
    for (_name, network) in &networks {
        network_rx += network.total_received();
        network_tx += network.total_transmitted();
    }

    // CPU temperature (Raspberry Pi specific)
    let cpu_temp = read_cpu_temperature().unwrap_or(0.0);

    // System information
    let hostname = System::host_name().unwrap_or_else(|| "unknown".to_string());
    let os_name = System::long_os_version().unwrap_or_else(|| "Unknown OS".to_string());
    let kernel_version = System::kernel_version().unwrap_or_else(|| "Unknown".to_string());
    let uptime = System::uptime();
    let load_avg = System::load_average();
    let current_user = env::var("USER").unwrap_or_else(|_| "unknown".to_string());
    let local_ips = get_local_ip_addresses();
    let pi_model = get_pi_model();

    SystemSnapshot {
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64,
        cpu_usage,
        cpu_temp,
        memory_total,
        memory_used,
        memory_percent,
        disk_total,
        disk_used,
        disk_percent,
        network_rx,
        network_tx,
        hostname,
        os_name,
        kernel_version,
        uptime,
        load_avg_1m: load_avg.one,
        load_avg_5m: load_avg.five,
        load_avg_15m: load_avg.fifteen,
        current_user,
        local_ips,
        pi_model,
    }
}

// Get local IP addresses
fn get_local_ip_addresses() -> Vec<String> {
    use std::net::IpAddr;

    let mut ips = Vec::new();

    if let Ok(output) = Command::new("hostname").arg("-I").output() {
        if output.status.success() {
            let ip_string = String::from_utf8_lossy(&output.stdout);
            for ip in ip_string.split_whitespace() {
                if let Ok(parsed_ip) = ip.parse::<IpAddr>() {
                    match parsed_ip {
                        IpAddr::V4(ipv4) => {
                            if !ipv4.is_loopback() && !ipv4.is_link_local() {
                                ips.push(ip.to_string());
                            }
                        }
                        IpAddr::V6(ipv6) => {
                            if !ipv6.is_loopback() && !ipv6.is_unspecified() {
                                ips.push(ip.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    // Fallback: try to get interface info from /proc/net/route and ifconfig
    if ips.is_empty() {
        if let Ok(output) = Command::new("ip")
            .args(["route", "get", "8.8.8.8"])
            .output()
        {
            if output.status.success() {
                let route_info = String::from_utf8_lossy(&output.stdout);
                // Parse "src <IP>" from the output
                for line in route_info.lines() {
                    if let Some(src_idx) = line.find("src ") {
                        let ip_part = &line[src_idx + 4..];
                        if let Some(ip_end) = ip_part.find(' ') {
                            let ip = &ip_part[..ip_end];
                            if let Ok(_) = ip.parse::<IpAddr>() {
                                ips.push(ip.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    if ips.is_empty() {
        ips.push("127.0.0.1".to_string());
    }

    ips
}

// Get Raspberry Pi model information
fn get_pi_model() -> Option<String> {
    // Try reading from /proc/device-tree/model first
    if let Ok(model) = fs::read_to_string("/proc/device-tree/model") {
        let cleaned = model.trim_end_matches('\0').trim();
        if !cleaned.is_empty() {
            return Some(cleaned.to_string());
        }
    }

    // Fallback: read from /proc/cpuinfo
    if let Ok(cpuinfo) = fs::read_to_string("/proc/cpuinfo") {
        for line in cpuinfo.lines() {
            if line.starts_with("Model") {
                if let Some(model) = line.split_once(':') {
                    return Some(model.1.trim().to_string());
                }
            }
        }
    }

    None
}

// Read CPU temperature from Raspberry Pi thermal zone
fn read_cpu_temperature() -> Result<f32, std::io::Error> {
    // Pi-specific temperature paths in order of preference
    let temp_paths = [
        "/sys/class/thermal/thermal_zone0/temp", // Most common
        "/sys/devices/virtual/thermal/thermal_zone0/temp", // Alternative path
        "/sys/class/hwmon/hwmon0/temp1_input",   // Hardware monitor
        "/sys/class/hwmon/hwmon1/temp1_input",   // Secondary hwmon
    ];

    // Try Pi-specific paths first
    for path in &temp_paths {
        if let Ok(temp_str) = fs::read_to_string(path) {
            if let Ok(temp_millidegrees) = temp_str.trim().parse::<i32>() {
                let temp_celsius = temp_millidegrees as f32 / 1000.0;
                // Sanity check: temperature should be reasonable (0-100°C)
                if temp_celsius > 0.0 && temp_celsius < 100.0 {
                    return Ok(temp_celsius);
                }
            }
        }
    }

    // Try vcgencmd (Raspberry Pi specific)
    if let Ok(output) = Command::new("vcgencmd").arg("measure_temp").output() {
        if output.status.success() {
            let temp_output = String::from_utf8_lossy(&output.stdout);
            // Parse "temp=XX.X'C" format
            if let Some(start) = temp_output.find("temp=") {
                let temp_part = &temp_output[start + 5..];
                if let Some(end) = temp_part.find("'") {
                    let temp_str = &temp_part[..end];
                    if let Ok(temp) = temp_str.parse::<f32>() {
                        if temp > 0.0 && temp < 100.0 {
                            return Ok(temp);
                        }
                    }
                }
            }
        }
    }

    // Final fallback: try other thermal zones
    for i in 0..10 {
        let path = format!("/sys/class/thermal/thermal_zone{}/temp", i);
        if let Ok(temp_str) = fs::read_to_string(&path) {
            if let Ok(temp_millidegrees) = temp_str.trim().parse::<i32>() {
                let temp_celsius = temp_millidegrees as f32 / 1000.0;
                if temp_celsius > 0.0 && temp_celsius < 100.0 {
                    return Ok(temp_celsius);
                }
            }
        }
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "No valid thermal zone found",
    ))
}

// API endpoint for metrics
async fn get_metrics(State(state): State<AppState>) -> Json<SystemSnapshot> {
    let snapshot = state.latest_snapshot.read().await.clone();
    Json(snapshot)
}

// Dashboard HTML
async fn dashboard() -> Html<&'static str> {
    Html(include_str!("../static/index.html"))
}
