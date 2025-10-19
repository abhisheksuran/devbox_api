use crate::models::DevBox;
use crate::providers::ProviderEnum;

use axum::response::{IntoResponse, Response};
use axum::{Json, http::StatusCode};
use serde_json::json;
use tracing::{error, info};
use utoipa::IntoParams;
use utoipa::ToSchema;
use validator::Validate;
use validator::ValidationError;
use validator_derive::Validate;

#[derive(serde::Serialize, serde::Deserialize, Validate, IntoParams, ToSchema)]
pub struct ProviderQuery {
    pub provider: ProviderEnum,
    #[validate(custom = "validate_path")]
    pub path: String,
}

fn validate_path(path: &str) -> Result<(), ValidationError> {
    let p = std::path::Path::new(path);

    if p.exists() && p.is_dir() {
        Ok(())
    } else {
        Err(ValidationError::new("Not a valid directory"))
    }
}

pub fn validator(params: &ProviderQuery, devcontainer: Option<&DevBox>) -> Response {
    if let Some(dc) = &devcontainer
        && !dc.is_valid()
    {
        error!("DevBox creation body is not valid");
        return error_response(
            StatusCode::BAD_REQUEST,
            "Invalid Creation, need either Image or Dockerfile",
        );
    };
    if let Err(e) = params.validate() {
        error!("Validation failed: {:?}", e);
        return error_response(StatusCode::BAD_REQUEST, e.to_string().as_str());
    };
    Response::default()
}

fn error_response(status: StatusCode, message: &str) -> Response {
    (status, Json(json!({ "error": message }))).into_response()
}
