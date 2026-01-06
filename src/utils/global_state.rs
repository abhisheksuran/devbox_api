use crate::db::get_latest_config;
use crate::providers::{
    DevBoxProvider, ProviderEnum, aws::AwsProvider, azure::AzureProvider, docker::DockerProvider,
    docker::DockerProviderMod,
};
use crate::utils::Client;
use crate::utils::Remote;
use crate::utils::TunnelConfig;
use bollard::Docker;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

pub struct TunnelManager {
    handle: Option<tokio::task::JoinHandle<()>>,
    cancel: Option<CancellationToken>,
    local_port: u32,
    remote_port: u32,
}

impl Clone for TunnelManager {
    fn clone(&self) -> Self {
        Self {
            handle: None,
            cancel: self.cancel.clone(),
            local_port: self.local_port,
            remote_port: self.remote_port,
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    pub docker: DockerProvider,
    pub azure: Option<AzureProvider>,
    pub aws: Option<AwsProvider>,
    pub log_storage_path: String,
    pub ssh_session: Option<Arc<russh::client::Handle<Client>>>,
    pub tunnel: Option<TunnelManager>,
}

impl AppState {
    pub fn get_provider_strategy(
        &self,
        provider: &ProviderEnum,
    ) -> Box<dyn DevBoxProvider + Send + Sync> {
        match provider {
            ProviderEnum::Docker => Box::new(self.docker.clone()),
            ProviderEnum::Azure => Box::new(self.azure.as_ref().unwrap().clone()),
            ProviderEnum::Aws => Box::new(self.aws.as_ref().unwrap().clone()),
        }
    }

    pub fn get_docker_connection(&self) -> Arc<Docker> {
        self.docker.get_connection()
    }

    pub async fn get_state(
        state: Arc<RwLock<Option<AppState>>>,
    ) -> Result<AppState, Box<dyn std::error::Error>> {
        let guard = state.read().await;

        let state_clone = match &*guard {
            Some(state) => Ok(state.clone()),
            None => {
                return Err("AppState not initialized".into());
            }
        };
        state_clone
    }
}

pub async fn update_appstate(app_state: Arc<RwLock<Option<AppState>>>) {
    let mut write_guard = app_state.write().await;

    let previous_state = write_guard.take(); // Option<AppState>

    let provider_data = match get_latest_config().await {
        Ok(data) => data,
        Err(e) => {
            error!("Failed to obtain provider config: {e}");
            *write_guard = previous_state;
            return;
        }
    };
    drop(write_guard);
    let mut azure_opt: Option<AzureProvider> = None;
    let mut aws_opt: Option<AwsProvider> = None;
    let mut docker_opt: Option<DockerProviderMod> = None;

    for (provider, cfg, art) in provider_data {
        info!("{:?}", provider);
        match provider.as_str() {
            "azure" => {
                azure_opt = if cfg != "" && art.is_some() {
                    Some(AzureProvider::new(cfg, art.unwrap()))
                } else {
                    None
                }
            }

            "aws" => {
                aws_opt = if cfg != "" && art.is_some() {
                    Some(AwsProvider::new(cfg, art.unwrap()))
                } else {
                    None
                }
            }
            "docker" => {
                let config_string: String = serde_json::from_value(cfg.clone()).unwrap();
                let config: Option<TunnelConfig> = serde_json::from_str(&config_string).ok();

                docker_opt = Some(DockerProviderMod {
                    artifactory: art,
                    remote: config,
                })
            }
            other => warn!("Unknown provider `{other}` – ignored"),
        }
    }

    let mut docker_conn =
        DockerProvider::refresh_connection(app_state.clone(), docker_opt.clone().unwrap().remote)
            .await
            .ok();
    if docker_conn.is_none() {
        docker_conn = Some(Arc::new(bollard::Docker::connect_with_defaults().unwrap()));
    }

    if let Some(old_state) = previous_state {
        let new_state = AppState {
            docker: DockerProvider::new(
                docker_conn.unwrap(),
                docker_opt.clone().unwrap().artifactory,
                docker_opt.unwrap().remote,
            ),
            azure: azure_opt,
            aws: aws_opt,
            ..old_state
        };

        let mut guard = app_state.write().await;
        *guard = Some(new_state);
    }
}

pub async fn update_tunnel(
    state: Arc<RwLock<Option<AppState>>>,
    new_remote: u32,
    new_local: u32,
    session_refresh: Option<TunnelConfig>,
) -> Result<String, Box<dyn std::error::Error>> {
    // take out old state
    let previous_state = {
        let mut guard = state.write().await;
        guard.take()
    };

    let copied_prev_state = previous_state.clone();

    // stop old tunnel outside the lock
    if let Some(old_state) = previous_state
        && let Some(mut mgr) = old_state.tunnel
    {
        if let Some(cancel) = mgr.cancel.take() {
            cancel.cancel();
        }
        if let Some(handle) = mgr.handle.take() {
            let _ = handle.await;
        }
    }
    if new_remote == 0 {
        if let Some(old_state) = copied_prev_state {
            let new_state = AppState {
                ssh_session: None,
                tunnel: None,
                ..old_state
            };

            let mut guard = state.write().await;
            *guard = Some(new_state);
        }
        return Ok("SUCCESS".to_string());
    }

    // create new session if requested
    let (session, new_session_arc) = match session_refresh {
        Some(cfg) => {
            let session = DockerProvider::connect(cfg).await?;
            let arc_session = Arc::new(session);
            (arc_session.clone(), Some(arc_session))
        }
        None => {
            // reuse existing session from state if available
            let guard = state.read().await;
            let arc_session = guard
                .as_ref()
                .and_then(|s| s.ssh_session.clone())
                .expect("no ssh_session available");
            (arc_session.clone(), Some(arc_session))
        }
    };

    // start new tunnel
    let cancel = CancellationToken::new();
    let handle = tokio::spawn(DockerProvider::run_tunnel(
        session,
        new_remote,
        new_local,
        cancel.clone(),
    ));

    // insert new state
    if let Some(old_state) = copied_prev_state {
        let new_state = AppState {
            ssh_session: new_session_arc,
            tunnel: Some(TunnelManager {
                handle: Some(handle),
                cancel: Some(cancel),
                local_port: new_local,
                remote_port: new_remote,
            }),
            ..old_state
        };

        let mut guard = state.write().await;
        *guard = Some(new_state);
    }

    Ok(format!(
        "Tunnel updated: local {new_local} -> remote {new_remote}"
    ))
}
