use axum::{
    Router,
    routing::{get, post},
};
use bollard::Docker;
use http::{HeaderName, HeaderValue, Method};
use routes::{all_containers, get_container, new_container, new_devbox, websocket_exec_handler};
use std::sync::Arc;
mod db;
mod handlers;
mod logs;
mod models;
mod openapi;
mod providers;
mod routes;
mod utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let docker = Arc::new(Docker::connect_with_local_defaults()?);
    let cors = tower_http::cors::CorsLayer::new()
        .allow_origin([HeaderValue::from_static("http://127.0.0.1:3000")])
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([HeaderName::from_static("content-type")]);

    let app = Router::new()
        .route("/container/get", get(get_container))
        .route("/container/list", get(all_containers))
        .route("/container/create", post(new_container))
        .route("/devbox/create", post(new_devbox))
        .route("/ws/docker/{id}", get(websocket_exec_handler))
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
        .layer(cors)
        .with_state(docker.clone());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8000")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
    Ok(())
}
