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
use serde::{Deserialize, Serialize};
use ssh2::{ErrorCode, Session};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshTunnelConfig {
    pub ssh_host: String,
    pub ssh_port: u16,
    pub ssh_username: String,
    pub ssh_auth_method: SshAuthMethod,
    pub remote_host: String,
    pub remote_port: u16,
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
    pub fn new(config: SshTunnelConfig) -> Result<Self> {
        let shutdown_signal = Arc::new(AtomicBool::new(false));
        let local_port = Self::find_available_port()?;
        let (setup_tx, setup_rx) = channel();

        let thread_handle = {
            let config = config.clone();
            let shutdown_signal = shutdown_signal.clone();

            thread::spawn(move || {
                let _ = Self::run_tunnel(config, local_port, setup_tx, shutdown_signal);
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
        config: SshTunnelConfig,
        local_port: u16,
        setup_tx: Sender<Result<()>>,
        shutdown_signal: Arc<AtomicBool>,
    ) {
        let tcp = match TcpStream::connect(format!("{}:{}", config.ssh_host, config.ssh_port)) {
            Ok(tcp) => tcp,
            Err(e) => {
                let _ = setup_tx.send(Err(anyhow!("Failed to connect to SSH server: {}", e)));
                shutdown_signal.store(true, Ordering::Relaxed);
                return;
            }
        };

        let mut session = match Session::new() {
            Ok(session) => session,
            Err(e) => {
                let _ = setup_tx.send(Err(anyhow!("Failed to create SSH session: {}", e)));
                shutdown_signal.store(true, Ordering::Relaxed);
                return;
            }
        };

        session.set_tcp_stream(tcp);

        if let Err(e) = session.handshake() {
            let _ = setup_tx.send(Err(anyhow!("SSH handshake failed: {}", e)));
            shutdown_signal.store(true, Ordering::Relaxed);
            return;
        };

        match &config.ssh_auth_method {
            SshAuthMethod::Password { password } => {
                if let Err(e) = session.userauth_password(&config.ssh_username, &password) {
                    let _ =
                        setup_tx.send(Err(anyhow!("SSH password authentication failed: {}", e)));
                    shutdown_signal.store(true, Ordering::Relaxed);
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
                    &config.ssh_username,
                    None,
                    std::path::Path::new(key_path),
                    passphrase_key,
                ) {
                    let _ = setup_tx.send(Err(anyhow!("SSH key authentication failed: {}", e)));
                    shutdown_signal.store(true, Ordering::Relaxed);
                    return;
                };
            }
        }

        let listener = match TcpListener::bind(format!("127.0.0.1:{}", local_port)) {
            Ok(listener) => listener,
            Err(e) => {
                let _ = setup_tx.send(Err(anyhow!("Failed to bind to local port: {}", e)));
                shutdown_signal.store(true, Ordering::Relaxed);
                return;
            }
        };

        if let Err(e) = listener.set_nonblocking(true) {
            let _ = setup_tx.send(Err(anyhow!("Failed to set non-blocking mode: {}", e)));
            shutdown_signal.store(true, Ordering::Relaxed);
            return;
        };

        let mut connection_threads: Vec<JoinHandle<()>> = Vec::new();

        let _ = setup_tx.send(Ok(()));

        // Set all subsequent channels as non blocking

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
                    let remote_host_clone = config.remote_host.clone();
                    let remote_port_clone = config.remote_port.clone();

                    let connection_thread =
                        thread::Builder::new().name(thread_name).spawn(move || {
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
                                            shutdown_signal,
                                        ) {
                                            eprintln!("Error copy loop: {}", e);
                                            // log::warn!("Data forwarding error for client {}: {}. Connection terminated.", peer_addr, e);
                                        } else {
                                            // log::debug!("Client connection {} handled and closed gracefully.", peer_addr);
                                        }

                                        break;
                                    }
                                    Err(e) if e.code() == ErrorCode::Session(-37) => {
                                        // Would block
                                        thread::sleep(Duration::from_millis(5));
                                        continue;
                                    }
                                    Err(e) => {
                                        eprintln!("{}", e);
                                        break;
                                    }
                                }
                            }
                        });

                    match connection_thread {
                        Ok(handle) => connection_threads.push(handle),
                        Err(e) => {}
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(5));
                    continue;
                }
                Err(e) => {
                    // log::error!("Failed to accept local connection: {}. Listener might be broken.", e);
                    // Depending on the error, we might want to break the main loop.
                    // For now, log and continue, hoping it's transient.
                    // If it's a persistent error (e.g. listener closed), the loop will likely exit.
                    thread::sleep(Duration::from_millis(100)); // Back off a bit on persistent errors
                }
            }
        }

        for handle in connection_threads {
            if let Err(e) = handle.join() {}
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
            let mut activity_this_iteration = false;

            if shutdown_signal.load(Ordering::Relaxed) {
                break;
            }

            if !local_eof_reached {
                match local_stream.read(&mut local_buffer) {
                    Ok(0) => {
                        local_eof_reached = true;
                        activity_this_iteration = true;
                        if let Err(e) = channel.send_eof() {}
                    }
                    Ok(n) => {
                        activity_this_iteration = true;

                        if let Err(e) = channel.write_all(&local_buffer[..n]) {
                            return Err(anyhow!("Error writing to SSH channel: {}", e));
                        }

                        if let Err(e) = channel.flush() {
                            eprintln!("{}", e);
                        }
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(50));
                    }
                    Err(e) => {
                        println!("{}", e);
                        local_eof_reached = true;
                        let _ = channel.send_eof();
                    }
                }
            }

            if !remote_eof_reached {
                match channel.read(&mut remote_buffer) {
                    Ok(0) => {
                        remote_eof_reached = true;
                        activity_this_iteration = true;
                        if let Err(e) = local_stream.shutdown(Shutdown::Write) {}
                    }
                    Ok(n) => {
                        activity_this_iteration = true;

                        if let Err(e) = local_stream.write_all(&remote_buffer[..n]) {
                            return Err(anyhow!("Error writing to local stream: {}", e));
                        }
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(5));
                    }
                    Err(e) => {
                        println!("Read error {}", e);

                        if channel.eof() {
                            // Explicit EOF from ssh2 error code
                            // log::debug!("[{}] SSH channel EOF (via error code).", peer_addr);
                            remote_eof_reached = true;
                            activity_this_iteration = true;
                            let _ = local_stream.shutdown(Shutdown::Write);
                        } else {
                            remote_eof_reached = true;
                            let _ = local_stream.shutdown(Shutdown::Write);
                        }
                    }
                }
            }

            if local_eof_reached && remote_eof_reached {
                break;
            }

            if local_eof_reached && channel.eof() {
                break;
            }

            // if !activity_this_iteration {
            // }

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
        self.shutdown_signal.store(true, Ordering::Relaxed);

        if let Some(handle) = self.thread_handle.take() {
            if let Err(e) = handle.join() {
                eprintln!(
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
        ssh_tunnel::{SshAuthMethod, SshTunnel, SshTunnelConfig},
        ConnectionType, DatabaseConfig, DatabaseConnectionTrait,
    };

    #[tokio::test]
    async fn test_01_run_tunnel() {
        dotenv().ok();

        let remote_port: u16 = env::var("POSTGRESQL_PORT")
            .unwrap_or("0".into())
            .parse()
            .expect("Unable to parse remote port");

        let ssh_config = SshTunnelConfig {
            remote_host: "localhost".into(),
            remote_port,
            ssh_host: "188.213.129.133".into(),
            ssh_port: 22,
            ssh_username: "ubuntu".into(),
            ssh_auth_method: SshAuthMethod::PrivateKey {
                key_path: "/home/pietro/projects/laseris/laseris.pem".into(),
                passphrase_key: None,
            },
        };

        let tunnel = SshTunnel::new(ssh_config).expect("Failed to get ssh tunnel");
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

        println!("is connected: {}", is_connected);
    }
}
