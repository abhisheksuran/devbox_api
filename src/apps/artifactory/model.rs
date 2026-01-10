#[derive(Clone, serde::Deserialize, serde::Serialize, utoipa::ToSchema)]
pub struct Artifactory {
    pub server: String,
    pub repository_name: String,
    pub username: Option<String>,
    pub password: Option<String>,
}

#[derive(Clone, serde::Deserialize, serde::Serialize, utoipa::ToSchema)]
pub struct ArtifactoryMod {
    pub provider: crate::providers::ProviderEnum,
    pub config: Artifactory,
}
