#[derive(Clone, serde::Deserialize, serde::Serialize, utoipa::ToSchema)]
pub struct BuilderMod {
    pub name: String,
    pub remote: u16,
    pub builder: crate::builders::BuilderEnum,
    pub config: serde_json::Value,
}

#[derive(Clone, serde::Deserialize, serde::Serialize, utoipa::ToSchema)]
pub struct BuilderModQuery {
    pub builder: String,
}
