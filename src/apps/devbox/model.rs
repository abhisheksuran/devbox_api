use std::sync::Arc;

use crate::builders::BuilderEnum;
use crate::models::DevBox;
use crate::providers::ProviderEnum;
use crate::utils::AppState;

use axum::response::{IntoResponse, Response};
use axum::{Json, http::StatusCode};
use serde_json::json;
use tokio::sync::RwLock;
use tracing::{error, info};
use utoipa::IntoParams;
use utoipa::ToSchema;
use validator::Validate;
use validator::ValidationError;
use validator_derive::Validate;

#[derive(serde::Serialize, ToSchema)]
pub struct ContainerListResponse {
    pub id: i32,
    pub name: String,
    pub provider: String,
    pub status: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq, Hash, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum ActionEnum {
    Start,
    Stop,
    Delete,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, IntoParams, ToSchema)]
pub struct ActionQuery {
    pub action: ActionEnum,
}

#[derive(serde::Serialize, serde::Deserialize, Validate, IntoParams, ToSchema)]
pub struct ProviderQuery {
    pub provider: ProviderEnum,
    #[validate(custom = "validate_path")]
    pub path: String,
    pub builder: String,
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

pub async fn validate_state(
    state: Arc<RwLock<Option<AppState>>>,
    provider: ProviderEnum,
) -> Response {
    let guard = state.read().await;

    let app_state = match &*guard {
        Some(state) => state.clone(),
        None => return error_response(StatusCode::BAD_REQUEST, "AppState is not initialized"),
    };

    match provider {
        ProviderEnum::Azure => match &app_state.azure {
            Some(_val) => (),
            None => {
                return error_response(StatusCode::BAD_REQUEST, "Azure config is not initialized");
            }
        },
        ProviderEnum::Aws => match &app_state.aws {
            Some(_val) => (),
            None => {
                return error_response(StatusCode::BAD_REQUEST, "Aws config is not initialized");
            }
        },
        ProviderEnum::Docker => (),
    }
    Response::default()
}
