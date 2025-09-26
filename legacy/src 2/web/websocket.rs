//! WebSocket handler for real-time system metrics streaming.

use crate::error::{Result, SystemError};
use crate::metrics::SystemSnapshot;
use axum::extract::ws::{Message, WebSocket};
use axum::{extract::WebSocketUpgrade, response::Response};
use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tracing::{debug, error, info, warn};

// Global broadcast channel for system snapshots
lazy_static::lazy_static! {
    static ref BROADCAST_TX: broadcast::Sender<SystemSnapshot> = {
        let (tx, _rx) = broadcast::channel(100);
        tx
    };

    static ref CONNECTED_CLIENTS: Arc<RwLock<HashMap<String, Client>>> = {
        Arc::new(RwLock::new(HashMap::new()))
    };
}

#[derive(Debug)]
struct Client {
    id: String,
    connected_at: std::time::SystemTime,
}

/// WebSocket upgrade handler.
pub async fn websocket_handler(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(handle_websocket)
}

/// Handle a WebSocket connection.
async fn handle_websocket(socket: WebSocket) {
    let client_id = uuid::Uuid::new_v4().to_string();
    info!("WebSocket client connected: {}", client_id);

    // Add client to connected clients list
    {
        let mut clients = CONNECTED_CLIENTS.write().await;
        clients.insert(
            client_id.clone(),
            Client {
                id: client_id.clone(),
                connected_at: std::time::SystemTime::now(),
            },
        );
    }

    let (mut sender, mut receiver) = socket.split();

    // Subscribe to broadcast channel
    let mut rx = BROADCAST_TX.subscribe();

    // Spawn a task to handle incoming messages from the client
    let client_id_recv = client_id.clone();
    let recv_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    debug!("Received message from {}: {}", client_id_recv, text);
                    // Handle any client messages here (e.g., configuration changes)
                }
                Ok(Message::Binary(_)) => {
                    debug!("Received binary message from {}", client_id_recv);
                }
                Ok(Message::Close(_)) => {
                    info!("WebSocket client {} disconnected", client_id_recv);
                    break;
                }
                Ok(Message::Ping(_)) => {
                    debug!("Received ping from {}", client_id_recv);
                }
                Ok(Message::Pong(_)) => {
                    debug!("Received pong from {}", client_id_recv);
                }
                Err(e) => {
                    warn!("WebSocket error for client {}: {}", client_id_recv, e);
                    break;
                }
            }
        }
    });

    // Spawn a task to send system snapshots to the client
    let client_id_send = client_id.clone();
    let send_task = tokio::spawn(async move {
        while let Ok(snapshot) = rx.recv().await {
            match serde_json::to_string(&snapshot) {
                Ok(json_string) => {
                    if let Err(e) = sender.send(Message::Text(json_string)).await {
                        warn!("Failed to send message to client {}: {}", client_id_send, e);
                        break;
                    }
                }
                Err(e) => {
                    error!(
                        "Failed to serialize snapshot for client {}: {}",
                        client_id_send, e
                    );
                }
            }
        }
    });

    // Wait for either task to complete
    tokio::select! {
        _ = recv_task => {
            debug!("Receive task completed for client {}", client_id);
        }
        _ = send_task => {
            debug!("Send task completed for client {}", client_id);
        }
    }

    // Remove client from connected clients list
    {
        let mut clients = CONNECTED_CLIENTS.write().await;
        clients.remove(&client_id);
    }

    info!("WebSocket client disconnected: {}", client_id);
}

/// Broadcast a system snapshot to all connected WebSocket clients.
pub async fn broadcast_snapshot(snapshot: SystemSnapshot) -> Result<()> {
    let client_count = {
        let clients = CONNECTED_CLIENTS.read().await;
        clients.len()
    };

    if client_count > 0 {
        match BROADCAST_TX.send(snapshot) {
            Ok(receiver_count) => {
                debug!(
                    "Broadcasted snapshot to {} receivers ({} connected clients)",
                    receiver_count, client_count
                );
            }
            Err(e) => {
                warn!("Failed to broadcast snapshot: {}", e);
                return Err(SystemError::web_server_error(format!(
                    "Failed to broadcast snapshot: {}",
                    e
                )));
            }
        }
    }

    Ok(())
}

/// Get the number of connected WebSocket clients.
pub async fn get_connected_client_count() -> usize {
    let clients = CONNECTED_CLIENTS.read().await;
    clients.len()
}

/// Get information about connected WebSocket clients.
pub async fn get_connected_clients() -> Vec<serde_json::Value> {
    let clients = CONNECTED_CLIENTS.read().await;
    let mut client_info = Vec::new();

    for client in clients.values() {
        let connected_duration = client.connected_at.elapsed().unwrap_or_default().as_secs();

        client_info.push(serde_json::json!({
            "id": client.id,
            "connected_at": client.connected_at
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            "connected_duration_seconds": connected_duration
        }));
    }

    client_info
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_broadcast_no_clients() {
        let snapshot = SystemSnapshot::new();
        let result = broadcast_snapshot(snapshot).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_connected_client_count() {
        let count = get_connected_client_count().await;
        assert!(count == 0); // No clients connected in test
    }

    #[tokio::test]
    async fn test_connected_clients_info() {
        let clients = get_connected_clients().await;
        assert!(clients.is_empty()); // No clients connected in test
    }
}
