use crate::providers::aws::AwsProvider;
use crate::providers::azure::AzureProvider;
use crate::providers::docker::DockerProvider;
use crate::utils::artifactory::Artifactory;
use apps::config::config_routes;
use apps::devbox::devbox_routes;
use axum::{Router, routing::get};
use db::get_latest_config;
use http::{HeaderName, HeaderValue, Method};
use std::env;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{Duration, sleep};
use tracing::{error, info, warn};
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
                if let Some(ref cur_state) = *read_guard {
                    container_status_update(cur_state).await;
                } else {
                    info!("State not yet initialized");
                    sleep(Duration::from_secs(5)).await;
                    continue;
                }
            }

            {
                let mut write_guard = bg_state.write().await;

                let previous_state = write_guard.take(); // Option<AppState>

                let provider_data = match get_latest_config().await {
                    Ok(data) => data,
                    Err(e) => {
                        error!("Failed to obtain provider config: {e}");
                        *write_guard = previous_state;
                        sleep(Duration::from_secs(5)).await;
                        continue;
                    }
                };

                let mut azure_opt: Option<AzureProvider> = None;
                let mut aws_opt: Option<AwsProvider> = None;
                let mut docker_art_opt: Option<Artifactory> = None;

                for (provider, cfg, art) in provider_data {
                    match provider.as_str() {
                        "azure" => azure_opt = Some(AzureProvider::new(cfg, art)),
                        "aws" => aws_opt = Some(AwsProvider::new(cfg, art)),
                        "docker" => docker_art_opt = Some(art),
                        other => warn!("Unknown provider `{other}` – ignored"),
                    }
                }

                let docker_conn = match &previous_state {
                    Some(prev) => prev.docker.get_connection().into(),
                    None => Arc::new(bollard::Docker::connect_with_local_defaults().unwrap()),
                };

                let new_state = AppState {
                    docker: DockerProvider::new(docker_conn, docker_art_opt),
                    azure: azure_opt,
                    aws: aws_opt,
                    log_storage_path: previous_state.unwrap().log_storage_path,
                };

                *write_guard = Some(new_state);
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
