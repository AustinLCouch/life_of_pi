//! Data structures for system metrics.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A complete snapshot of system metrics at a point in time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemSnapshot {
    /// Timestamp when this snapshot was taken (Unix timestamp in milliseconds)
    pub timestamp: u64,
    /// CPU information and usage statistics
    pub cpu: CpuInfo,
    /// Memory usage information
    pub memory: MemoryInfo,
    /// Storage device information
    pub storage: Vec<StorageInfo>,
    /// Network interface information
    pub network: Vec<NetworkInfo>,
    /// System temperature sensors
    pub temperature: TemperatureInfo,
    /// General system information
    pub system: SystemInfo,
    /// GPIO pin status (only available with gpio feature)
    #[cfg(feature = "gpio")]
    pub gpio: super::gpio::GpioStatus,
}

/// CPU information and usage statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuInfo {
    /// CPU model name
    pub model: String,
    /// Number of CPU cores
    pub cores: u32,
    /// CPU architecture (e.g., "aarch64")
    pub architecture: String,
    /// Current CPU frequency in MHz
    pub frequency_mhz: u32,
    /// CPU usage percentage (0.0 to 100.0)
    pub usage_percent: f32,
    /// Per-core CPU usage percentages
    pub core_usage: Vec<f32>,
    /// Current CPU governor (e.g., "ondemand", "performance")
    pub governor: Option<String>,
    /// Load averages (1, 5, 15 minutes)
    pub load_average: LoadAverage,
}

/// System load averages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadAverage {
    pub one_minute: f64,
    pub five_minutes: f64,
    pub fifteen_minutes: f64,
}

/// Memory usage information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInfo {
    /// Total system memory in bytes
    pub total_bytes: u64,
    /// Available memory in bytes
    pub available_bytes: u64,
    /// Used memory in bytes
    pub used_bytes: u64,
    /// Memory usage percentage (0.0 to 100.0)
    pub usage_percent: f32,
    /// Swap information
    pub swap: SwapInfo,
    /// Memory breakdown by type
    pub breakdown: MemoryBreakdown,
}

/// Swap memory information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapInfo {
    /// Total swap space in bytes
    pub total_bytes: u64,
    /// Used swap space in bytes
    pub used_bytes: u64,
    /// Free swap space in bytes
    pub free_bytes: u64,
}

/// Memory usage breakdown by type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryBreakdown {
    /// Buffers in bytes
    pub buffers_bytes: u64,
    /// Cached memory in bytes
    pub cached_bytes: u64,
    /// Shared memory in bytes
    pub shared_bytes: u64,
}

/// Storage device information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageInfo {
    /// Device name (e.g., "/dev/mmcblk0p2")
    pub device: String,
    /// Mount point (e.g., "/", "/boot")
    pub mount_point: String,
    /// Filesystem type (e.g., "ext4", "vfat")
    pub filesystem: String,
    /// Total space in bytes
    pub total_bytes: u64,
    /// Available space in bytes
    pub available_bytes: u64,
    /// Used space in bytes
    pub used_bytes: u64,
    /// Usage percentage (0.0 to 100.0)
    pub usage_percent: f32,
}

/// Network interface information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInfo {
    /// Interface name (e.g., "wlan0", "eth0")
    pub interface: String,
    /// Whether the interface is up
    pub is_up: bool,
    /// MAC address
    pub mac_address: Option<String>,
    /// IPv4 addresses
    pub ipv4_addresses: Vec<String>,
    /// IPv6 addresses
    pub ipv6_addresses: Vec<String>,
    /// Bytes transmitted
    pub tx_bytes: u64,
    /// Bytes received
    pub rx_bytes: u64,
    /// Packets transmitted
    pub tx_packets: u64,
    /// Packets received
    pub rx_packets: u64,
    /// Transmission errors
    pub tx_errors: u64,
    /// Receive errors
    pub rx_errors: u64,
}

/// Temperature sensor information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemperatureInfo {
    /// CPU temperature in Celsius
    pub cpu_celsius: Option<f32>,
    /// GPU temperature in Celsius (if available)
    pub gpu_celsius: Option<f32>,
    /// Additional thermal zones by name
    pub thermal_zones: HashMap<String, f32>,
    /// Whether thermal throttling is active
    pub is_throttling: bool,
}

/// General system information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    /// System hostname
    pub hostname: String,
    /// Operating system name
    pub os_name: String,
    /// Operating system version
    pub os_version: String,
    /// Kernel version
    pub kernel_version: String,
    /// System uptime in seconds
    pub uptime_seconds: u64,
    /// Boot time (Unix timestamp)
    pub boot_time: u64,
    /// Number of processes running
    pub process_count: u64,
}

impl SystemSnapshot {
    /// Create a new system snapshot with the current timestamp.
    pub fn new() -> Self {
        Self {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            cpu: CpuInfo::default(),
            memory: MemoryInfo::default(),
            storage: Vec::new(),
            network: Vec::new(),
            temperature: TemperatureInfo::default(),
            system: SystemInfo::default(),
            #[cfg(feature = "gpio")]
            gpio: super::gpio::GpioStatus::default(),
        }
    }
}

impl Default for CpuInfo {
    fn default() -> Self {
        Self {
            model: String::new(),
            cores: 0,
            architecture: String::new(),
            frequency_mhz: 0,
            usage_percent: 0.0,
            core_usage: Vec::new(),
            governor: None,
            load_average: LoadAverage::default(),
        }
    }
}

impl Default for LoadAverage {
    fn default() -> Self {
        Self {
            one_minute: 0.0,
            five_minutes: 0.0,
            fifteen_minutes: 0.0,
        }
    }
}

impl Default for MemoryInfo {
    fn default() -> Self {
        Self {
            total_bytes: 0,
            available_bytes: 0,
            used_bytes: 0,
            usage_percent: 0.0,
            swap: SwapInfo::default(),
            breakdown: MemoryBreakdown::default(),
        }
    }
}

impl Default for SwapInfo {
    fn default() -> Self {
        Self {
            total_bytes: 0,
            used_bytes: 0,
            free_bytes: 0,
        }
    }
}

impl Default for MemoryBreakdown {
    fn default() -> Self {
        Self {
            buffers_bytes: 0,
            cached_bytes: 0,
            shared_bytes: 0,
        }
    }
}

impl Default for TemperatureInfo {
    fn default() -> Self {
        Self {
            cpu_celsius: None,
            gpu_celsius: None,
            thermal_zones: HashMap::new(),
            is_throttling: false,
        }
    }
}

impl Default for SystemInfo {
    fn default() -> Self {
        Self {
            hostname: String::new(),
            os_name: String::new(),
            os_version: String::new(),
            kernel_version: String::new(),
            uptime_seconds: 0,
            boot_time: 0,
            process_count: 0,
        }
    }
}