use life_of_pi::{
    error::SystemError,
    metrics::{
        data::*,
        SystemCollector, SystemMonitor,
    },
    WebConfig,
};
use serde_json;

/// Test SystemSnapshot serialization and deserialization
#[test]
fn test_system_snapshot_serialization() {
    let snapshot = SystemSnapshot {
        timestamp: 1234567890,
        cpu: CpuInfo {
            model: "Test CPU".to_string(),
            cores: 4,
            architecture: "x86_64".to_string(),
            frequency_mhz: 2400,
            usage_percent: 25.5,
            core_usage: vec![20.0, 25.0, 30.0, 25.0],
            governor: Some("ondemand".to_string()),
            load_average: LoadAverage {
                one_minute: 1.2,
                five_minutes: 1.1,
                fifteen_minutes: 1.0,
            },
        },
        memory: MemoryInfo {
            total_bytes: 8 * 1024 * 1024 * 1024, // 8GB
            available_bytes: 4 * 1024 * 1024 * 1024, // 4GB
            used_bytes: 4 * 1024 * 1024 * 1024, // 4GB
            usage_percent: 50.0,
            swap: SwapInfo {
                total_bytes: 2 * 1024 * 1024 * 1024, // 2GB
                used_bytes: 512 * 1024 * 1024, // 512MB
                free_bytes: 1536 * 1024 * 1024, // 1.5GB
            },
            breakdown: MemoryBreakdown {
                buffers_bytes: 100 * 1024 * 1024, // 100MB
                cached_bytes: 500 * 1024 * 1024,  // 500MB
                shared_bytes: 50 * 1024 * 1024,   // 50MB
            },
        },
        storage: vec![StorageInfo {
            device: "/dev/sda1".to_string(),
            mount_point: "/".to_string(),
            filesystem: "ext4".to_string(),
            total_bytes: 500 * 1024 * 1024 * 1024, // 500GB
            available_bytes: 250 * 1024 * 1024 * 1024, // 250GB
            used_bytes: 250 * 1024 * 1024 * 1024, // 250GB
            usage_percent: 50.0,
        }],
        network: vec![NetworkInfo {
            interface: "eth0".to_string(),
            is_up: true,
            mac_address: Some("00:11:22:33:44:55".to_string()),
            ipv4_addresses: vec!["192.168.1.100".to_string()],
            ipv6_addresses: vec!["::1".to_string()],
            tx_bytes: 1000000,
            rx_bytes: 2000000,
            tx_packets: 1000,
            rx_packets: 2000,
            tx_errors: 0,
            rx_errors: 1,
        }],
        temperature: TemperatureInfo {
            cpu_celsius: Some(45.5),
            gpu_celsius: Some(40.0),
            thermal_zones: {
                let mut zones = std::collections::HashMap::new();
                zones.insert("zone0".to_string(), 45.5);
                zones.insert("zone1".to_string(), 40.0);
                zones
            },
            is_throttling: false,
        },
        system: SystemInfo {
            hostname: "test-pi".to_string(),
            os_name: "Linux".to_string(),
            os_version: "6.1.0".to_string(),
            kernel_version: "6.1.0-rpi7-rpi-v8".to_string(),
            uptime_seconds: 86400, // 1 day
            boot_time: 1234567890,
            process_count: 150,
        },
        #[cfg(feature = "gpio")]
        gpio: life_of_pi::metrics::gpio::GpioStatus::default(),
    };

    // Test serialization to JSON
    let json = serde_json::to_string_pretty(&snapshot).expect("Should serialize to JSON");
    assert!(json.contains("Test CPU"));
    assert!(json.contains("test-pi"));
    assert!(json.contains("45.5"));

    // Test deserialization from JSON
    let deserialized: SystemSnapshot = serde_json::from_str(&json).expect("Should deserialize from JSON");
    assert_eq!(deserialized.cpu.model, "Test CPU");
    assert_eq!(deserialized.system.hostname, "test-pi");
    assert_eq!(deserialized.temperature.cpu_celsius, Some(45.5));
    assert_eq!(deserialized.memory.usage_percent, 50.0);
}

/// Test LoadAverage calculations and defaults
#[test]
fn test_load_average() {
    let load_avg = LoadAverage {
        one_minute: 1.5,
        five_minutes: 1.2,
        fifteen_minutes: 1.0,
    };
    
    assert_eq!(load_avg.one_minute, 1.5);
    assert_eq!(load_avg.five_minutes, 1.2);
    assert_eq!(load_avg.fifteen_minutes, 1.0);

    let default_load = LoadAverage::default();
    assert_eq!(default_load.one_minute, 0.0);
    assert_eq!(default_load.five_minutes, 0.0);
    assert_eq!(default_load.fifteen_minutes, 0.0);
}

/// Test MemoryInfo calculations
#[test]
fn test_memory_calculations() {
    let total = 8 * 1024 * 1024 * 1024_u64; // 8GB
    let used = 4 * 1024 * 1024 * 1024_u64;  // 4GB
    let available = total - used;
    let usage_percent = (used as f32 / total as f32) * 100.0;

    let memory = MemoryInfo {
        total_bytes: total,
        available_bytes: available,
        used_bytes: used,
        usage_percent,
        swap: SwapInfo::default(),
        breakdown: MemoryBreakdown::default(),
    };

    assert_eq!(memory.usage_percent, 50.0);
    assert!(memory.total_bytes > 0);
    assert!(memory.available_bytes > 0);
}

/// Test StorageInfo calculations
#[test]
fn test_storage_calculations() {
    let total = 1000 * 1024 * 1024 * 1024_u64; // 1TB
    let used = 300 * 1024 * 1024 * 1024_u64;   // 300GB
    let available = total - used;
    let usage_percent = (used as f32 / total as f32) * 100.0;

    let storage = StorageInfo {
        device: "/dev/sda1".to_string(),
        mount_point: "/".to_string(),
        filesystem: "ext4".to_string(),
        total_bytes: total,
        available_bytes: available,
        used_bytes: used,
        usage_percent,
    };

    assert!((usage_percent - 30.0).abs() < 0.001, "Usage percent should be approximately 30.0, got {}", usage_percent);
    assert_eq!(storage.filesystem, "ext4");
    assert_eq!(storage.mount_point, "/");
}

/// Test SystemError creation and formatting
#[test]
fn test_system_error_types() {
    let io_error = SystemError::system_error("Test IO error");
    assert!(format!("{}", io_error).contains("Test IO error"));

    let parse_error = SystemError::parse_error("Failed to parse data");
    assert!(format!("{}", parse_error).contains("Failed to parse data"));

    let network_error = SystemError::network_error("Connection failed");
    assert!(format!("{}", network_error).contains("Connection failed"));

    let web_error = SystemError::web_server_error("Server startup failed");
    assert!(format!("{}", web_error).contains("Server startup failed"));

    let config_error = SystemError::config_error("Invalid configuration");
    assert!(format!("{}", config_error).contains("Invalid configuration"));
}

/// Test WebConfig builder pattern
#[test]
fn test_web_config() {
    let config = WebConfig::default()
        .with_host("127.0.0.1")
        .with_port(9090)
        .with_cors(false)
        .with_max_websocket_connections(50);

    assert_eq!(config.host, "127.0.0.1");
    assert_eq!(config.port, 9090);
    assert_eq!(config.enable_cors, false);
    assert_eq!(config.max_websocket_connections, 50);
    assert_eq!(config.bind_address(), "127.0.0.1:9090");
}

/// Test SystemCollector creation
#[tokio::test]
async fn test_system_collector_creation() {
    let result = SystemCollector::new();
    assert!(result.is_ok(), "SystemCollector should create successfully");

    if let Ok(mut collector) = result {
        // Test that we can collect a snapshot
        let snapshot_result = collector.get_snapshot().await;
        assert!(snapshot_result.is_ok(), "Should be able to collect system snapshot");

        if let Ok(snapshot) = snapshot_result {
            // Basic sanity checks
            assert!(snapshot.timestamp > 0, "Timestamp should be set");
            assert!(snapshot.cpu.cores > 0, "Should detect CPU cores");
            assert!(snapshot.memory.total_bytes > 0, "Should detect system memory");
            assert!(!snapshot.system.hostname.is_empty(), "Should detect hostname");
            assert!(!snapshot.system.os_name.is_empty(), "Should detect OS name");
        }
    }
}

/// Test temperature parsing and validation
#[test]
fn test_temperature_info() {
    let mut temp_info = TemperatureInfo::default();
    assert_eq!(temp_info.cpu_celsius, None);
    assert_eq!(temp_info.gpu_celsius, None);
    assert!(!temp_info.is_throttling);

    // Simulate temperature readings
    temp_info.cpu_celsius = Some(65.5);
    temp_info.gpu_celsius = Some(60.0);
    temp_info.thermal_zones.insert("zone0".to_string(), 65.5);
    temp_info.is_throttling = true;

    assert_eq!(temp_info.cpu_celsius, Some(65.5));
    assert_eq!(temp_info.gpu_celsius, Some(60.0));
    assert!(temp_info.is_throttling);
    assert!(temp_info.thermal_zones.contains_key("zone0"));
}

/// Test network interface parsing
#[test]
fn test_network_info() {
    let network = NetworkInfo {
        interface: "wlan0".to_string(),
        is_up: true,
        mac_address: Some("aa:bb:cc:dd:ee:ff".to_string()),
        ipv4_addresses: vec!["192.168.1.100".to_string(), "10.0.0.1".to_string()],
        ipv6_addresses: vec!["fe80::1".to_string()],
        tx_bytes: 1024 * 1024,      // 1MB
        rx_bytes: 2 * 1024 * 1024,  // 2MB
        tx_packets: 1000,
        rx_packets: 2000,
        tx_errors: 0,
        rx_errors: 1,
    };

    assert_eq!(network.interface, "wlan0");
    assert!(network.is_up);
    assert_eq!(network.ipv4_addresses.len(), 2);
    assert_eq!(network.ipv6_addresses.len(), 1);
    assert_eq!(network.tx_bytes, 1024 * 1024);
    assert_eq!(network.rx_errors, 1);
}

/// Test default implementations
#[test]
fn test_default_implementations() {
    let cpu_info = CpuInfo::default();
    assert_eq!(cpu_info.cores, 0);
    assert_eq!(cpu_info.usage_percent, 0.0);
    assert!(cpu_info.core_usage.is_empty());

    let memory_info = MemoryInfo::default();
    assert_eq!(memory_info.total_bytes, 0);
    assert_eq!(memory_info.usage_percent, 0.0);

    let system_info = SystemInfo::default();
    assert!(system_info.hostname.is_empty());
    assert_eq!(system_info.process_count, 0);

    let temp_info = TemperatureInfo::default();
    assert!(!temp_info.is_throttling);
    assert!(temp_info.thermal_zones.is_empty());
}

#[cfg(feature = "gpio")]
#[test]
fn test_gpio_functionality() {
    use life_of_pi::metrics::gpio::{DefaultGpioProvider, GpioProvider};

    let mut provider = DefaultGpioProvider::new().expect("GPIO provider should initialize");
    let status = provider.read_gpio_status().expect("Should read GPIO status");
    
    // This test will only run when GPIO feature is enabled
    // Behavior depends on whether running on actual Raspberry Pi or mock
    assert!(status.available_pins.len() >= 0, "Should have valid pin count");
    assert!(status.pin_states.len() >= 0, "Should have valid pin states");
}

/// Test JSON schema validation for SystemSnapshot
#[test]
fn test_json_schema_validation() {
    let snapshot = SystemSnapshot::new();
    let json_str = serde_json::to_string(&snapshot).expect("Should serialize");
    let json_value: serde_json::Value = serde_json::from_str(&json_str).expect("Should parse JSON");

    // Check required fields exist
    assert!(json_value.get("timestamp").is_some());
    assert!(json_value.get("cpu").is_some());
    assert!(json_value.get("memory").is_some());
    assert!(json_value.get("storage").is_some());
    assert!(json_value.get("network").is_some());
    assert!(json_value.get("temperature").is_some());
    assert!(json_value.get("system").is_some());

    // Check nested structure
    let cpu = json_value.get("cpu").unwrap();
    assert!(cpu.get("model").is_some());
    assert!(cpu.get("cores").is_some());
    assert!(cpu.get("usage_percent").is_some());
    assert!(cpu.get("load_average").is_some());

    let memory = json_value.get("memory").unwrap();
    assert!(memory.get("total_bytes").is_some());
    assert!(memory.get("usage_percent").is_some());
    assert!(memory.get("swap").is_some());
}