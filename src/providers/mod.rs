pub mod docker;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum ProviderEnum {
    DOCKER,
    AZURE,
    AWS,
}
