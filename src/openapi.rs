use crate::apps::devbox::ProviderQuery;
use crate::models::Build;
use crate::providers::ProviderEnum;
use utoipa::OpenApi;

/// Main OpenAPI document combining schemas and paths
#[derive(OpenApi)]
#[openapi(
    paths(crate::apps::devbox::new_devbox),
    components(schemas(Build, ProviderEnum, ProviderQuery)),
    info(title = "Devbox API", version = "0.1.0")
)]
pub struct ApiDoc;

pub fn openapi_json() -> String {
    serde_json::to_string(&ApiDoc::openapi()).unwrap()
}
