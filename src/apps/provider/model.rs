use crate::providers::ProviderEnum;

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct ProviderMod {
    pub provider: ProviderEnum,
    pub config: serde_json::Value,
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct AzureMod {
    tanent: String,
    subscription: String,
    token: String,
    location: String,
    resource_group: String,
    address_type: Option<String>,
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct AwsMod {
    access_key: String,
    secret_key: String,
    region: String,
}
