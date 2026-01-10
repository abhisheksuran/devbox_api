use crate::providers::ProviderEnum;

#[derive(Clone, serde::Deserialize, serde::Serialize, utoipa::ToSchema)]
pub struct ProviderMod {
    pub provider: ProviderEnum,
    pub config: serde_json::Value,
}

#[derive(Clone, serde::Deserialize, serde::Serialize, utoipa::ToSchema)]
pub struct AzureMod {
    tanent: String,
    subscription: String,
    token: String,
    location: String,
    resource_group: String,
    address_type: Option<String>,
}

#[derive(Clone, serde::Deserialize, serde::Serialize, utoipa::ToSchema)]
pub struct AwsMod {
    access_key: String,
    secret_key: String,
    region: String,
}

#[derive(Clone, serde::Deserialize, serde::Serialize, utoipa::ToSchema)]

pub struct RemoteConfig {
    pub remote_ip: String,
    pub ssh_port: Option<u16>,
    pub service_port: u32,
    pub private_key: Option<String>,
    pub passphrase: Option<String>,
    pub username: String,
    pub password: Option<String>,
    pub local_port: u32,
}
