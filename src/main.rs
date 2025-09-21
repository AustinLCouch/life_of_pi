//! Life of Pi - Raspberry Pi System Diagnostics Binary
//!
//! A standalone binary for real-time Raspberry Pi system monitoring with web interface.

use clap::{Args, Parser, Subcommand};
use life_of_pi::{
    start_web_server, SystemCollector, SystemMonitor, WebConfig, DEFAULT_INTERVAL_MS,
    DEFAULT_WEB_PORT,
};
use tracing::{error, info, Level};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

#[derive(Parser)]
#[command(name = "life_of_pi")]
#[command(about = "🥧 Life of Pi - Raspberry Pi System Diagnostics")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(author = "Austin Couch")]
#[command(long_about = "A real-time system monitoring tool for Raspberry Pi with web interface")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Web server bind address
    #[arg(long, default_value = "0.0.0.0")]
    host: String,

    /// Web server port
    #[arg(short, long, default_value_t = DEFAULT_WEB_PORT)]
    port: u16,

    /// System metrics collection interval in milliseconds
    #[arg(short, long, default_value_t = DEFAULT_INTERVAL_MS)]
    interval: u64,

    /// Disable GPIO monitoring (useful for non-Pi systems)
    #[arg(long)]
    no_gpio: bool,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the web server (default)
    Serve(ServeArgs),

    /// Get a single system snapshot and exit
    Snapshot(SnapshotArgs),

    /// Show system information
    Info,
}

#[derive(Args)]
struct ServeArgs {
    /// Static files directory (optional)
    #[arg(long)]
    static_dir: Option<String>,

    /// Disable CORS headers
    #[arg(long)]
    no_cors: bool,

    /// Maximum WebSocket connections
    #[arg(long, default_value_t = 100)]
    max_connections: usize,
}

#[derive(Args)]
struct SnapshotArgs {
    /// Output format: json, yaml, or pretty
    #[arg(short, long, default_value = "pretty")]
    format: String,

    /// Include GPIO information (if available)
    #[cfg(feature = "gpio")]
    #[arg(long)]
    include_gpio: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Initialize tracing/logging
    init_logging(&cli)?;

    // Print banner
    print_banner();

    match &cli.command {
        Some(Commands::Serve(args)) => {
            serve_command(&cli, args).await?;
        }
        Some(Commands::Snapshot(args)) => {
            snapshot_command(&cli, args).await?;
        }
        Some(Commands::Info) => {
            info_command().await?;
        }
        None => {
            // Default to serve command
            let serve_args = ServeArgs {
                static_dir: None,
                no_cors: false,
                max_connections: 100,
            };
            serve_command(&cli, &serve_args).await?;
        }
    }

    Ok(())
}

fn init_logging(cli: &Cli) -> Result<(), Box<dyn std::error::Error>> {
    let level = if cli.debug {
        Level::DEBUG
    } else if cli.verbose {
        Level::INFO
    } else {
        Level::WARN
    };

    let subscriber = FmtSubscriber::builder()
        .with_max_level(level)
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(false)
        .compact()
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    Ok(())
}

fn print_banner() {
    println!("🥧 Life of Pi - Raspberry Pi System Diagnostics");
    println!("   Version: {}", env!("CARGO_PKG_VERSION"));
    println!("   Built for real-time Pi monitoring");
    println!();
}

async fn serve_command(cli: &Cli, args: &ServeArgs) -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting Life of Pi system monitor...");

    // Create system collector
    let mut collector = SystemCollector::new()?;
    info!("System collector initialized");

    // Start metrics collection stream
    let stream = collector
        .start_collecting_with_interval(cli.interval)
        .await?;
    info!(
        "Started metrics collection with {}ms interval",
        cli.interval
    );

    // Configure web server
    let mut web_config = WebConfig::new(&cli.host, cli.port);

    if let Some(static_dir) = &args.static_dir {
        web_config = web_config.with_static_path(Some(static_dir.clone()));
        info!("Using static files from: {}", static_dir);
    }

    web_config = web_config
        .with_cors(!args.no_cors)
        .with_max_websocket_connections(args.max_connections);

    if cli.no_gpio {
        info!("GPIO monitoring disabled");
    } else {
        #[cfg(feature = "gpio")]
        info!("GPIO monitoring enabled");

        #[cfg(not(feature = "gpio"))]
        info!("GPIO monitoring not available (feature not compiled)");
    }

    info!("Web server configuration:");
    info!("  - Bind address: {}:{}", cli.host, cli.port);
    info!("  - CORS enabled: {}", !args.no_cors);
    info!("  - Max WebSocket connections: {}", args.max_connections);
    info!("  - Metrics interval: {}ms", cli.interval);

    // Start web server
    info!("Starting web server...");
    start_web_server(web_config, stream).await?;

    Ok(())
}

async fn snapshot_command(
    _cli: &Cli,
    args: &SnapshotArgs,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut collector = SystemCollector::new()?;
    let snapshot = collector.get_snapshot().await?;

    match args.format.as_str() {
        "json" => {
            let json = serde_json::to_string_pretty(&snapshot)?;
            println!("{}", json);
        }
        "pretty" => {
            print_pretty_snapshot(&snapshot);
        }
        _ => {
            error!(
                "Unsupported format: {}. Use 'json' or 'pretty'",
                args.format
            );
            std::process::exit(1);
        }
    }

    Ok(())
}

async fn info_command() -> Result<(), Box<dyn std::error::Error>> {
    println!("🥧 Life of Pi System Information");
    println!("================================");
    println!();

    let mut collector = SystemCollector::new()?;
    let snapshot = collector.get_snapshot().await?;

    println!("System Details:");
    println!("  Hostname: {}", snapshot.system.hostname);
    println!(
        "  OS: {} {}",
        snapshot.system.os_name, snapshot.system.os_version
    );
    println!("  Kernel: {}", snapshot.system.kernel_version);
    println!("  Uptime: {} seconds", snapshot.system.uptime_seconds);
    println!();

    println!("Hardware:");
    println!(
        "  CPU: {} ({} cores)",
        snapshot.cpu.model, snapshot.cpu.cores
    );
    println!("  Architecture: {}", snapshot.cpu.architecture);
    println!(
        "  Memory: {:.1} GB total",
        snapshot.memory.total_bytes as f64 / 1024.0 / 1024.0 / 1024.0
    );

    if let Some(temp) = snapshot.temperature.cpu_celsius {
        println!("  CPU Temperature: {:.1}°C", temp);
    }

    println!();

    println!("Storage:");
    for storage in &snapshot.storage {
        println!(
            "  {}: {:.1} GB total, {:.1}% used",
            storage.mount_point,
            storage.total_bytes as f64 / 1024.0 / 1024.0 / 1024.0,
            storage.usage_percent
        );
    }

    println!();

    println!("Network Interfaces:");
    for iface in &snapshot.network {
        println!(
            "  {}: {}",
            iface.interface,
            if iface.is_up { "UP" } else { "DOWN" }
        );
    }

    #[cfg(feature = "gpio")]
    {
        println!();
        println!("GPIO:");
        if snapshot.gpio.gpio_available {
            println!("  Available pins: {}", snapshot.gpio.available_pins.len());
            println!("  GPIO support: enabled");
        } else {
            println!("  GPIO support: not available");
        }
    }

    println!();
    println!("Features compiled:");
    #[cfg(feature = "gpio")]
    println!("  - GPIO support: ✓");
    #[cfg(not(feature = "gpio"))]
    println!("  - GPIO support: ✗");

    Ok(())
}

fn print_pretty_snapshot(snapshot: &life_of_pi::SystemSnapshot) {
    println!(
        "🥧 System Snapshot ({})",
        chrono::DateTime::from_timestamp_millis(snapshot.timestamp as i64)
            .unwrap_or_default()
            .format("%Y-%m-%d %H:%M:%S UTC")
    );
    println!("==========================================");
    println!();

    // CPU info
    println!("⚡ CPU:");
    println!("  Model: {}", snapshot.cpu.model);
    println!("  Cores: {}", snapshot.cpu.cores);
    println!("  Usage: {:.1}%", snapshot.cpu.usage_percent);
    println!("  Frequency: {} MHz", snapshot.cpu.frequency_mhz);
    if let Some(governor) = &snapshot.cpu.governor {
        println!("  Governor: {}", governor);
    }
    println!(
        "  Load: {:.2}, {:.2}, {:.2}",
        snapshot.cpu.load_average.one_minute,
        snapshot.cpu.load_average.five_minutes,
        snapshot.cpu.load_average.fifteen_minutes
    );
    println!();

    // Memory info
    println!("🧠 Memory:");
    println!(
        "  Total: {:.1} GB",
        snapshot.memory.total_bytes as f64 / 1024.0 / 1024.0 / 1024.0
    );
    println!(
        "  Available: {:.1} GB",
        snapshot.memory.available_bytes as f64 / 1024.0 / 1024.0 / 1024.0
    );
    println!("  Usage: {:.1}%", snapshot.memory.usage_percent);
    println!(
        "  Swap: {:.1} GB used",
        snapshot.memory.swap.used_bytes as f64 / 1024.0 / 1024.0 / 1024.0
    );
    println!();

    // Temperature info
    println!("🌡️  Temperature:");
    if let Some(cpu_temp) = snapshot.temperature.cpu_celsius {
        println!("  CPU: {:.1}°C", cpu_temp);
    }
    if let Some(gpu_temp) = snapshot.temperature.gpu_celsius {
        println!("  GPU: {:.1}°C", gpu_temp);
    }
    if snapshot.temperature.is_throttling {
        println!("  Status: 🔥 THROTTLING");
    } else {
        println!("  Status: ✅ Normal");
    }
    println!();

    // Storage info
    if !snapshot.storage.is_empty() {
        println!("💾 Storage:");
        for storage in &snapshot.storage {
            println!(
                "  {}: {:.1} GB total, {:.1}% used",
                storage.mount_point,
                storage.total_bytes as f64 / 1024.0 / 1024.0 / 1024.0,
                storage.usage_percent
            );
        }
        println!();
    }

    // Network info
    if !snapshot.network.is_empty() {
        println!("🌐 Network:");
        for iface in &snapshot.network {
            println!(
                "  {}: {} (TX: {:.1} MB, RX: {:.1} MB)",
                iface.interface,
                if iface.is_up { "UP" } else { "DOWN" },
                iface.tx_bytes as f64 / 1024.0 / 1024.0,
                iface.rx_bytes as f64 / 1024.0 / 1024.0
            );
        }
        println!();
    }

    // System info
    println!("🖥️  System:");
    println!("  Hostname: {}", snapshot.system.hostname);
    println!(
        "  OS: {} {}",
        snapshot.system.os_name, snapshot.system.os_version
    );
    println!("  Uptime: {} seconds", snapshot.system.uptime_seconds);
    println!("  Processes: {}", snapshot.system.process_count);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parsing() {
        use clap::Parser;

        let cli = Cli::try_parse_from(["life_of_pi", "--port", "9090"]).unwrap();
        assert_eq!(cli.port, 9090);
    }

    #[test]
    fn test_default_values() {
        use clap::Parser;

        let cli = Cli::try_parse_from(["life_of_pi"]).unwrap();
        assert_eq!(cli.port, DEFAULT_WEB_PORT);
        assert_eq!(cli.interval, DEFAULT_INTERVAL_MS);
        assert_eq!(cli.host, "0.0.0.0");
    }
}
