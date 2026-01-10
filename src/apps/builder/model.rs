#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct BuilderMod {
    pub name: String,
    pub remote: u16,
    pub builder: String,
    pub config: serde_json::Value,
}
