#[derive(Clone, serde::Deserialize, serde::Serialize, utoipa::ToSchema)]
pub struct Artifactory {
    pub server: String,
    pub repository_name: String,
    pub username: Option<String>,
    pub password: Option<String>,
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct ArtifactoryMod {
    pub provider: String,
    pub config: Artifactory,
}
