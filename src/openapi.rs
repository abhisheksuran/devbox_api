use crate::apps::data::ContainerQuery;
use crate::apps::devbox::ProviderQuery;
use crate::models::{Build, Container, ContainerConfig};
use crate::providers::ProviderEnum;
use utoipa::OpenApi;

/// Main OpenAPI document combining schemas and paths
#[derive(OpenApi)]
#[openapi(
    paths(
        crate::apps::data::all_containers,
        crate::apps::data::get_container,
        crate::apps::data::new_container,
        crate::apps::devbox::new_devbox
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
