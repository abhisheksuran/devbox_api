use crate::db::{delete_builder, insert_builder, list_builders};
use axum::http::StatusCode;
use axum::{Json, extract::Query, response::IntoResponse};

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct BuilderMod {
    name: String,
    remote: u16,
    builder: String,
    config: serde_json::Value,
}

pub async fn add_builder(Json(data): Json<BuilderMod>) -> impl IntoResponse {
    let success = (insert_builder(
        &data.name,
        data.remote,
        &data.builder,
        data.config.as_str().unwrap(),
    )
    .await)
        .is_ok();

    if success {
        Ok((StatusCode::OK, "SUCCESS"))
    } else {
        Err((StatusCode::INTERNAL_SERVER_ERROR, "FAILED TO ADD BUILDER"))
    }
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct BuilderModQuery {
    pub builder: String,
}

pub async fn remove_builder(Query(builder): Query<BuilderModQuery>) -> impl IntoResponse {
    let success = delete_builder(&builder.builder.to_string()).await.is_ok();
    if success {
        Ok((StatusCode::OK, "SUCCESS"))
    } else {
        Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "FAILED TO REMOVE BUILDER",
        ))
    }
}

pub async fn list_all_builders() -> impl IntoResponse {
    match list_builders().await {
        Ok(data) => Ok((StatusCode::OK, Json(data))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("FAILED: {e}"))),
    }
}
