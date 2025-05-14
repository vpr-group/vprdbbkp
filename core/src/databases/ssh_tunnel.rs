use std::{
    io::{Read, Write},
    net::{Shutdown, TcpListener, TcpStream},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{channel, Sender},
        Arc,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use anyhow::{anyhow, Context, Result};
use log::{debug, error, info, trace, warn};
use serde::{Deserialize, Serialize};
use ssh2::{ErrorCode, Session};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshTunnelConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub auth_method: SshAuthMethod,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshRemoteConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SshAuthMethod {
    Password {
        password: String,
    },
    PrivateKey {
        key_path: String,
        passphrase_key: Option<String>,
    },
}

pub struct SshTunnel {
    pub local_port: u16,
    shutdown_signal: Arc<AtomicBool>,
    thread_handle: Option<JoinHandle<()>>,
}

impl SshTunnel {
    pub fn new(ssh_config: SshTunnelConfig, remote_config: SshRemoteConfig) -> Result<Self> {
        let shutdown_signal = Arc::new(AtomicBool::new(false));
        let local_port = Self::find_available_port()?;
        let (setup_tx, setup_rx) = channel();

        let thread_handle = {
            let ssh_config = ssh_config.clone();
            let remote_config = remote_config.clone();
            let shutdown_signal = shutdown_signal.clone();

            thread::spawn(move || {
                Self::run_tunnel(
                    ssh_config,
                    remote_config,
                    local_port,
                    setup_tx,
                    shutdown_signal,
                );
            })
        };

        let status = setup_rx.recv()?;
        match status {
            Ok(()) => {}
            Err(e) => return Err(anyhow!("Failed to start ssh tunnel: {}", e.to_string())),
        }

        Ok(Self {
            thread_handle: Some(thread_handle),
            shutdown_signal,
            local_port,
        })
    }

    fn find_available_port() -> Result<u16> {
        let listener = TcpListener::bind("127.0.0.1:0")
            .map_err(|e| anyhow!("Failed to find available port: {}", e))?;

        let port = listener
            .local_addr()
            .map_err(|e| anyhow!("Failed to get local address: {}", e))?
            .port();

        Ok(port)
    }

    fn run_tunnel(
        ssh_config: SshTunnelConfig,
        remote_config: SshRemoteConfig,
        local_port: u16,
        setup_tx: Sender<Result<()>>,
        shutdown_signal: Arc<AtomicBool>,
    ) {
        let tcp = match TcpStream::connect(format!("{}:{}", ssh_config.host, ssh_config.port)) {
            Ok(tcp) => tcp,
            Err(e) => {
                shutdown_signal.store(true, Ordering::Relaxed);
                if let Err(e) =
                    setup_tx.send(Err(anyhow!("Failed to create TCP connection: {}", e)))
                {
                    warn!("Failed to send setup message: {}", e);
                };
                return;
            }
        };

        trace!("TCP connection established with SSH server");

        let mut session = match Session::new() {
            Ok(session) => session,
            Err(e) => {
                shutdown_signal.store(true, Ordering::Relaxed);
                if let Err(e) = setup_tx.send(Err(anyhow!("Failed to create SSH session: {}", e))) {
                    warn!("Failed to send setup message: {}", e);
                };
                return;
            }
        };

        trace!("SSH session created");

        session.set_tcp_stream(tcp);

        if let Err(e) = session.handshake() {
            shutdown_signal.store(true, Ordering::Relaxed);
            if let Err(e) = setup_tx.send(Err(anyhow!("SSH handshake failed: {}", e))) {
                warn!("Failed to send setup message: {}", e);
            };
            return;
        };

        trace!("SSH handshake successful");

        match &ssh_config.auth_method {
            SshAuthMethod::Password { password } => {
                if let Err(e) = session.userauth_password(&ssh_config.username, &password) {
                    shutdown_signal.store(true, Ordering::Relaxed);
                    if let Err(e) =
                        setup_tx.send(Err(anyhow!("SSH password authentication failed: {}", e)))
                    {
                        warn!("Failed to send setup message: {}", e);
                    };
                    return;
                };
            }
            SshAuthMethod::PrivateKey {
                key_path,
                passphrase_key,
            } => {
                let passphrase_key = match passphrase_key {
                    Some(key) => Some(key.as_str()),
                    None => None,
                };

                if let Err(e) = session.userauth_pubkey_file(
                    &ssh_config.username,
                    None,
                    std::path::Path::new(key_path),
                    passphrase_key,
                ) {
                    shutdown_signal.store(true, Ordering::Relaxed);
                    if let Err(e) =
                        setup_tx.send(Err(anyhow!("SSH key authentication failed: {}", e)))
                    {
                        warn!("Failed to send setup message: {}", e);
                    };
                    return;
                };
            }
        }

        let listener = match TcpListener::bind(format!("127.0.0.1:{}", local_port)) {
            Ok(listener) => listener,
            Err(e) => {
                shutdown_signal.store(true, Ordering::Relaxed);
                if let Err(e) = setup_tx.send(Err(anyhow!("Failed to bind to local port: {}", e))) {
                    warn!("Failed to send setup message: {}", e);
                };
                return;
            }
        };

        trace!("TCP listener created");

        if let Err(e) = listener.set_nonblocking(true) {
            shutdown_signal.store(true, Ordering::Relaxed);
            if let Err(e) = setup_tx.send(Err(anyhow!("Failed to set non-blocking mode: {}", e))) {
                warn!("Failed to send setup message: {}", e);
            };
            return;
        };

        trace!("TCP listener set to non blocking");

        session.set_blocking(false);

        trace!("SSH session set to non blocking");

        if let Err(e) = setup_tx.send(Ok(())) {
            warn!("Failed to send setup message: {}", e);
        };

        drop(setup_tx);

        debug!("SSH tunnel setup successful");

        let mut connection_threads: Vec<JoinHandle<()>> = Vec::new();
        for stream in listener.incoming() {
            if shutdown_signal.load(Ordering::Relaxed) {
                break;
            }

            match stream {
                Ok(local_stream) => {
                    let shutdown_signal = shutdown_signal.clone();
                    let session_clone = session.clone();
                    let peer_addr = local_stream
                        .peer_addr()
                        .map_or_else(|_| "unknown".to_string(), |a| a.to_string());

                    let thread_name = format!("ssh_fwd_{}", peer_addr);
                    let remote_host_clone = remote_config.host.clone();
                    let remote_port_clone = remote_config.port.clone();

                    let connection_thread =
                        thread::Builder::new().name(thread_name.clone()).spawn(move || {
                            info!("Local connection thread created: {}", thread_name);

                            loop {
                                match session_clone.channel_direct_tcpip(
                                    &remote_host_clone,
                                    remote_port_clone,
                                    None,
                                ) {
                                    Ok(channel) => {
                                        if let Err(e) = Self::copy_loop(
                                            local_stream,
                                            channel,
                                            &peer_addr,
                                            shutdown_signal.clone(),
                                        ) {
                                            error!("Data forwarding error for client {}: {}. Connection terminated.", peer_addr, e);
                                            shutdown_signal.store(true, Ordering::Relaxed);
                                        } else {
                                            debug!("Client connection {} handled and closed gracefully.", peer_addr);
                                        }

                                        break;
                                    }
                                    Err(e) if e.code() == ErrorCode::Session(-37) => {
                                        trace!("Error while getting SSH channel: would block");
                                        thread::sleep(Duration::from_millis(10));
                                        continue;
                                    }
                                    Err(e) => {
                                        error!("Failed to get direct tcp ip connection {}", e);
                                        shutdown_signal.store(true, Ordering::Relaxed);
                                        break;
                                    }
                                }
                            }
                        });

                    match connection_thread {
                        Ok(handle) => connection_threads.push(handle),
                        Err(e) => {
                            error!("Error while creating connection threads: {}", e);
                        }
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(5));
                    continue;
                }
                Err(e) => {
                    error!("Failed to get stream: {}", e);
                    shutdown_signal.store(true, Ordering::Relaxed);
                    break;
                }
            }
        }

        for handle in connection_threads {
            if let Err(_) = handle.join() {
                error!("Failed to join connection thread");
            }
        }
    }

    fn copy_loop(
        mut local_stream: TcpStream,
        mut channel: ssh2::Channel,
        peer_addr: &str,
        shutdown_signal: Arc<AtomicBool>,
    ) -> Result<()> {
        local_stream.set_nonblocking(true).with_context(|| {
            format!(
                "Failed to set local stream (client: {}) to non-blocking",
                peer_addr
            )
        })?;

        let mut local_buffer = vec![0; 8192];
        let mut remote_buffer = vec![0; 8192];

        let mut local_eof_reached = false;
        let mut remote_eof_reached = false;

        loop {
            if shutdown_signal.load(Ordering::Relaxed) {
                break;
            }

            if !local_eof_reached {
                match local_stream.read(&mut local_buffer) {
                    Ok(0) => {
                        local_eof_reached = true;
                        if let Err(e) = channel.send_eof() {
                            warn!("Failed to set channel EOF: {}", e);
                        }
                    }
                    Ok(n) => {
                        if let Err(e) = channel.write_all(&local_buffer[..n]) {
                            return Err(anyhow!("Error writing to SSH channel: {}", e));
                        }

                        if let Err(e) = channel.flush() {
                            warn!("Failed to flush channel: {}", e);
                        }
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(e) => {
                        warn!("Failed to read from local stream: {}", e);
                        local_eof_reached = true;
                        if let Err(e) = channel.send_eof() {
                            warn!("Failed to set channel EOF: {}", e);
                        }
                    }
                }
            }

            if !remote_eof_reached {
                match channel.read(&mut remote_buffer) {
                    Ok(0) => {
                        remote_eof_reached = true;
                        if let Err(e) = local_stream.shutdown(Shutdown::Write) {
                            warn!("Failed to shutdown local stream: {}", e);
                        }
                    }
                    Ok(n) => {
                        if let Err(e) = local_stream.write_all(&remote_buffer[..n]) {
                            return Err(anyhow!("Error writing to local stream: {}", e));
                        }
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(5));
                    }
                    Err(e) => {
                        warn!("Failed to read from local stream: {}", e);
                        remote_eof_reached = true;
                        if let Err(e) = local_stream.shutdown(Shutdown::Write) {
                            warn!("Failed to shutdown local stream: {}", e);
                        }
                    }
                }
            }

            if local_eof_reached && remote_eof_reached {
                break;
            }

            thread::sleep(Duration::from_millis(5));
        }

        if !local_eof_reached {
            channel.send_eof().ok();
        }

        channel.close().ok();
        local_stream.shutdown(Shutdown::Both).ok();
        Ok(())
    }
}

impl Drop for SshTunnel {
    fn drop(&mut self) {
        debug!("Dropping SSH tunnel");
        self.shutdown_signal.store(true, Ordering::Relaxed);
        if let Some(handle) = self.thread_handle.take() {
            if let Err(e) = handle.join() {
                error!(
                    "Error joining SshTunnel main thread for local port {}: {:?}",
                    self.local_port, e
                );
            }
        }
    }
}

#[cfg(test)]
mod ssh_tunnel_tests {

    use dotenv::dotenv;
    use std::env;

    use crate::databases::{
        postgres::connection::PostgreSqlConnection,
        ssh_tunnel::{SshAuthMethod, SshRemoteConfig, SshTunnel, SshTunnelConfig},
        ConnectionType, DatabaseConfig, DatabaseConnectionTrait,
    };

    #[ignore]
    #[tokio::test]
    async fn test_01_run_tunnel() {
        dotenv().ok();
        env_logger::builder()
            .filter_level(log::LevelFilter::Debug)
            .is_test(true)
            .try_init()
            .ok();

        let ssh_config = SshTunnelConfig {
            host: env::var("SSH_HOST").unwrap_or_default(),
            username: env::var("SSH_USERNAME").unwrap_or_default(),
            port: 22,
            auth_method: SshAuthMethod::PrivateKey {
                key_path: env::var("SSH_KEY_PATH").unwrap_or_default(),
                passphrase_key: None,
            },
        };

        let remote_port: u16 = env::var("POSTGRESQL_PORT")
            .unwrap_or("0".into())
            .parse()
            .expect("Unable to parse remote port");

        let ssh_remote_config = SshRemoteConfig {
            host: "localhost".into(),
            port: remote_port,
        };

        let tunnel =
            SshTunnel::new(ssh_config, ssh_remote_config).expect("Failed to get ssh tunnel");
        let password = env::var("DB_PASSWORD").unwrap_or_default();

        let database_config = DatabaseConfig {
            id: "test".into(),
            name: "test".into(),
            connection_type: ConnectionType::PostgreSql,
            host: "localhost".into(),
            port: tunnel.local_port,
            username: env::var("DB_USERNAME").unwrap_or_default(),
            database: env::var("DB_NAME").unwrap_or_default(),
            password: Some(password),
            ssh_tunnel: None,
        };

        let postgres_connection = PostgreSqlConnection::new(database_config)
            .await
            .expect("Unable to get postgresql connection");

        let is_connected = postgres_connection
            .test()
            .await
            .expect("Failed to test database connection");

        assert!(is_connected);
    }
}
