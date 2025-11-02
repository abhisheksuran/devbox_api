use crate::apps::config::{
    __path_update_aws_provider, __path_update_azure_provider, __path_update_docker_provider,
    __path_update_state,
};
use crate::apps::devbox::{
    ActionQuery, ProviderQuery, __path_action_on_devbox, __path_get_task_logs,
    __path_list_all_devbox, __path_new_devbox,
};
use crate::models::Build;
use crate::providers::ProviderEnum;
use utoipa::OpenApi;

/// Main OpenAPI document combining schemas and paths
#[derive(OpenApi)]
#[openapi(
    paths(
        new_devbox,
        list_all_devbox,
        action_on_devbox,
        get_task_logs,
        update_state,
        update_docker_provider,
        update_azure_provider,
        update_aws_provider
    ),
    components(schemas(Build, ProviderEnum, ProviderQuery, ActionQuery)),
    info(title = "Devbox API", version = "0.1.1")
)]
pub struct ApiDoc;

pub fn openapi_json() -> String {
    serde_json::to_string(&ApiDoc::openapi()).unwrap()
}
