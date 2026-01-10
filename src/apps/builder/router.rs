use crate::apps::builder::model::BuilderMod;
use crate::db::{delete_builder, insert_builder, list_builders};
use crate::utils::TunnelConfig;
use axum::http::StatusCode;
use axum::{Json, extract::Query, response::IntoResponse};

pub async fn add_builder(Json(data): Json<BuilderMod>) -> impl IntoResponse {
    let cfg = data.config.clone();

    if data.remote == 1 {
        match serde_json::from_value::<TunnelConfig>(cfg) {
            Ok(c) => c,
            Err(_) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Remote builder should have
                    pub remote_ip: String,
                    pub ssh_port: Option<u16>,
                    pub service_port: u32,
                    pub private_key: Option<PathBuf>,
                    pub passphrase: Option<String>,
                    pub username: String,
                    pub password: Option<String>,
                    pub local_port: u32,",
                )
                    .into_response();
            }
        };
    }

    let success = (insert_builder(
        &data.name,
        data.remote,
        &data.builder,
        &data.config.to_string(),
    )
    .await)
        .is_ok();

    if success {
        (StatusCode::OK, "SUCCESS").into_response()
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, "FAILED TO ADD BUILDER").into_response()
    }
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct BuilderModQuery {
    pub builder: String,
}

pub async fn remove_builder(Query(builder): Query<BuilderModQuery>) -> impl IntoResponse {
    let success = delete_builder(&builder.builder.to_string()).await.is_ok();
    if success {
        (StatusCode::OK, "SUCCESS").into_response()
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "FAILED TO REMOVE BUILDER",
        )
            .into_response()
    }
}

pub async fn list_all_builders() -> impl IntoResponse {
    match list_builders().await {
        Ok(data) => (StatusCode::OK, Json(data)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("FAILED: {e}")).into_response(),
    }
}
