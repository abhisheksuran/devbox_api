use utoipa::IntoParams;
use utoipa::ToSchema;

#[derive(serde::Deserialize, IntoParams, ToSchema)]
pub struct ContainerQuery {
    pub name: String,
}
