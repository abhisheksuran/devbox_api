use apps::config::config_routes;
use apps::devbox::devbox_routes;
use axum::{Router, routing::get};
use http::{HeaderName, HeaderValue, Method};
use std::env;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{Duration, sleep};
use tracing::info;
use utils::AppState;
use utils::monitor::container_status_update;
mod apps;
mod db;
mod logs;
mod models;
mod openapi;
mod providers;
mod utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let app_state: Arc<RwLock<Option<AppState>>> = Arc::new(RwLock::new(None));

    let bg_state = Arc::clone(&app_state);

    tokio::spawn(async move {
        loop {
            {
                let read_guard = bg_state.read().await;
                if let Some(ref app_state) = *read_guard {
                    let _ = container_status_update(app_state).await;
                } else {
                    info!("State not initialized.");
                }
            }
            sleep(Duration::from_secs(5)).await;
        }
    });

    let allowed_origin =
        env::var("ALLOWED_ORIGIN").unwrap_or_else(|_| "http://localhost:8080".to_string());

    let origin_header =
        HeaderValue::from_str(&allowed_origin).expect("Invalid ALLOWED_ORIGIN value");

    let cors = tower_http::cors::CorsLayer::new()
        .allow_origin([origin_header])
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([HeaderName::from_static("content-type")]);

    let app = Router::new()
        .with_state(app_state.clone())
        .nest("/devbox", devbox_routes(app_state.clone()))
        .nest("/config", config_routes(app_state.clone()))
        // OpenAPI JSON
        .route(
            "/api-docs/openapi.json",
            get(|| async {
                axum::Json(
                    serde_json::from_str::<serde_json::Value>(&openapi::openapi_json()).unwrap(),
                )
            }),
        )
        // Swagger UI (serve a simple Swagger UI page using the CDN)
        .route(
            "/swagger",
            get(|| async {
                axum::response::Html(
                    r#"<!DOCTYPE html>
        <html>
          <head>
            <meta charset="utf-8" />
            <title>Swagger UI</title>
            <link rel="stylesheet" href="https://unpkg.com/swagger-ui-dist/swagger-ui.css" />
          </head>
          <body>
            <div id="swagger-ui"></div>
            <script src="https://unpkg.com/swagger-ui-dist/swagger-ui-bundle.js"></script>
            <script>
              window.onload = function() {
                SwaggerUIBundle({
                  url: '/api-docs/openapi.json',
                  dom_id: '#swagger-ui'
                });
              };
            </script>
          </body>
        </html>"#
                        .to_string(),
                )
            }),
        )
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .layer(cors);
    let binding_address =
        env::var("BINDING_ADDRESS").unwrap_or_else(|_| "127.0.0.1:8000".to_string());
    info!("Starting server at {}", binding_address);
    let listener = tokio::net::TcpListener::bind(binding_address)
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
    Ok(())
}
