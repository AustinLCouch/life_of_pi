//! Core system metrics collection implementation.

use crate::error::{Result, SystemError};
use crate::metrics::{
    data::*,
    traits::{MetricsProvider, SystemMonitor},
};
use futures_util::stream::{self, BoxStream};
use std::collections::HashMap;
use std::fs;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use sysinfo::{System, Disks, Networks};
use tokio::time;

#[cfg(feature = "gpio")]
use crate::metrics::gpio::{DefaultGpioProvider, GpioProvider};

/// System metrics collector using sysinfo and direct /proc access.
pub struct SystemCollector {
    system: System,
    disks: Disks,
    networks: Networks,
    #[cfg(feature = "gpio")]
    gpio_provider: Option<DefaultGpioProvider>,
}

impl SystemCollector {
    /// Create a new system collector instance.
    pub fn new() -> Result<Self> {
        let mut system = System::new_all();
        system.refresh_all();
        let mut disks = Disks::new_with_refreshed_list();
        disks.refresh();
        let mut networks = Networks::new_with_refreshed_list();
        networks.refresh();
        
        #[cfg(feature = "gpio")]
        let gpio_provider = match DefaultGpioProvider::new() {
            Ok(provider) => Some(provider),
            Err(_) => {
                // GPIO initialization failed, continue without GPIO support
                tracing::warn!("Failed to initialize GPIO support, continuing without GPIO");
                None
            }
        };
        
        Ok(Self {
            system,
            disks,
            networks,
            #[cfg(feature = "gpio")]
            gpio_provider,
        })
    }
    
    /// Refresh system information.
    fn refresh(&mut self) {
        self.system.refresh_all();
        self.disks.refresh();
        self.networks.refresh();
    }
    
    /// Collect CPU information.
    fn collect_cpu_info(&self) -> Result<CpuInfo> {
        let cpus = self.system.cpus();
        
        if cpus.is_empty() {
            return Err(SystemError::system_error("No CPU information available"));
        }
        
        // Get CPU model from the first CPU (they should all be the same on Pi)
        let model = cpus[0].brand().to_string();
        let cores = cpus.len() as u32;
        
        // Calculate overall CPU usage
        let usage_percent = cpus.iter().map(|cpu| cpu.cpu_usage()).sum::<f32>() / cores as f32;
        
        // Get per-core usage
        let core_usage: Vec<f32> = cpus.iter().map(|cpu| cpu.cpu_usage()).collect();
        
        // Get architecture from /proc/cpuinfo if available
        let architecture = self.read_cpu_architecture().unwrap_or_else(|| "unknown".to_string());
        
        // Get CPU frequency from /sys/devices/system/cpu/cpu0/cpufreq/scaling_cur_freq
        let frequency_mhz = self.read_cpu_frequency().unwrap_or(0);
        
        // Get CPU governor
        let governor = self.read_cpu_governor();
        
        // Get load averages
        let load_average = self.read_load_average().unwrap_or_default();
        
        Ok(CpuInfo {
            model,
            cores,
            architecture,
            frequency_mhz,
            usage_percent,
            core_usage,
            governor,
            load_average,
        })
    }
    
    /// Read CPU architecture from /proc/cpuinfo.
    fn read_cpu_architecture(&self) -> Option<String> {
        let cpuinfo = fs::read_to_string("/proc/cpuinfo").ok()?;
        
        for line in cpuinfo.lines() {
            if line.starts_with("architecture") {
                if let Some((_, arch)) = line.split_once(':') {
                    return Some(arch.trim().to_string());
                }
            }
        }
        
        // Fallback: try to determine from processor info
        if cpuinfo.contains("aarch64") || cpuinfo.contains("ARMv8") {
            Some("aarch64".to_string())
        } else if cpuinfo.contains("arm") {
            Some("arm".to_string())
        } else if cpuinfo.contains("x86_64") {
            Some("x86_64".to_string())
        } else {
            None
        }
    }
    
    /// Read current CPU frequency in MHz.
    fn read_cpu_frequency(&self) -> Option<u32> {
        let freq_khz = fs::read_to_string("/sys/devices/system/cpu/cpu0/cpufreq/scaling_cur_freq")
            .ok()?
            .trim()
            .parse::<u32>()
            .ok()?;
        
        Some(freq_khz / 1000) // Convert kHz to MHz
    }
    
    /// Read CPU governor.
    fn read_cpu_governor(&self) -> Option<String> {
        fs::read_to_string("/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor")
            .ok()
            .map(|s| s.trim().to_string())
    }
    
    /// Read system load averages.
    fn read_load_average(&self) -> Option<LoadAverage> {
        let loadavg = fs::read_to_string("/proc/loadavg").ok()?;
        let parts: Vec<&str> = loadavg.split_whitespace().collect();
        
        if parts.len() >= 3 {
            let one_minute = parts[0].parse().ok()?;
            let five_minutes = parts[1].parse().ok()?;
            let fifteen_minutes = parts[2].parse().ok()?;
            
            Some(LoadAverage {
                one_minute,
                five_minutes,
                fifteen_minutes,
            })
        } else {
            None
        }
    }
    
    /// Collect memory information.
    fn collect_memory_info(&self) -> Result<MemoryInfo> {
        let total_bytes = self.system.total_memory();
        let available_bytes = self.system.available_memory();
        let used_bytes = self.system.used_memory();
        
        let usage_percent = if total_bytes > 0 {
            (used_bytes as f32 / total_bytes as f32) * 100.0
        } else {
            0.0
        };
        
        // Get swap information
        let swap = SwapInfo {
            total_bytes: self.system.total_swap(),
            used_bytes: self.system.used_swap(),
            free_bytes: self.system.free_swap(),
        };
        
        // Get memory breakdown from /proc/meminfo
        let breakdown = self.read_memory_breakdown().unwrap_or_default();
        
        Ok(MemoryInfo {
            total_bytes,
            available_bytes,
            used_bytes,
            usage_percent,
            swap,
            breakdown,
        })
    }
    
    /// Read detailed memory breakdown from /proc/meminfo.
    fn read_memory_breakdown(&self) -> Option<MemoryBreakdown> {
        let meminfo = fs::read_to_string("/proc/meminfo").ok()?;
        let mut buffers_bytes = 0;
        let mut cached_bytes = 0;
        let mut shared_bytes = 0;
        
        for line in meminfo.lines() {
            if let Some((key, value_str)) = line.split_once(':') {
                if let Some(value_kb) = value_str.trim().split_whitespace().next() {
                    if let Ok(kb) = value_kb.parse::<u64>() {
                        let bytes = kb * 1024; // Convert kB to bytes
                        match key {
                            "Buffers" => buffers_bytes = bytes,
                            "Cached" => cached_bytes = bytes,
                            "Shmem" => shared_bytes = bytes,
                            _ => {}
                        }
                    }
                }
            }
        }
        
        Some(MemoryBreakdown {
            buffers_bytes,
            cached_bytes,
            shared_bytes,
        })
    }
    
    /// Collect storage information.
    fn collect_storage_info(&self) -> Vec<StorageInfo> {
        self.disks
            .iter()
            .map(|disk| {
                let total_bytes = disk.total_space();
                let available_bytes = disk.available_space();
                let used_bytes = total_bytes - available_bytes;
                
                let usage_percent = if total_bytes > 0 {
                    (used_bytes as f32 / total_bytes as f32) * 100.0
                } else {
                    0.0
                };
                
                StorageInfo {
                    device: disk.name().to_string_lossy().to_string(),
                    mount_point: disk.mount_point().to_string_lossy().to_string(),
                    filesystem: disk.file_system().to_string_lossy().to_string(),
                    total_bytes,
                    available_bytes,
                    used_bytes,
                    usage_percent,
                }
            })
            .collect()
    }
    
    /// Collect network information.
    fn collect_network_info(&self) -> Vec<NetworkInfo> {
        self.networks
            .iter()
            .map(|(interface_name, network)| {
                NetworkInfo {
                    interface: interface_name.clone(),
                    is_up: network.total_transmitted() > 0 || network.total_received() > 0,
                    mac_address: None, // sysinfo doesn't provide MAC addresses
                    ipv4_addresses: Vec::new(), // Would need additional parsing
                    ipv6_addresses: Vec::new(), // Would need additional parsing  
                    tx_bytes: network.total_transmitted(),
                    rx_bytes: network.total_received(),
                    tx_packets: network.total_packets_transmitted(),
                    rx_packets: network.total_packets_received(),
                    tx_errors: network.total_errors_on_transmitted(),
                    rx_errors: network.total_errors_on_received(),
                }
            })
            .collect()
    }
    
    /// Collect temperature information.
    fn collect_temperature_info(&self) -> Result<TemperatureInfo> {
        let mut thermal_zones = HashMap::new();
        let mut cpu_celsius = None;
        let mut gpu_celsius = None;
        let mut is_throttling = false;
        
        // Read CPU temperature from Raspberry Pi thermal zone
        if let Ok(temp_str) = fs::read_to_string("/sys/class/thermal/thermal_zone0/temp") {
            if let Ok(temp_millicelsius) = temp_str.trim().parse::<i32>() {
                let temp_celsius = temp_millicelsius as f32 / 1000.0;
                cpu_celsius = Some(temp_celsius);
                thermal_zones.insert("cpu".to_string(), temp_celsius);
            }
        }
        
        // Try to read GPU temperature (Raspberry Pi specific)
        if let Ok(output) = std::process::Command::new("vcgencmd").arg("measure_temp").output() {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                if let Some(temp_part) = output_str.strip_prefix("temp=").and_then(|s| s.strip_suffix("'C\n")) {
                    if let Ok(temp) = temp_part.parse::<f32>() {
                        gpu_celsius = Some(temp);
                        thermal_zones.insert("gpu".to_string(), temp);
                    }
                }
            }
        }
        
        // Check for thermal throttling (Raspberry Pi specific)
        if let Ok(output) = std::process::Command::new("vcgencmd").arg("get_throttled").output() {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                if let Some(throttled_hex) = output_str.strip_prefix("throttled=0x") {
                    if let Ok(throttled_value) = u32::from_str_radix(throttled_hex.trim(), 16) {
                        // Bit 0: under-voltage detected
                        // Bit 1: arm frequency capped
                        // Bit 2: currently throttled
                        // Bit 3: soft temperature limit active
                        is_throttling = (throttled_value & 0x000E) != 0;
                    }
                }
            }
        }
        
        // Read additional thermal zones
        for i in 1..10 {
            let path = format!("/sys/class/thermal/thermal_zone{}/temp", i);
            if let Ok(temp_str) = fs::read_to_string(&path) {
                if let Ok(temp_millicelsius) = temp_str.trim().parse::<i32>() {
                    let temp_celsius = temp_millicelsius as f32 / 1000.0;
                    thermal_zones.insert(format!("zone{}", i), temp_celsius);
                }
            }
        }
        
        Ok(TemperatureInfo {
            cpu_celsius,
            gpu_celsius,
            thermal_zones,
            is_throttling,
        })
    }
    
    /// Collect general system information.
    fn collect_system_info(&self) -> Result<SystemInfo> {
        let hostname = System::host_name().unwrap_or_else(|| "unknown".to_string());
        let os_name = System::name().unwrap_or_else(|| "unknown".to_string());
        let os_version = System::os_version().unwrap_or_else(|| "unknown".to_string());
        let kernel_version = System::kernel_version().unwrap_or_else(|| "unknown".to_string());
        let uptime_seconds = System::uptime();
        let boot_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() - uptime_seconds;
        let process_count = self.system.processes().len() as u64;
        
        Ok(SystemInfo {
            hostname,
            os_name,
            os_version,
            kernel_version,
            uptime_seconds,
            boot_time,
            process_count,
        })
    }
    
    /// Collect GPIO information if available.
    #[cfg(feature = "gpio")]
    fn collect_gpio_info(&mut self) -> crate::metrics::gpio::GpioStatus {
        if let Some(ref mut provider) = self.gpio_provider {
            provider.read_gpio_status().unwrap_or_default()
        } else {
            crate::metrics::gpio::GpioStatus::default()
        }
    }
}

impl MetricsProvider for SystemCollector {
    async fn collect_snapshot(&mut self) -> Result<SystemSnapshot> {
        self.refresh();
        
        let mut snapshot = SystemSnapshot::new();
        snapshot.cpu = self.collect_cpu_info()?;
        snapshot.memory = self.collect_memory_info()?;
        snapshot.storage = self.collect_storage_info();
        snapshot.network = self.collect_network_info();
        snapshot.temperature = self.collect_temperature_info()?;
        snapshot.system = self.collect_system_info()?;
        
        #[cfg(feature = "gpio")]
        {
            snapshot.gpio = self.collect_gpio_info();
        }
        
        Ok(snapshot)
    }
    
    async fn start_stream(&mut self, interval_ms: u64) -> Result<BoxStream<'static, SystemSnapshot>> {
        let interval = Duration::from_millis(interval_ms);
        let collector = SystemCollector::new()?;
        
        let stream = stream::unfold(
            (collector, time::interval(interval)),
            |(mut collector, mut interval)| async move {
                interval.tick().await;
                match collector.collect_snapshot().await {
                    Ok(snapshot) => Some((snapshot, (collector, interval))),
                    Err(err) => {
                        tracing::error!("Failed to collect system snapshot: {}", err);
                        None
                    }
                }
            },
        );
        
        Ok(Box::pin(stream))
    }
}

impl SystemMonitor for SystemCollector {
    fn new() -> Result<Self> {
        SystemCollector::new()
    }
    
    async fn start_collecting(&mut self) -> Result<BoxStream<'static, SystemSnapshot>> {
        self.start_stream(crate::DEFAULT_INTERVAL_MS).await
    }
    
    async fn start_collecting_with_interval(&mut self, interval_ms: u64) -> Result<BoxStream<'static, SystemSnapshot>> {
        self.start_stream(interval_ms).await
    }
    
    async fn get_snapshot(&mut self) -> Result<SystemSnapshot> {
        self.collect_snapshot().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::StreamExt;
    
    #[tokio::test]
    async fn test_system_collector_creation() {
        let collector = SystemCollector::new();
        assert!(collector.is_ok());
    }
    
    #[tokio::test]
    async fn test_snapshot_collection() {
        let mut collector = SystemCollector::new().unwrap();
        let snapshot = collector.collect_snapshot().await;
        assert!(snapshot.is_ok());
        
        let snapshot = snapshot.unwrap();
        assert!(snapshot.timestamp > 0);
        assert!(!snapshot.cpu.model.is_empty());
        assert!(snapshot.cpu.cores > 0);
    }
    
    #[tokio::test]
    async fn test_stream_collection() {
        let mut collector = SystemCollector::new().unwrap();
        let mut stream = collector.start_stream(100).await.unwrap();
        
        // Collect first snapshot
        if let Some(snapshot) = stream.next().await {
            assert!(snapshot.timestamp > 0);
        }
    }
    
    #[test]
    fn test_load_average_parsing() {
        // This would require mocking /proc/loadavg, but we can test the structure
        let load_avg = LoadAverage::default();
        assert_eq!(load_avg.one_minute, 0.0);
        assert_eq!(load_avg.five_minutes, 0.0);
        assert_eq!(load_avg.fifteen_minutes, 0.0);
    }
}
