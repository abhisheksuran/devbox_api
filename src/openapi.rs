use crate::models::{Build, Container, ContainerConfig};
use crate::providers::ProviderEnum;
use crate::routes::{ContainerQuery, ProviderQuery};
use utoipa::OpenApi;

/// Main OpenAPI document combining schemas and paths
#[derive(OpenApi)]
#[openapi(
    paths(
        crate::routes::all_containers,
        crate::routes::get_container,
        crate::routes::new_container,
        crate::routes::new_devbox
    ),
    components(schemas(
        Container,
        ContainerConfig,
        Build,
        ProviderEnum,
        ContainerQuery,
        ProviderQuery
    )),
    info(title = "Devbox API", version = "0.1.0")
)]
pub struct ApiDoc;

pub fn openapi_json() -> String {
    serde_json::to_string(&ApiDoc::openapi()).unwrap()
}
