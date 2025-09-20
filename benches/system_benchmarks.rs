use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use life_of_pi::{
    SystemCollector, SystemMonitor,
    metrics::data::SystemSnapshot,
};
use serde_json;
use std::time::Duration;

/// Benchmark system snapshot collection
fn bench_snapshot_collection(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().expect("Should create tokio runtime");
    
    c.bench_function("system_snapshot_collection", |b| {
        b.to_async(&rt).iter(|| async {
            let mut collector = SystemCollector::new().expect("Should create collector");
            collector.get_snapshot().await.expect("Should collect snapshot")
        })
    });
}

/// Benchmark JSON serialization of system snapshots
fn bench_json_serialization(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().expect("Should create tokio runtime");
    
    // Collect a real snapshot to benchmark serialization
    let snapshot = rt.block_on(async {
        let mut collector = SystemCollector::new().expect("Should create collector");
        collector.get_snapshot().await.expect("Should collect snapshot")
    });
    
    c.bench_function("json_serialization", |b| {
        b.iter(|| {
            serde_json::to_string(&snapshot).expect("Should serialize")
        })
    });
    
    c.bench_function("json_pretty_serialization", |b| {
        b.iter(|| {
            serde_json::to_string_pretty(&snapshot).expect("Should serialize pretty")
        })
    });
}

/// Benchmark JSON deserialization
fn bench_json_deserialization(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().expect("Should create tokio runtime");
    
    // Create a JSON string to deserialize
    let json_string = rt.block_on(async {
        let mut collector = SystemCollector::new().expect("Should create collector");
        let snapshot = collector.get_snapshot().await.expect("Should collect snapshot");
        serde_json::to_string(&snapshot).expect("Should serialize")
    });
    
    c.bench_function("json_deserialization", |b| {
        b.iter(|| {
            serde_json::from_str::<SystemSnapshot>(&json_string).expect("Should deserialize")
        })
    });
}

/// Benchmark concurrent snapshot collection
fn bench_concurrent_collection(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().expect("Should create tokio runtime");
    
    for concurrency in [1, 2, 4, 8].iter() {
        c.bench_with_input(
            BenchmarkId::new("concurrent_collection", concurrency),
            concurrency,
            |b, &concurrency| {
                b.to_async(&rt).iter(|| async move {
                    let mut handles = Vec::new();
                    
                    for _ in 0..concurrency {
                        let handle = tokio::spawn(async move {
                            let mut collector = SystemCollector::new().expect("Should create collector");
                            collector.get_snapshot().await.expect("Should collect snapshot")
                        });
                        handles.push(handle);
                    }
                    
                    futures_util::future::join_all(handles).await
                })
            }
        );
    }
}

/// Benchmark memory allocation during collection
fn bench_memory_overhead(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().expect("Should create tokio runtime");
    
    c.bench_function("repeated_collection_memory", |b| {
        b.to_async(&rt).iter(|| async {
            let mut collector = SystemCollector::new().expect("Should create collector");
            
            // Collect multiple snapshots to test memory usage
            for _ in 0..10 {
                let _snapshot = collector.get_snapshot().await.expect("Should collect snapshot");
            }
        })
    });
}

/// Benchmark snapshot data structure cloning
fn bench_snapshot_clone(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().expect("Should create tokio runtime");
    
    let snapshot = rt.block_on(async {
        let mut collector = SystemCollector::new().expect("Should create collector");
        collector.get_snapshot().await.expect("Should collect snapshot")
    });
    
    c.bench_function("snapshot_clone", |b| {
        b.iter(|| {
            snapshot.clone()
        })
    });
}

/// Benchmark WebSocket message preparation
fn bench_websocket_message_prep(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().expect("Should create tokio runtime");
    
    let snapshot = rt.block_on(async {
        let mut collector = SystemCollector::new().expect("Should create collector");
        collector.get_snapshot().await.expect("Should collect snapshot")
    });
    
    c.bench_function("websocket_message_prep", |b| {
        b.iter(|| {
            // Simulate preparing a WebSocket message
            let json = serde_json::to_string(&snapshot).expect("Should serialize");
            json.into_bytes()
        })
    });
}

/// Benchmark system collector initialization
fn bench_collector_init(c: &mut Criterion) {
    c.bench_function("collector_initialization", |b| {
        b.iter(|| {
            SystemCollector::new().expect("Should create collector")
        })
    });
}

/// Benchmark different collection intervals
fn bench_collection_intervals(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().expect("Should create tokio runtime");
    
    for interval_ms in [100, 250, 500, 1000].iter() {
        c.bench_with_input(
            BenchmarkId::new("collection_with_interval", interval_ms),
            interval_ms,
            |b, &interval_ms| {
                b.to_async(&rt).iter(|| async move {
                    let mut collector = SystemCollector::new().expect("Should create collector");
                    
                    // Simulate collecting a few snapshots with the given interval
                    for _ in 0..3 {
                        let _snapshot = collector.get_snapshot().await.expect("Should collect snapshot");
                        tokio::time::sleep(Duration::from_millis(interval_ms / 10)).await; // Shorter sleep for benchmarking
                    }
                })
            }
        );
    }
}

/// Benchmark temperature parsing (Raspberry Pi specific)
fn bench_temperature_parsing(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().expect("Should create tokio runtime");
    
    c.bench_function("temperature_data_processing", |b| {
        b.to_async(&rt).iter(|| async {
            let mut collector = SystemCollector::new().expect("Should create collector");
            let snapshot = collector.get_snapshot().await.expect("Should collect snapshot");
            
            // Access temperature data to trigger any lazy evaluation
            let _cpu_temp = snapshot.temperature.cpu_celsius;
            let _gpu_temp = snapshot.temperature.gpu_celsius;
            let _throttling = snapshot.temperature.is_throttling;
            let _zones = &snapshot.temperature.thermal_zones;
        })
    });
}

/// Benchmark CPU metrics collection
fn bench_cpu_metrics(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().expect("Should create tokio runtime");
    
    c.bench_function("cpu_metrics_collection", |b| {
        b.to_async(&rt).iter(|| async {
            let mut collector = SystemCollector::new().expect("Should create collector");
            let snapshot = collector.get_snapshot().await.expect("Should collect snapshot");
            
            // Access CPU-specific data
            let _model = &snapshot.cpu.model;
            let _cores = snapshot.cpu.cores;
            let _usage = snapshot.cpu.usage_percent;
            let _core_usage = &snapshot.cpu.core_usage;
            let _load_avg = &snapshot.cpu.load_average;
        })
    });
}

criterion_group!(
    benches,
    bench_snapshot_collection,
    bench_json_serialization,
    bench_json_deserialization,
    bench_concurrent_collection,
    bench_memory_overhead,
    bench_snapshot_clone,
    bench_websocket_message_prep,
    bench_collector_init,
    bench_collection_intervals,
    bench_temperature_parsing,
    bench_cpu_metrics
);

criterion_main!(benches);