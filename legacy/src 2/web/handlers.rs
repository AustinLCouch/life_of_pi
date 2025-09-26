//! HTTP handlers for API endpoints.

use crate::metrics::{MetricsProvider, SystemCollector};
use axum::{
    http::StatusCode,
    response::{Html, Json},
};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::error;

// Global state for the system collector
// In a real application, this would be passed via app state
lazy_static::lazy_static! {
    static ref COLLECTOR: Arc<Mutex<SystemCollector>> = {
        match SystemCollector::new() {
            Ok(collector) => Arc::new(Mutex::new(collector)),
            Err(e) => {
                panic!("Failed to initialize system collector: {}", e);
            }
        }
    };
}

/// Get current system snapshot as JSON.
pub async fn get_snapshot() -> Result<Json<serde_json::Value>, StatusCode> {
    let mut collector = COLLECTOR.lock().await;

    match collector.collect_snapshot().await {
        Ok(snapshot) => match serde_json::to_value(&snapshot) {
            Ok(json_value) => Ok(Json(json_value)),
            Err(e) => {
                error!("Failed to serialize snapshot: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        },
        Err(e) => {
            error!("Failed to collect snapshot: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Health check endpoint.
pub async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "ok",
        "service": "life-of-pi",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// Serve the main dashboard HTML page from static files.
pub async fn serve_index() -> Result<Html<String>, StatusCode> {
    match tokio::fs::read_to_string("static/index.html").await {
        Ok(content) => Ok(Html(content)),
        Err(e) => {
            error!("Failed to read index.html: {}", e);
            Err(StatusCode::NOT_FOUND)
        }
    }
}

/// Serve a default dashboard HTML page when no static files are available.
pub async fn default_index() -> Html<&'static str> {
    Html(DEFAULT_INDEX_HTML)
}

/// Default HTML content when no static files are provided.
const DEFAULT_INDEX_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Life of Pi - System Monitor</title>
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }
        
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: #333;
            min-height: 100vh;
            padding: 20px;
        }
        
        .container {
            max-width: 1200px;
            margin: 0 auto;
        }
        
        .header {
            text-align: center;
            margin-bottom: 40px;
            color: white;
        }
        
        .header h1 {
            font-size: 3rem;
            margin-bottom: 10px;
            text-shadow: 2px 2px 4px rgba(0,0,0,0.3);
        }
        
        .header p {
            font-size: 1.2rem;
            opacity: 0.9;
        }
        
        .dashboard {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
            gap: 20px;
            margin-bottom: 40px;
        }
        
        .card {
            background: white;
            border-radius: 15px;
            padding: 25px;
            box-shadow: 0 10px 30px rgba(0,0,0,0.1);
            transition: transform 0.3s ease;
        }
        
        .card:hover {
            transform: translateY(-5px);
        }
        
        .card h3 {
            color: #667eea;
            margin-bottom: 15px;
            font-size: 1.5rem;
        }
        
        .metric {
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 10px;
            padding: 10px 0;
            border-bottom: 1px solid #eee;
        }
        
        .metric:last-child {
            border-bottom: none;
            margin-bottom: 0;
        }
        
        .metric-label {
            font-weight: 600;
            color: #666;
        }
        
        .metric-value {
            font-weight: bold;
            color: #333;
        }
        
        .status {
            text-align: center;
            color: white;
            padding: 20px;
            background: rgba(255,255,255,0.1);
            border-radius: 10px;
            backdrop-filter: blur(10px);
        }
        
        .loading {
            display: inline-block;
            width: 20px;
            height: 20px;
            border: 3px solid rgba(255,255,255,0.3);
            border-radius: 50%;
            border-top: 3px solid white;
            animation: spin 1s linear infinite;
            margin-right: 10px;
        }
        
        @keyframes spin {
            0% { transform: rotate(0deg); }
            100% { transform: rotate(360deg); }
        }
        
        .error {
            color: #ff6b6b;
            background: rgba(255,107,107,0.1);
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>🥧 Life of Pi</h1>
            <p>Raspberry Pi System Monitor</p>
        </div>
        
        <div class="dashboard" id="dashboard">
            <div class="card">
                <h3>CPU</h3>
                <div class="metric">
                    <span class="metric-label">Usage</span>
                    <span class="metric-value" id="cpu-usage">Loading...</span>
                </div>
                <div class="metric">
                    <span class="metric-label">Temperature</span>
                    <span class="metric-value" id="cpu-temp">Loading...</span>
                </div>
                <div class="metric">
                    <span class="metric-label">Frequency</span>
                    <span class="metric-value" id="cpu-freq">Loading...</span>
                </div>
            </div>
            
            <div class="card">
                <h3>Memory</h3>
                <div class="metric">
                    <span class="metric-label">Usage</span>
                    <span class="metric-value" id="mem-usage">Loading...</span>
                </div>
                <div class="metric">
                    <span class="metric-label">Total</span>
                    <span class="metric-value" id="mem-total">Loading...</span>
                </div>
                <div class="metric">
                    <span class="metric-label">Available</span>
                    <span class="metric-value" id="mem-available">Loading...</span>
                </div>
            </div>
            
            <div class="card">
                <h3>System</h3>
                <div class="metric">
                    <span class="metric-label">Hostname</span>
                    <span class="metric-value" id="hostname">Loading...</span>
                </div>
                <div class="metric">
                    <span class="metric-label">Uptime</span>
                    <span class="metric-value" id="uptime">Loading...</span>
                </div>
                <div class="metric">
                    <span class="metric-label">Load Average</span>
                    <span class="metric-value" id="load-avg">Loading...</span>
                </div>
            </div>
            
            <div class="card">
                <h3>Network</h3>
                <div id="network-interfaces">
                    <div class="metric">
                        <span class="metric-label">Interfaces</span>
                        <span class="metric-value">Loading...</span>
                    </div>
                </div>
            </div>
        </div>
        
        <div class="status" id="status">
            <div class="loading"></div>
            Connecting to system monitor...
        </div>
    </div>
    
    <script>
        // WebSocket connection for real-time updates
        let ws;
        let reconnectAttempts = 0;
        const maxReconnectAttempts = 5;
        
        function connectWebSocket() {
            const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
            const wsUrl = `${protocol}//${window.location.host}/ws`;
            
            ws = new WebSocket(wsUrl);
            
            ws.onopen = function() {
                console.log('Connected to Life of Pi monitor');
                document.getElementById('status').innerHTML = '🟢 Connected to system monitor';
                document.getElementById('status').className = 'status';
                reconnectAttempts = 0;
            };
            
            ws.onmessage = function(event) {
                try {
                    const data = JSON.parse(event.data);
                    updateDashboard(data);
                } catch (e) {
                    console.error('Failed to parse WebSocket message:', e);
                }
            };
            
            ws.onclose = function() {
                console.log('Disconnected from Life of Pi monitor');
                document.getElementById('status').innerHTML = '🔴 Disconnected from system monitor';
                document.getElementById('status').className = 'status error';
                
                // Attempt to reconnect
                if (reconnectAttempts < maxReconnectAttempts) {
                    reconnectAttempts++;
                    setTimeout(connectWebSocket, 2000 * reconnectAttempts);
                }
            };
            
            ws.onerror = function(error) {
                console.error('WebSocket error:', error);
            };
        }
        
        function updateDashboard(data) {
            // Update CPU metrics
            if (data.cpu) {
                document.getElementById('cpu-usage').textContent = `${data.cpu.usage_percent.toFixed(1)}%`;
                document.getElementById('cpu-freq').textContent = `${data.cpu.frequency_mhz} MHz`;
            }
            
            // Update temperature
            if (data.temperature && data.temperature.cpu_celsius) {
                document.getElementById('cpu-temp').textContent = `${data.temperature.cpu_celsius.toFixed(1)}°C`;
            }
            
            // Update memory metrics
            if (data.memory) {
                document.getElementById('mem-usage').textContent = `${data.memory.usage_percent.toFixed(1)}%`;
                document.getElementById('mem-total').textContent = formatBytes(data.memory.total_bytes);
                document.getElementById('mem-available').textContent = formatBytes(data.memory.available_bytes);
            }
            
            // Update system info
            if (data.system) {
                document.getElementById('hostname').textContent = data.system.hostname;
                document.getElementById('uptime').textContent = formatUptime(data.system.uptime_seconds);
                
                if (data.cpu && data.cpu.load_average) {
                    const load = data.cpu.load_average;
                    document.getElementById('load-avg').textContent = 
                        `${load.one_minute.toFixed(2)}, ${load.five_minutes.toFixed(2)}, ${load.fifteen_minutes.toFixed(2)}`;
                }
            }
            
            // Update network interfaces
            if (data.network) {
                const networkDiv = document.getElementById('network-interfaces');
                networkDiv.innerHTML = '';
                
                data.network.forEach(iface => {
                    const metric = document.createElement('div');
                    metric.className = 'metric';
                    metric.innerHTML = `
                        <span class="metric-label">${iface.interface}</span>
                        <span class="metric-value">${iface.is_up ? '🟢 UP' : '🔴 DOWN'}</span>
                    `;
                    networkDiv.appendChild(metric);
                });
            }
        }
        
        function formatBytes(bytes) {
            const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
            if (bytes === 0) return '0 B';
            const i = Math.floor(Math.log(bytes) / Math.log(1024));
            return `${(bytes / Math.pow(1024, i)).toFixed(1)} ${sizes[i]}`;
        }
        
        function formatUptime(seconds) {
            const days = Math.floor(seconds / 86400);
            const hours = Math.floor((seconds % 86400) / 3600);
            const minutes = Math.floor((seconds % 3600) / 60);
            
            if (days > 0) {
                return `${days}d ${hours}h ${minutes}m`;
            } else if (hours > 0) {
                return `${hours}h ${minutes}m`;
            } else {
                return `${minutes}m`;
            }
        }
        
        // Start the WebSocket connection
        connectWebSocket();
        
        // Also fetch initial data via REST API
        fetch('/api/snapshot')
            .then(response => response.json())
            .then(data => updateDashboard(data))
            .catch(error => console.error('Failed to fetch initial data:', error));
    </script>
</body>
</html>"#;
