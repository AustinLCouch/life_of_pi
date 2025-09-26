# 🥧 Life of Pi - Raspberry Pi Monitor

A clean, minimal Rust application for real-time Raspberry Pi system monitoring with a beautiful web dashboard.

## ✨ Features

- **🔄 Real-time System Monitoring**: CPU usage, temperature, memory, disk, network
- **📊 System Information Display**: Hostname, IP addresses, OS version, Pi model, uptime, load averages
- **🌐 Beautiful Web Dashboard**: Responsive interface with live charts and system info
- **🎯 Smart Platform Detection**: Automatically detects Pi vs non-Pi systems with appropriate UI
- **⚡ High Performance**: Efficient async implementation with minimal 1MB binary
- **🏗️ Cross-compilation**: Build on macOS/Linux for Raspberry Pi deployment
- **🛡️ Safe & Clean**: Simple, focused single-file codebase following Rust best practices

## 🚀 Quick Start

### 1. Cross-compile for Raspberry Pi

```bash
# Install cross-compilation target (you already have this!)
rustup target add aarch64-unknown-linux-gnu

# Build for Raspberry Pi
make pi
# or manually: cargo build --release --target aarch64-unknown-linux-gnu
```

### 2. Deploy to your Raspberry Pi

```bash
# Copy binary to your Pi (update the IP address)
scp target/aarch64-unknown-linux-gnu/release/life_of_pi pi@YOUR_PI_IP:/home/pi/
```

### 3. Run on Raspberry Pi

```bash
# SSH into your Pi and run
ssh pi@YOUR_PI_IP
./life_of_pi
```

### 4. View the dashboard

Open your browser to `http://YOUR_PI_IP:8080` to see the beautiful monitoring dashboard!

## 🖥️ Development

```bash
# Run locally for development (will show mock data on non-Pi systems)
cargo run --target aarch64-apple-darwin
# or: make run

# Check code quality
make check

# Format code
make fmt
```

## 📊 What it monitors

**💻 System Information:**
- **Hostname & User**: Current system identity
- **IP Addresses**: Local network addresses with multi-IP support
- **Operating System**: OS version and kernel information  
- **Pi Model**: Raspberry Pi model detection (if applicable)
- **System Uptime**: Human-readable uptime display
- **Load Averages**: 1m, 5m, 15m system load indicators

**📈 Real-time Metrics:**
- **CPU Usage**: Real-time percentage with history charts
- **CPU Temperature**: Enhanced thermal monitoring with Pi-specific sensors
- **Memory Usage**: RAM utilization with detailed breakdown
- **Disk Usage**: Root filesystem usage with formatted display
- **Network Traffic**: Total RX/TX across all interfaces

## 🏛️ Simple Architecture

```
src/
└── main.rs              # Single-file application, ~200 lines
static/
└── index.html           # Beautiful web dashboard
Makefile                 # Build & deployment helpers
```

## 📋 Requirements

- **Rust 1.75+**
- **Raspberry Pi 4/5** with network access
- **Cross-compilation tools** (handled automatically)

## 📝 License

MIT OR Apache-2.0
