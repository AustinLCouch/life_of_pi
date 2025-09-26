use axum::{
    extract::State,
    response::{Html, Json},
    routing::{get, Router},
    serve,
};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    net::SocketAddr,
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
    }
}

// Read CPU temperature from Raspberry Pi thermal zone
fn read_cpu_temperature() -> Result<f32, std::io::Error> {
    // Try Raspberry Pi thermal zone first
    if let Ok(temp_str) = fs::read_to_string("/sys/class/thermal/thermal_zone0/temp") {
        if let Ok(temp_millidegrees) = temp_str.trim().parse::<i32>() {
            return Ok(temp_millidegrees as f32 / 1000.0);
        }
    }

    // Fallback: try common thermal zone files
    for i in 0..5 {
        let path = format!("/sys/class/thermal/thermal_zone{}/temp", i);
        if let Ok(temp_str) = fs::read_to_string(&path) {
            if let Ok(temp_millidegrees) = temp_str.trim().parse::<i32>() {
                return Ok(temp_millidegrees as f32 / 1000.0);
            }
        }
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "No thermal zone found",
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
