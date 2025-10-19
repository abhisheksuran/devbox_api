use azure_core::credentials::TokenCredential;
use azure_identity::AzureCliCredential;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct ContainerGroup {
    pub location: String,
    pub properties: ContainerGroupProperties,
}

#[derive(Serialize)]
pub struct ContainerGroupProperties {
    pub containers: Vec<Container>,
    pub os_type: String,
    pub ip_address: IpAddress,
    pub image_registry_credentials: Vec<ImageRegistryCredential>,
}

#[derive(Serialize)]
pub struct Container {
    pub name: String,
    pub properties: ContainerProperties,
}

#[derive(Serialize)]
pub struct ContainerProperties {
    pub image: String,
    pub resources: Resources,
    pub ports: Vec<Port>,
}

#[derive(Serialize)]
pub struct Resources {
    pub requests: ResourceRequests,
}

#[derive(Serialize)]
pub struct ResourceRequests {
    pub cpu: f64,
    pub memory_in_gb: f64,
}

#[derive(Serialize)]
pub struct Port {
    pub port: u16,
}

#[derive(Serialize)]
pub struct IpAddress {
    pub r#type: String,
    pub ports: Vec<IpPort>,
}

#[derive(Serialize)]
pub struct IpPort {
    pub protocol: String,
    pub port: u16,
}

#[derive(Serialize)]
pub struct ImageRegistryCredential {
    pub server: String,
    pub username: String,
    pub password: String,
}

pub async fn create(container_group: ContainerGroup) -> Result<(), Box<dyn std::error::Error>> {
    // Authenticate
    let credentials = AzureCliCredential::new(None)?;
    let token = credentials
        .get_token(&["https://management.azure.com/.default"], None)
        .await?;

    let token_str = token.token.secret();

    // Send request to Azure
    let client = Client::new();
    let response = client
        .put("https://management.azure.com/subscriptions/<subscription-id>/resourceGroups/<resource-group>/providers/Microsoft.ContainerInstance/containerGroups/<container-name>?api-version=2021-10-01")
        .bearer_auth(token_str)
        .json(&container_group)
        .send()
        .await?;

    println!("Response: {:?}", response.text().await?);
    Ok(())
}
