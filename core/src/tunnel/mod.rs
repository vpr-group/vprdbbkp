use std::{
    process::{Child, Command},
    time::Duration,
};

use anyhow::{anyhow, Result};
use config::TunnelConfig;
use tokio::{
    net::{TcpListener, TcpStream},
    time::sleep,
};

use crate::databases::configs::SourceConfig;

pub mod config;

pub struct Tunnel {
    config: TunnelConfig,
    process: Option<Child>,
    local_port: Option<u16>,
}

impl Tunnel {
    pub fn new(config: TunnelConfig) -> Self {
        Tunnel {
            config,
            process: None,
            local_port: None,
        }
    }

    async fn find_available_port(&self) -> Result<u16> {
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let port = listener.local_addr()?.port();
        Ok(port)
    }

    pub async fn establish_tunnel(&mut self, source_config: &SourceConfig) -> Result<u16> {
        let local_port = self.find_available_port().await?;

        let remote_host = match source_config {
            SourceConfig::PG(config) => &config.host,
        };

        let remote_port = match source_config {
            SourceConfig::PG(config) => config.port,
        };

        let username = match source_config {
            SourceConfig::PG(config) => &config.username,
        };

        // Build the SSH command - using standard Command since TokioCommand doesn't
        // work well for long-running processes like SSH tunnels
        let mut command = Command::new("ssh");
        command
            .arg("-i")
            .arg(&self.config.key_path)
            .arg("-L")
            .arg(format!(
                "127.0.0.1:{}:{}:{}",
                local_port, remote_host, remote_port
            ))
            .arg("-N")
            .arg(format!("{}@{}", username, remote_host))
            // Add options for a stable connection
            .arg("-o")
            .arg("StrictHostKeyChecking=no")
            .arg("-o")
            .arg("ExitOnForwardFailure=yes");

        let process = command.spawn()?;
        self.process = Some(process);
        self.local_port = Some(local_port);

        // Wait a moment for the tunnel to establish
        sleep(Duration::from_secs(1)).await;

        match TcpStream::connect(format!("127.0.0.1:{}", local_port)).await {
            Ok(_) => {
                println!("SSH tunnel established successfully on port {}", local_port);
                Ok(local_port)
            }
            Err(e) => {
                self.close_tunnel().await?;
                Err(anyhow!("Failed to establish SSH tunnel: {}", e))
            }
        }
    }

    pub fn get_local_port(&self) -> Option<u16> {
        self.local_port
    }

    pub async fn close_tunnel(&mut self) -> Result<()> {
        if let Some(mut process) = self.process.take() {
            match process.kill() {
                Ok(_) => {
                    println!("SSH tunnel closed successfully");
                    self.local_port = None;
                    Ok(())
                }
                Err(e) => Err(anyhow!("Failed to close SSH tunnel: {}", e)),
            }
        } else {
            Ok(())
        }
    }

    pub fn get_tunneled_config(&self, source_config: &SourceConfig) -> Option<SourceConfig> {
        if let Some(local_port) = self.local_port {
            match source_config {
                SourceConfig::PG(pg_config) => {
                    let mut tunneled_config = pg_config.clone();
                    tunneled_config.host = "127.0.0.1".to_string();
                    tunneled_config.port = local_port;

                    Some(SourceConfig::PG(tunneled_config))
                }
            }
        } else {
            None
        }
    }
}

impl Drop for Tunnel {
    fn drop(&mut self) {
        if self.process.is_some() {
            let _ = self.close_tunnel();
        }
    }
}
