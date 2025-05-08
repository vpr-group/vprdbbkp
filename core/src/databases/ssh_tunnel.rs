use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::mpsc::channel,
    thread::{self, JoinHandle},
};

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use ssh2::Session;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshTunnelConfig {
    pub ssh_host: String,
    pub ssh_port: u16,
    pub ssh_username: String,
    pub ssh_auth_method: SshAuthMethod,
    pub remote_host: String,
    pub remote_port: u16,
}

/// SSH authentication methods
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
    thread_handle: Option<JoinHandle<()>>,
}

impl SshTunnel {
    pub fn new(config: SshTunnelConfig) -> Result<Self> {
        let local_port = Self::find_available_port()?;
        let (status_tx, status_rx) = channel();

        let thread_handle = thread::spawn({
            let config = config.clone();
            move || {
                let result = Self::run_tunnel(config, local_port);
                let _ = status_tx.send(result);
            }
        });

        let status = status_rx.recv()?;
        match status {
            Ok(()) => {}
            Err(e) => return Err(anyhow!("Failed to start ssh tunnel: {}", e.to_string())),
        }

        Ok(Self {
            thread_handle: Some(thread_handle),
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

    fn run_tunnel(config: SshTunnelConfig, local_port: u16) -> Result<()> {
        let tcp = TcpStream::connect(format!("{}:{}", config.ssh_host, config.ssh_port))
            .map_err(|e| anyhow!("Failed to connect to SSH server: {}", e))?;

        let mut session =
            Session::new().map_err(|e| anyhow!("Failed to create SSH session: {}", e))?;

        session.set_tcp_stream(tcp);
        session
            .handshake()
            .map_err(|e| anyhow!("SSH handshake failed: {}", e))?;

        match &config.ssh_auth_method {
            SshAuthMethod::Password { password } => {
                session
                    .userauth_password(&config.ssh_username, &password)
                    .map_err(|e| anyhow!("SSH password authentication failed: {}", e))?;
            }
            SshAuthMethod::PrivateKey {
                key_path,
                passphrase_key,
            } => {
                let passphrase_key = match passphrase_key {
                    Some(key) => Some(key.as_str()),
                    None => None,
                };

                session
                    .userauth_pubkey_file(
                        &config.ssh_username,
                        None,
                        std::path::Path::new(key_path),
                        passphrase_key,
                    )
                    .map_err(|e| anyhow!("SSH key authentication failed: {}", e))?;
            }
        }

        let listener = TcpListener::bind(format!("127.0.0.1:{}", local_port))
            .map_err(|e| anyhow!("Failed to bind to local port: {}", e))?;

        listener
            .set_nonblocking(true)
            .map_err(|e| anyhow!("Failed to set non-blocking mode: {}", e))?;

        for stream in listener.incoming() {
            match stream {
                Ok(mut local_stream) => {
                    let session_clone = session.clone();

                    match session_clone.channel_direct_tcpip(
                        &config.remote_host,
                        config.remote_port,
                        None,
                    ) {
                        Ok(mut channel) => {
                            thread::spawn(move || {
                                // Forward data between local stream and SSH channel
                                // This is simplified - you'd need proper handling of both directions
                                let mut buffer = [0; 1024];
                                loop {
                                    match local_stream.read(&mut buffer) {
                                        Ok(0) => break, // Connection closed
                                        Ok(n) => {
                                            if channel.write_all(&buffer[..n]).is_err() {
                                                break;
                                            }
                                        }
                                        Err(_) => break,
                                    }

                                    match channel.read(&mut buffer) {
                                        Ok(0) => break, // Connection closed
                                        Ok(n) => {
                                            if local_stream.write_all(&buffer[..n]).is_err() {
                                                break;
                                            }
                                        }
                                        Err(_) => break,
                                    }
                                }
                            });
                        }
                        Err(e) => eprintln!("Failed to open direct TCP/IP channel: {}", e),
                    }
                }
                Err(e) => eprintln!("Failed to accept connection: {}", e),
            }
        }

        Ok(())
    }
}

impl Drop for SshTunnel {
    fn drop(&mut self) {
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }
}

#[cfg(test)]
mod ssh_tunnel_tests {
    use crate::databases::ssh_tunnel::{SshAuthMethod, SshTunnel, SshTunnelConfig};

    #[tokio::test]
    async fn test_01_run_tunnel() {
        let config = SshTunnelConfig {
            remote_host: "localhost".into(),
            remote_port: 5432,
            ssh_host: "188.213.129.133".into(),
            ssh_port: 22,
            ssh_username: "ubuntu".into(),
            ssh_auth_method: SshAuthMethod::PrivateKey {
                key_path: "/home/pietro/projects/laseris/laseris.pem".into(),
                passphrase_key: None,
            },
        };

        let _ = SshTunnel::new(config).expect("Failed to get ssh tunnel");
    }
}
