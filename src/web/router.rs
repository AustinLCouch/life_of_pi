//! Web application router and middleware setup.

use crate::error::Result;
use crate::web::config::WebConfig;
use crate::web::handlers;
use crate::web::websocket;
use axum::{
    routing::{get, get_service},
    Router,
};
use std::path::PathBuf;
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
    trace::TraceLayer,
};
use tracing::info;

/// Create the main axum application with all routes and middleware.
pub async fn create_app(config: WebConfig) -> Result<Router> {
    let mut app = Router::new()
        // API routes
        .route("/api/snapshot", get(handlers::get_snapshot))
        .route("/api/health", get(handlers::health_check))
        // WebSocket route
        .route("/ws", get(websocket::websocket_handler));

    // Add static file serving if path is configured
    if let Some(static_path) = &config.static_path {
        let static_path = PathBuf::from(static_path);

        if static_path.exists() {
            info!("Serving static files from: {:?}", static_path);

            // Serve static files at /static/*
            app = app.nest_service(
                "/static",
                get_service(ServeDir::new(&static_path)).handle_error(|error| async move {
                    (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Unhandled internal error: {}", error),
                    )
                }),
            );

            // Serve index.html at root
            let index_file = static_path.join("index.html");
            if index_file.exists() {
                app = app.route("/", get(handlers::serve_index));
            } else {
                app = app.route("/", get(handlers::default_index));
            }
        } else {
            tracing::warn!(
                "Static path {:?} does not exist, serving default index",
                static_path
            );
            app = app.route("/", get(handlers::default_index));
        }
    } else {
        // No static path configured, serve default index
        app = app.route("/", get(handlers::default_index));
    }

    // Add middleware layers
    let service_builder = ServiceBuilder::new().layer(TraceLayer::new_for_http());

    // Add CORS if enabled
    if config.enable_cors {
        app = app.layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        );
    }

    app = app.layer(service_builder);

    Ok(app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_app() {
        let config = WebConfig::default();
        let app = create_app(config).await;
        assert!(app.is_ok());
    }
}
