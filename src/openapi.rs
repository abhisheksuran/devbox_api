use crate::apps::artifactory::{
    __path_add_artifactory, __path_list_all_artifactory, __path_modify_artifactory,
    __path_remove_artifactory,
};
use crate::apps::builder::{__path_add_builder, __path_list_all_builders, __path_remove_builder};
use crate::apps::config::{
    __path_update_aws_provider, __path_update_azure_provider, __path_update_docker_provider,
    __path_update_state,
};
use crate::apps::devbox::{
    __path_action_on_devbox, __path_get_details, __path_get_task_logs, __path_list_all_devbox,
    __path_new_devbox, ActionQuery, ProviderQuery,
};
use crate::apps::provider::{
    __path_list_all_providers, __path_modify_providers, __path_update_aws, __path_update_azure,
    __path_update_docker,
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
        get_details,
        action_on_devbox,
        get_task_logs,
        update_state,
        update_docker_provider,
        update_azure_provider,
        update_aws_provider,
        add_artifactory,
        list_all_artifactory,
        modify_artifactory,
        remove_artifactory,
        add_builder,
        list_all_builders,
        remove_builder,
        list_all_providers,
        modify_providers,
        update_aws,
        update_azure,
        update_docker
    ),
    components(schemas(Build, ProviderEnum, ProviderQuery, ActionQuery)),
    info(title = "Devbox API", version = "0.1.1")
)]
pub struct ApiDoc;

pub fn openapi_json() -> String {
    serde_json::to_string(&ApiDoc::openapi()).unwrap()
}
