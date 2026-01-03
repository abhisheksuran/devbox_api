use russh::{ChannelId, client, keys::*};
use std::future::Future;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::copy_bidirectional;
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;

use std::net::{SocketAddr, ToSocketAddrs};
use tokio::net::TcpStream;
use tokio::time::{Duration, timeout};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TunnelConfig {
    pub remote_ip: String,
    pub ssh_port: Option<u16>,
    pub service_port: u32,
    pub private_key: Option<PathBuf>,
    pub passphrase: Option<String>,
    pub username: String,
    pub password: Option<String>,
    pub local_port: u32,
}

#[derive(Clone)]
pub struct Client;

impl client::Handler for Client {
    type Error = Box<dyn std::error::Error + Send + Sync>;

    async fn check_server_key(
        &mut self,
        server_public_key: &ssh_key::PublicKey,
    ) -> Result<bool, Self::Error> {
        println!("check_server_key: {server_public_key:?}");
        Ok(true)
    }

    async fn data(
        &mut self,
        channel: ChannelId,
        data: &[u8],
        _session: &mut client::Session,
    ) -> Result<(), Self::Error> {
        println!("data on channel {:?}: {}", channel, data.len());
        Ok(())
    }
}

#[async_trait::async_trait]
pub trait Remote {
    async fn connect(
        tunnel_cfg: TunnelConfig,
    ) -> Result<client::Handle<Client>, Box<dyn std::error::Error>> {
        let config = russh::client::Config::default();
        let sh = Client {};
        let mut session = russh::client::connect(
            Arc::new(config),
            (
                tunnel_cfg.remote_ip.as_str(),
                tunnel_cfg.ssh_port.unwrap_or(22),
            ),
            sh,
        )
        .await
        .unwrap();

        let condition = if tunnel_cfg.private_key.is_some() {
            let ssh_key = load_secret_key(
                tunnel_cfg.private_key.unwrap(),
                tunnel_cfg.passphrase.as_deref(),
            )
            .expect("Cannot load SSH private key");
            let ssh_key = PrivateKeyWithHashAlg::new(Arc::new(ssh_key), None);
            session
                .authenticate_publickey(tunnel_cfg.username, ssh_key)
                .await
                .unwrap()
                .success()
        } else {
            session
                .authenticate_password(tunnel_cfg.username, tunnel_cfg.password.unwrap())
                .await
                .unwrap()
                .success()
        };
        if condition {
            Ok(session)
        } else {
            Err(Box::new(std::io::Error::other("FAIL TO CONNECT TO SSH")))
        }
    }

    async fn disconnect(
        session: russh::client::Handle<Client>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        session
            .disconnect(russh::Disconnect::ByApplication, "Closing session", "")
            .await?;
        Ok(())
    }

    async fn check_port(host: &str, port: u16, tout: Option<Duration>) -> bool {
        let mut addrs: Vec<std::net::SocketAddr> = match (host, port).to_socket_addrs() {
            Ok(it) => it.collect::<Vec<_>>(),
            Err(_) => return false,
        };

        for addr in addrs.drain(..) {
            match timeout(
                tout.unwrap_or(Duration::from_secs(5)),
                TcpStream::connect(addr),
            )
            .await
            {
                Ok(Ok(_stream)) => return true,
                Ok(Err(_)) => continue,
                Err(_) => continue,
            }
        }
        false
    }

    async fn live_port_forward(
        session: russh::client::Handle<Client>,
        remote_port: u32,
        local_port: u32,
    ) {
        let local_listener = TcpListener::bind(("127.0.0.1", local_port as u16))
            .await
            .expect("Cannot bind local port");

        println!("Listening ...");
        loop {
            let (mut socket, _) = local_listener.accept().await.expect("Cannot accept client");
            println!("Client connected");

            let mut ssh_channel = session
                .channel_open_direct_tcpip("127.0.0.1", remote_port, "127.0.0.1", local_port)
                .await
                .expect("Cannot open SSH forwarding channel");

            let mut ssh_stream = ssh_channel.into_stream();
            tokio::spawn(async move {
                if let Err(e) = copy_bidirectional(&mut socket, &mut ssh_stream).await {
                    eprintln!("Forwarding error: {e}");
                }
                println!("Forwarding stopped");
            });
        }
    }

    async fn run_tunnel(
        session: Arc<russh::client::Handle<Client>>,
        remote_port: u32,
        local_port: u32,
        cancel: CancellationToken,
    ) {
        let listener = TcpListener::bind(("127.0.0.1", local_port as u16))
            .await
            .expect("Cannot bind local port");
        loop {
            tokio::select! {
                _ = cancel.cancelled() => {
                    println!("Tunnel cancelled");
                }
                res = listener.accept() => {
                    if let Ok((mut socket, _)) = res {
                        let mut channel = session
                            .channel_open_direct_tcpip("127.0.0.1".to_string(), remote_port,
                                                       "127.0.0.1".to_string(), local_port)
                            .await
                            .expect("Cannot open SSH channel");

                        let mut ssh_stream = channel.into_stream();
                        if let Err(e) = copy_bidirectional(&mut socket, &mut ssh_stream).await {
                            eprintln!("Forwarding error: {e}");
                        }
                    }
                }
            }
        }
    }

    async fn once_port_forward<T>(
        session: russh::client::Handle<Client>,
        remote_port: u32,
        local_port: u32,
        callable: T,
    ) -> Result<String, Box<dyn std::error::Error + Send>>
    where
        T: Future<Output = Result<(), Box<dyn std::error::Error + Send>>> + Send,
    {
        let local_listener = TcpListener::bind(("127.0.0.1", local_port as u16))
            .await
            .expect("Cannot bind local port");

        let mut ssh_channel = session
            .channel_open_direct_tcpip("127.0.0.1", remote_port, "127.0.0.1", local_port)
            .await
            .expect("Cannot open SSH forwarding channel");
        let mut ssh_stream = ssh_channel.into_stream();

        let forward_task = tokio::spawn(async move {
            if let Ok((mut socket, _)) = local_listener.accept().await
                && let Err(e) = copy_bidirectional(&mut socket, &mut ssh_stream).await
            {
                eprintln!("Forwarding error: {e}");
            }
        });

        callable.await?;
        match forward_task.await {
            Ok(_) => Ok(String::from("Success")),
            Err(e) => Err(Box::new(std::io::Error::other(format!("ERROR: {e}")))),
        }
    }
}
