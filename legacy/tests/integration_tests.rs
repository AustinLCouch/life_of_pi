use life_of_pi::{metrics::data::SystemSnapshot, SystemCollector, SystemMonitor};
use std::time::Duration;
use tokio::time::timeout;

#[tokio::test]
async fn test_system_collector_basic_functionality() {
    let mut collector = SystemCollector::new().expect("Should create SystemCollector");

    let snapshot = collector
        .get_snapshot()
        .await
        .expect("Should collect system snapshot");

    // Basic sanity checks
    assert!(snapshot.cpu.cores > 0);
    assert!(snapshot.memory.total_bytes > 0);
    assert!(!snapshot.system.hostname.is_empty());
    assert!(!snapshot.system.os_name.is_empty());
}

#[tokio::test]
async fn test_snapshot_serialization_roundtrip() {
    let mut collector = SystemCollector::new().expect("Should create SystemCollector");

    let original_snapshot = collector
        .get_snapshot()
        .await
        .expect("Should collect system snapshot");

    // Serialize to JSON
    let json_string =
        serde_json::to_string(&original_snapshot).expect("Should serialize snapshot to JSON");

    // Deserialize back
    let deserialized_snapshot: SystemSnapshot =
        serde_json::from_str(&json_string).expect("Should deserialize snapshot from JSON");

    // Verify key fields match
    assert_eq!(original_snapshot.cpu.cores, deserialized_snapshot.cpu.cores);
    assert_eq!(
        original_snapshot.memory.total_bytes,
        deserialized_snapshot.memory.total_bytes
    );
    assert_eq!(
        original_snapshot.system.hostname,
        deserialized_snapshot.system.hostname
    );
    assert_eq!(
        original_snapshot.system.os_name,
        deserialized_snapshot.system.os_name
    );
}

#[tokio::test]
async fn test_multiple_snapshot_collections() {
    let mut collector = SystemCollector::new().expect("Should create SystemCollector");

    let mut snapshots = Vec::new();

    // Collect multiple snapshots
    for _ in 0..5 {
        let snapshot = collector
            .get_snapshot()
            .await
            .expect("Should collect system snapshot");
        snapshots.push(snapshot);

        // Small delay between collections
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    assert_eq!(snapshots.len(), 5);

    // All snapshots should have consistent system information
    let first = &snapshots[0];
    for snapshot in &snapshots[1..] {
        assert_eq!(first.cpu.cores, snapshot.cpu.cores);
        assert_eq!(first.system.hostname, snapshot.system.hostname);
        assert_eq!(first.system.os_name, snapshot.system.os_name);
    }
}

#[tokio::test]
async fn test_concurrent_snapshot_collection() {
    let num_tasks = 10;
    let mut handles = Vec::new();

    for _ in 0..num_tasks {
        let handle = tokio::spawn(async move {
            let mut collector = SystemCollector::new().expect("Should create SystemCollector");
            collector
                .get_snapshot()
                .await
                .expect("Should collect system snapshot")
        });
        handles.push(handle);
    }

    let results = futures_util::future::join_all(handles).await;

    // All tasks should complete successfully
    assert_eq!(results.len(), num_tasks);
    for result in results {
        let snapshot = result.expect("Task should complete successfully");
        assert!(snapshot.cpu.cores > 0);
        assert!(snapshot.memory.total_bytes > 0);
    }
}

#[tokio::test]
async fn test_snapshot_collection_timeout() {
    let mut collector = SystemCollector::new().expect("Should create SystemCollector");

    // Collection should complete within reasonable time
    let result = timeout(Duration::from_secs(5), collector.get_snapshot()).await;

    assert!(result.is_ok(), "Snapshot collection should not timeout");
    let snapshot = result.unwrap().expect("Should collect snapshot");
    assert!(snapshot.cpu.cores > 0);
}

#[tokio::test]
async fn test_memory_stability_over_time() {
    let mut collector = SystemCollector::new().expect("Should create SystemCollector");

    // Collect snapshots repeatedly to check for memory leaks
    let iterations = 50;
    let mut initial_memory = None;

    for i in 0..iterations {
        let snapshot = collector
            .get_snapshot()
            .await
            .expect("Should collect system snapshot");

        // Record initial memory usage
        if i == 0 {
            initial_memory = Some(snapshot.memory.available_bytes);
        }

        // Verify snapshot is valid
        assert!(snapshot.cpu.cores > 0);
        assert!(snapshot.memory.total_bytes > 0);

        // Small delay to allow any cleanup
        if i % 10 == 0 {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }

    // Memory should remain relatively stable (this is a rough check)
    let final_snapshot = collector
        .get_snapshot()
        .await
        .expect("Should collect final snapshot");
    let initial_mem = initial_memory.unwrap();
    let final_mem = final_snapshot.memory.available_bytes;

    // Memory difference should not be excessive (allowing for normal system variance)
    // Skip this check if memory values are zero (platform might not support detailed memory info)
    if initial_mem > 0 && final_mem > 0 {
        let diff_ratio = if initial_mem > final_mem {
            (initial_mem - final_mem) as f64 / initial_mem as f64
        } else {
            (final_mem - initial_mem) as f64 / initial_mem as f64
        };

        assert!(
            diff_ratio < 0.5,
            "Memory usage should remain relatively stable. Initial: {}, Final: {}, Ratio: {}",
            initial_mem,
            final_mem,
            diff_ratio
        );
    } else {
        // Just verify that the structure is consistent even if values are zero
        println!("Memory values are zero - this may be expected on this platform");
        assert_eq!(
            initial_mem, final_mem,
            "Memory values should at least be consistent"
        );
    }
}

#[tokio::test]
async fn test_temperature_data_consistency() {
    let mut collector = SystemCollector::new().expect("Should create SystemCollector");

    let snapshot = collector
        .get_snapshot()
        .await
        .expect("Should collect system snapshot");

    // Temperature values should be reasonable
    if let Some(cpu_temp) = snapshot.temperature.cpu_celsius {
        assert!(
            cpu_temp > -50.0 && cpu_temp < 150.0,
            "CPU temperature should be reasonable: {}",
            cpu_temp
        );
    }

    if let Some(gpu_temp) = snapshot.temperature.gpu_celsius {
        assert!(
            gpu_temp > -50.0 && gpu_temp < 150.0,
            "GPU temperature should be reasonable: {}",
            gpu_temp
        );
    }

    // Thermal zones should have valid names if present
    for (zone_name, temp) in &snapshot.temperature.thermal_zones {
        assert!(
            !zone_name.is_empty(),
            "Thermal zone name should not be empty"
        );
        assert!(
            *temp > -100.0 && *temp < 200.0,
            "Thermal zone temperature should be reasonable: {}",
            temp
        );
    }
}

#[tokio::test]
async fn test_network_interface_data() {
    let mut collector = SystemCollector::new().expect("Should create SystemCollector");

    let snapshot = collector
        .get_snapshot()
        .await
        .expect("Should collect system snapshot");

    // Should have at least one network interface (loopback)
    assert!(
        !snapshot.network.is_empty(),
        "Should have at least one network interface"
    );

    for interface in &snapshot.network {
        assert!(
            !interface.interface.is_empty(),
            "Interface name should not be empty"
        );
        // Bytes and packet counters are unsigned integers (always non-negative)
        // Just verify they exist by accessing them
        let _ = interface.tx_bytes;
        let _ = interface.rx_bytes;
        let _ = interface.tx_packets;
        let _ = interface.rx_packets;
    }
}

#[tokio::test]
async fn test_disk_usage_data() {
    let mut collector = SystemCollector::new().expect("Should create SystemCollector");

    let snapshot = collector
        .get_snapshot()
        .await
        .expect("Should collect system snapshot");

    // Should have at least one disk/mount point
    assert!(
        !snapshot.storage.is_empty(),
        "Should have at least one disk"
    );

    for disk in &snapshot.storage {
        assert!(
            !disk.device.is_empty(),
            "Disk device name should not be empty"
        );
        assert!(
            !disk.mount_point.is_empty(),
            "Mount point should not be empty"
        );
        assert!(disk.total_bytes > 0, "Total disk space should be positive");
        assert!(
            disk.available_bytes <= disk.total_bytes,
            "Available space should not exceed total"
        );
    }
}

// Process information is tracked in SystemInfo as process_count only

#[tokio::test]
async fn test_snapshot_timestamp_accuracy() {
    let mut collector = SystemCollector::new().expect("Should create SystemCollector");

    let before = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    let snapshot = collector
        .get_snapshot()
        .await
        .expect("Should collect system snapshot");
    let after = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    // Snapshot timestamp should be between before and after
    assert!(
        snapshot.timestamp >= before,
        "Snapshot timestamp should not be in the past"
    );
    assert!(
        snapshot.timestamp <= after,
        "Snapshot timestamp should not be in the future"
    );
}

#[tokio::test]
async fn test_cpu_load_average_validity() {
    let mut collector = SystemCollector::new().expect("Should create SystemCollector");

    let snapshot = collector
        .get_snapshot()
        .await
        .expect("Should collect system snapshot");

    // Load averages should be non-negative
    assert!(
        snapshot.cpu.load_average.one_minute >= 0.0,
        "1-minute load average should be non-negative"
    );
    assert!(
        snapshot.cpu.load_average.five_minutes >= 0.0,
        "5-minute load average should be non-negative"
    );
    assert!(
        snapshot.cpu.load_average.fifteen_minutes >= 0.0,
        "15-minute load average should be non-negative"
    );

    // Per-core usage should be valid percentages
    for (core, usage) in snapshot.cpu.core_usage.iter().enumerate() {
        assert!(
            *usage >= 0.0 && *usage <= 100.0,
            "Core {} usage should be between 0-100%: {}",
            core,
            usage
        );
    }

    // Overall CPU usage should be a valid percentage
    assert!(
        snapshot.cpu.usage_percent >= 0.0 && snapshot.cpu.usage_percent <= 100.0,
        "Overall CPU usage should be between 0-100%: {}",
        snapshot.cpu.usage_percent
    );
}

#[tokio::test]
async fn test_snapshot_performance() {
    let mut collector = SystemCollector::new().expect("Should create SystemCollector");

    let start = std::time::Instant::now();
    let snapshot = collector
        .get_snapshot()
        .await
        .expect("Should collect snapshot");
    let duration = start.elapsed();

    // Collection should be reasonably fast (under 1 second)
    assert!(
        duration < Duration::from_secs(1),
        "Snapshot collection took too long: {:?}",
        duration
    );

    // Verify snapshot has expected data
    assert!(snapshot.cpu.cores > 0);
    assert!(snapshot.memory.total_bytes > 0);
}

#[tokio::test]
async fn test_json_serialization_performance() {
    let mut collector = SystemCollector::new().expect("Should create SystemCollector");

    let snapshot = collector
        .get_snapshot()
        .await
        .expect("Should collect snapshot");

    let start = std::time::Instant::now();
    let json = serde_json::to_string(&snapshot).expect("Should serialize");
    let duration = start.elapsed();

    // Serialization should be fast
    assert!(
        duration < Duration::from_millis(100),
        "JSON serialization took too long: {:?}",
        duration
    );
    assert!(!json.is_empty());
    assert!(json.len() > 100); // Should have substantial content
}

#[tokio::test]
async fn test_data_consistency_across_collections() {
    let mut collector = SystemCollector::new().expect("Should create SystemCollector");

    let snapshot1 = collector
        .get_snapshot()
        .await
        .expect("Should collect first snapshot");

    // Small delay
    tokio::time::sleep(Duration::from_millis(100)).await;

    let snapshot2 = collector
        .get_snapshot()
        .await
        .expect("Should collect second snapshot");

    // Some values should remain consistent
    assert_eq!(
        snapshot1.cpu.cores, snapshot2.cpu.cores,
        "CPU core count should be consistent"
    );
    assert_eq!(
        snapshot1.cpu.model, snapshot2.cpu.model,
        "CPU model should be consistent"
    );
    assert_eq!(
        snapshot1.system.hostname, snapshot2.system.hostname,
        "Hostname should be consistent"
    );

    // Timestamps should be different and in order
    assert!(
        snapshot2.timestamp >= snapshot1.timestamp,
        "Timestamps should be in order"
    );
}

#[tokio::test]
async fn test_error_handling_graceful_degradation() {
    // This test verifies that our error handling works correctly
    let collector_result = SystemCollector::new();
    assert!(
        collector_result.is_ok(),
        "Collector creation should not fail on supported systems"
    );

    // Test that we can handle collection gracefully even if some metrics fail
    if let Ok(mut collector) = collector_result {
        let snapshot_result = collector.get_snapshot().await;
        assert!(
            snapshot_result.is_ok(),
            "Should handle partial metric collection gracefully"
        );
    }
}
