use crate::task_log;
use reqwest::Client;
use serde::Serialize;
use tokio::time::{Duration, sleep};
use tracing::{error, info};

#[derive(Serialize)]
pub struct ContainerGroup {
    pub location: String,
    pub properties: ContainerGroupProperties,
}

#[derive(Serialize)]
pub struct ContainerGroupProperties {
    pub containers: Vec<Container>,
    #[serde(rename = "osType")]
    pub os_type: String,
    #[serde(rename = "ipAddress")]
    pub ip_address: IpAddress,
    #[serde(rename = "imageRegistryCredentials")]
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
    #[serde(rename = "memoryInGB")]
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

pub async fn create(
    name: &str,
    container_group: ContainerGroup,
    subscription: &str,
    resource_group: &str,
    token: &str,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    // // Authenticate
    // let credentials = AzureCliCredential::new(None)?;
    // let token = credentials
    //     .get_token(&["https://management.azure.com/.default"], None)
    //     .await?;

    // let token_str = token.token.secret();

    // Send request to Azure
    let client = Client::new();
    let response = client
        .put(format!("https://management.azure.com/subscriptions/{subscription}/resourceGroups/{resource_group}/providers/Microsoft.ContainerInstance/containerGroups/{name}?api-version=2021-10-01"))
        .bearer_auth(token)
        .json(&container_group)
        .send()
        .await?;

    // println!("Response: {:?}", response.text().await?);
    // Parse the JSON body
    let body: serde_json::Value = response.json().await?;

    // Extract the "id" field
    if let Some(container_id) = body.get("id").and_then(|v| v.as_str()) {
        task_log!("Container Group ID: {}", container_id);
        Ok(serde_json::json!({ "status": "success", "container_id": container_id }))
    } else {
        Ok(serde_json::json!({ "status": "notfound", "container_id": "Not Found" }))
    }
}

pub async fn get_status(
    container_name: &str,
    subscription: &str,
    resource_group: &str,
    token: &str,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let client = Client::new();
    let url = format!(
        "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.ContainerInstance/containerGroups/{}?api-version=2023-05-01",
        subscription, resource_group, container_name
    );

    let response = client.get(&url).bearer_auth(token).send().await?;

    let body: serde_json::Value = response.json().await?;

    if let Some(status) = body.get("provisioningState").and_then(|v| v.as_str()) {
        Ok(serde_json::json!({ "status": status}))
    } else {
        Ok(serde_json::json!({ "status": "notfound" }))
    }
}

pub async fn wait_until_running(
    container_name: &str,
    subscription: &str,
    resource_group: &str,
    token: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        let status_json = get_status(container_name, subscription, resource_group, token).await?;
        let status = status_json
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        task_log!("Current status: {}", status);

        match status {
            "Running" => {
                task_log!("Container is now running.");
                return Ok(());
            }
            "Failed" | "Stopped" | "Terminated" => {
                task_log!("Container provisioning failed or stopped.");
                return Err("Container provisioning failed or stopped.".into());
            }
            _ => {
                task_log!("Waiting for container to start...");
                sleep(Duration::from_secs(10)).await;
            }
        }
    }
}
