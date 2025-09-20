# 🥧 Life of Pi - Raspberry Pi System Diagnostics

[![Rust](https://github.com/austincouch/life_of_pi/workflows/CI/badge.svg)](https://github.com/austincouch/life_of_pi/actions)
[![Crates.io](https://img.shields.io/crates/v/life_of_pi)](https://crates.io/crates/life_of_pi)
[![Documentation](https://docs.rs/life_of_pi/badge.svg)](https://docs.rs/life_of_pi)

A clean, minimalist Rust crate for real-time Raspberry Pi system monitoring with a beautiful web interface. Designed specifically for plug-and-play operation on Raspberry Pi 5 running RaspberryOS x64.

## ✨ Features

- **🔄 Real-time System Monitoring**: CPU usage, temperature, memory, storage, network metrics
- **🔌 GPIO Status Monitoring**: Pin states and availability (feature-gated for cross-compilation)
- **🌐 Web Dashboard**: Beautiful, responsive web interface with live charts
- **📊 WebSocket Streaming**: Real-time data updates with minimal latency
- **🖥️ CLI Tools**: Both library crate and standalone binary
- **🏗️ Cross-compilation**: Build on macOS/Linux for Raspberry Pi deployment
- **⚡ High Performance**: Efficient async implementation with minimal overhead
- **🛡️ Safe & Idiomatic**: Written in safe Rust with comprehensive error handling

## 🚀 Quick Start

### Using the Binary

1. **Download the latest release** for Raspberry Pi (aarch64):
   ```bash
   wget https://github.com/austincouch/life_of_pi/releases/latest/download/life_of_pi-aarch64
   chmod +x life_of_pi-aarch64
   sudo mv life_of_pi-aarch64 /usr/local/bin/life_of_pi
   ```

2. **Run the monitor**:
   ```bash
   life_of_pi
   ```

3. **Open your browser** and navigate to `http://your-pi:8080`

### Using as a Library

Add to your `Cargo.toml`:

```toml
[dependencies]
life_of_pi = "0.1"
```

Basic usage:

```rust
use life_of_pi::{SystemCollector, SystemMonitor, WebConfig, start_web_server};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a system collector
    let mut collector = SystemCollector::new()?;
    
    // Get a single snapshot
    let snapshot = collector.get_snapshot().await?;
    println!("CPU Usage: {:.1}%", snapshot.cpu.usage_percent);
    
    // Start real-time monitoring with web server
    let stream = collector.start_collecting().await?;
    let config = WebConfig::default().with_port(8080);
    start_web_server(config, stream).await?;
    
    Ok(())
}
```

## 🖥️ Command Line Interface

```bash
# Start web server on custom port
life_of_pi --port 9090

# Start with custom metrics interval
life_of_pi --interval 1000

# Get a single system snapshot as JSON
life_of_pi snapshot --format json

# Show detailed system information
life_of_pi info

# Enable verbose logging
life_of_pi --verbose

# Use external static files
life_of_pi serve --static-dir ./my-dashboard
```

## 🏗️ Cross-compilation for Raspberry Pi

### Prerequisites

Install `cross`:
```bash
cargo install cross
```

### Build for Raspberry Pi 5 (aarch64)

```bash
# Build with GPIO support for Raspberry Pi
cross build --target aarch64-unknown-linux-gnu --release --features gpio

# The binary will be in target/aarch64-unknown-linux-gnu/release/life_of_pi
```

### Transfer to Raspberry Pi

```bash
scp target/aarch64-unknown-linux-gnu/release/life_of_pi pi@your-pi-ip:/home/pi/
```

## 📊 Web Dashboard

The web interface provides real-time monitoring with:

- **📈 Live CPU Usage** - Per-core and aggregate statistics
- **🌡️ Temperature Monitoring** - CPU/GPU temps with throttling alerts
- **🧠 Memory Usage** - RAM and swap utilization
- **💾 Storage Info** - Disk usage across all mounted filesystems
- **🌐 Network Status** - Interface status and traffic statistics
- **🔌 GPIO Status** - Pin states and configurations (when enabled)
- **⚡ System Health** - Uptime, load averages, and process count

## 🔧 Configuration

### Features

- `gpio` (optional) - Enable GPIO monitoring with `rppal`
- Default: No features enabled for cross-compilation compatibility

### Environment Variables

- `RUST_LOG` - Configure logging level (`debug`, `info`, `warn`, `error`)
- `LIFE_OF_PI_PORT` - Default web server port

## 🏛️ Architecture

Life of Pi follows a clean MVC architecture:

```
src/
├── lib.rs              # Public API and re-exports
├── main.rs             # CLI binary
├── error.rs            # Unified error handling
├── metrics/            # Model - System data collection
│   ├── collector.rs    # Core metrics collector
│   ├── data.rs         # Data structures
│   ├── gpio.rs         # GPIO support (feature-gated)
│   └── traits.rs       # Monitoring traits
└── web/                # Controller - Web server
    ├── config.rs       # Web server configuration
    ├── handlers.rs     # HTTP request handlers
    ├── router.rs       # Route definitions
    └── websocket.rs    # WebSocket streaming
```

## 📋 System Requirements

### Raspberry Pi
- **Raspberry Pi 5** (recommended) or Pi 4
- **RaspberryOS 64-bit** (Bookworm recommended)
- **1GB RAM** minimum
- **Network connectivity** for web access

### Development
- **Rust 1.70+** (MSRV)
- **tokio** async runtime
- For cross-compilation: `cross` and Docker

## 🧪 Testing

```bash
# Run tests (without GPIO features)
cargo test

# Run tests with all features
cargo test --features gpio

# Run clippy for linting
cargo clippy

# Format code
cargo fmt
```

## 📝 License

This project is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## ⚠️ Note on GPIO Support

GPIO functionality is feature-gated to allow compilation on non-Raspberry Pi systems. When running on macOS or Linux without the `gpio` feature, GPIO monitoring will be disabled but all other functionality remains available.

---

Made with ❤️ for the Raspberry Pi community