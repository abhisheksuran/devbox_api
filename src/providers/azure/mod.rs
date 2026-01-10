mod container;

use bollard::Docker;
use container::{
    Container, ContainerGroup, ContainerGroupProperties, ContainerProperties,
    ImageRegistryCredential, IpAddress, IpPort, Port, ResourceRequests, Resources, create,
    wait_until_running,
};
use std::sync::Arc;

use crate::apps::artifactory::Artifactory;
use crate::models::DevBox;
use crate::providers::DevBoxProvider;
use crate::task_log;
use azure_core::credentials::AccessToken;
use azure_core::credentials::TokenCredential;
use azure_identity::AzureCliCredential;

#[derive(Clone, serde::Deserialize, serde::Serialize, utoipa::ToSchema)]
pub struct AzureProvider {
    tanent: String,
    subscription: String,
    token: String,
    location: String,
    resource_group: String,
    address_type: Option<String>, //Public or Private
    artifactory: Artifactory,
}

impl AzureProvider {
    async fn get_token(&mut self) -> Result<AccessToken, Box<dyn std::error::Error>> {
        let credentials = AzureCliCredential::new(None)?;
        let token = credentials
            .get_token(&["https://management.azure.com/.default"], None)
            .await?;
        self.token = token.token.secret().to_string();
        Ok(token)
    }
    pub fn new(config: serde_json::Value, artifactory: Artifactory) -> Self {
        AzureProvider {
            tanent: config.get("tanent").unwrap().to_string(),
            subscription: config.get("subscription").unwrap().to_string(),
            token: "TOKEN".to_string(),
            location: config.get("location").unwrap().to_string(),
            resource_group: config.get("resource_group").unwrap().to_string(),
            address_type: Some(config.get("address_type").unwrap().to_string()),
            artifactory,
        }
    }
}

#[async_trait::async_trait]
impl DevBoxProvider for AzureProvider {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    async fn create_devbox(
        &mut self,
        builder: &str,
        devcontainer: DevBox,
        path: String,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // let devcontainer = match devcontainer {
        //     Some(dc) => dc,
        //     None => DevBox::new(path.clone()).await,
        // };
        let devcontainer_cp = devcontainer.clone();

        let docker = Arc::new(Docker::connect_with_local_defaults().unwrap());
        let image = devcontainer
            .create_image(builder, path, Some(self.artifactory.clone()))
            .await?;

        // Define container group
        let container_group = ContainerGroup {
            location: self.location.clone(),
            properties: ContainerGroupProperties {
                containers: vec![Container {
                    name: devcontainer.name,
                    properties: ContainerProperties {
                        image,
                        resources: Resources {
                            requests: ResourceRequests {
                                cpu: devcontainer.cpu_limit.unwrap_or(1.0),
                                memory_in_gb: devcontainer.memory_limit.unwrap_or(1.5),
                            },
                        },
                        ports: vec![Port { port: 80 }],
                    },
                }],
                os_type: devcontainer.target_platform.unwrap_or("Linux".to_string()),
                ip_address: IpAddress {
                    r#type: self.address_type.clone().unwrap_or("Public".to_string()),
                    ports: vec![IpPort {
                        protocol: "tcp".to_string(),
                        port: 80,
                    }],
                },
                image_registry_credentials: vec![ImageRegistryCredential {
                    server: self.artifactory.server.clone(),
                    username: self.artifactory.username.clone().unwrap_or("".to_string()),
                    password: self.artifactory.password.clone().unwrap_or("".to_string()),
                }],
            },
        };

        let _ = self.get_token().await;
        task_log!("Creating Azure Container Instance");
        let id = create(
            &devcontainer_cp.name,
            container_group,
            &self.subscription,
            &self.resource_group,
            &self.token,
        )
        .await?;

        match wait_until_running(
            &devcontainer_cp.name,
            &self.subscription,
            &self.resource_group,
            &self.token,
        )
        .await
        {
            Ok(()) => Ok(id),
            _ => Err("Fail to create container".into()),
        }
    }

    async fn delete_devbox(&self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        todo!("To be implemented");
    }

    async fn start_devbox(&self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        todo!("To be implemented");
    }

    async fn stop_devbox(&self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        todo!("To be implemented");
    }
    async fn get_status(&self, id: String) -> Result<String, Box<dyn std::error::Error>> {
        todo!("To be implemented")
    }
}
