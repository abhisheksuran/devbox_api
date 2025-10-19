mod container;
use container::{
    Container, ContainerGroup, ContainerGroupProperties, ContainerProperties,
    ImageRegistryCredential, IpAddress, IpPort, Port, ResourceRequests, Resources, create,
};

use crate::models::DevBox;
use crate::providers::DevBoxProvider;
use crate::utils::artifactory::Artifactory;

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct AzureProvider {
    tanent: String,
    subscription: String,
    token: String,
    location: String,
    resource_group: String,
    address_type: Option<String>, //Public or Private
    artifactory: Artifactory,
}

#[async_trait::async_trait]
impl DevBoxProvider for AzureProvider {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    async fn create_devbox(
        &self,
        devcontainer: Option<DevBox>,
        path: String,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let devcontainer = match devcontainer {
            Some(dc) => dc,
            None => DevBox::new(path.clone()).await,
        };
        let image = self
            .artifactory
            .push_image(&devcontainer.image, "latest")
            .await;

        // Define container group
        let container_group = ContainerGroup {
            location: self.location.clone(),
            properties: ContainerGroupProperties {
                containers: vec![Container {
                    name: devcontainer.name,
                    properties: ContainerProperties {
                        image: image,
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
        create(container_group).await?;
        let azure_url = "https://management.azure.com/.default";
        todo!("Implement this feature later");
    }
}
