use anyhow::{anyhow, Result};
use colored::*;
use dbkp_core::{
    databases::{
        ssh_tunnel::{SshAuthMethod, SshTunnelConfig},
        ConnectionType, DatabaseConfig,
    },
    storage::provider::{LocalStorageConfig, S3StorageConfig, StorageConfig},
};
use inquire::{Confirm, Password, Select, Text};

use crate::spinner::Spinner;
use crate::workspace::{Workspace, WorkspaceCollection, WorkspaceManager};

pub struct InteractiveSetup {
    workspace_manager: WorkspaceManager,
}

impl InteractiveSetup {
    pub fn new() -> Result<Self> {
        Ok(Self {
            workspace_manager: WorkspaceManager::new()?,
        })
    }

    pub async fn run(&self) -> Result<()> {
        println!("Welcome to dbkp interactive mode!");
        println!();

        let mut spinner = Spinner::new("Loading workspaces...");
        spinner.start();

        let mut collection = match self.workspace_manager.load() {
            Ok(collection) => {
                spinner.stop();
                collection
            }
            Err(e) => {
                spinner.error("Failed to load workspaces");
                return Err(e);
            }
        };

        loop {
            let action = self.select_main_action(&collection)?;

            match action {
                MainAction::CreateWorkspace => {
                    let workspace = self.create_workspace_interactive().await?;
                    collection.add_workspace(workspace.clone());
                    collection.set_active(&workspace.name)?;

                    let mut spinner = Spinner::new("Saving workspace configuration...");
                    spinner.start();
                    match self.workspace_manager.save(&collection) {
                        Ok(_) => {
                            spinner.success(format!(
                                "Workspace '{}' created and activated!",
                                workspace.name.green().bold()
                            ));
                        }
                        Err(e) => {
                            spinner.error("Failed to save workspace configuration");
                            return Err(e);
                        }
                    }
                }
                MainAction::UseWorkspace => {
                    if collection.workspaces.is_empty() {
                        println!(
                            "{}",
                            "[ERROR] No workspaces available. Create one first.".red()
                        );
                        continue;
                    }
                    let workspace_name = self.select_workspace(&collection)?;
                    collection.set_active(&workspace_name)?;

                    let mut spinner = Spinner::new("Switching workspace...");
                    spinner.start();
                    match self.workspace_manager.save(&collection) {
                        Ok(_) => {
                            spinner.success(format!(
                                "Switched to workspace '{}'",
                                workspace_name.green().bold()
                            ));
                        }
                        Err(e) => {
                            spinner.error("Failed to save workspace configuration");
                            return Err(e);
                        }
                    }
                }
                MainAction::BackupDatabase => {
                    if let Some(workspace) = collection.get_active() {
                        self.run_backup(workspace).await?;
                    } else {
                        println!(
                            "{}",
                            "[ERROR] No active workspace. Please create or select one first.".red()
                        );
                    }
                }
                MainAction::RestoreDatabase => {
                    if let Some(workspace) = collection.get_active() {
                        self.run_restore(workspace).await?;
                    } else {
                        println!(
                            "{}",
                            "[ERROR] No active workspace. Please create or select one first.".red()
                        );
                    }
                }
                MainAction::ListBackups => {
                    if let Some(workspace) = collection.get_active() {
                        self.run_list(workspace).await?;
                    } else {
                        println!(
                            "{}",
                            "[ERROR] No active workspace. Please create or select one first.".red()
                        );
                    }
                }
                MainAction::ManageWorkspaces => {
                    self.manage_workspaces(&mut collection).await?;
                }
                MainAction::Exit => {
                    println!("Goodbye!");
                    break;
                }
            }

            println!();
        }

        Ok(())
    }

    fn select_main_action(&self, collection: &WorkspaceCollection) -> Result<MainAction> {
        let mut options = vec![MainAction::CreateWorkspace, MainAction::ManageWorkspaces];

        if !collection.workspaces.is_empty() {
            options.insert(1, MainAction::UseWorkspace);
        }

        if collection.get_active().is_some() {
            let active_workspace = collection.get_active().unwrap();
            println!("Active workspace: {}", active_workspace.name.green().bold());
            options.extend_from_slice(&[
                MainAction::BackupDatabase,
                MainAction::RestoreDatabase,
                MainAction::ListBackups,
            ]);
        }

        options.push(MainAction::Exit);

        let action = Select::new("What would you like to do?", options).prompt()?;
        Ok(action)
    }

    async fn create_workspace_interactive(&self) -> Result<Workspace> {
        println!("Creating a new workspace...");
        println!();

        let name = Text::new("Workspace name:")
            .with_help_message("Choose a descriptive name for this workspace")
            .prompt()?;

        println!();
        println!("Database Configuration");
        let database_config = self.setup_database_interactive().await?;

        println!();
        println!("Storage Configuration");
        let storage_config = self.setup_storage_interactive().await?;

        let mut spinner = Spinner::new("Configuring workspace...");
        spinner.start();

        let workspace = Workspace {
            name,
            database: database_config,
            storage: storage_config,
            created_at: chrono::Utc::now().to_rfc3339(),
            last_used: None,
        };

        spinner.stop();
        Ok(workspace)
    }

    async fn setup_database_interactive(&self) -> Result<DatabaseConfig> {
        let db_type = Select::new(
            "Database type:",
            vec![DatabaseType::PostgreSQL, DatabaseType::MySQL],
        )
        .prompt()?;

        let host = Text::new("Host:").with_default("localhost").prompt()?;

        let port = Text::new("Port:")
            .with_default(&match db_type {
                DatabaseType::PostgreSQL => "5432".to_string(),
                DatabaseType::MySQL => "3306".to_string(),
            })
            .prompt()?
            .parse::<u16>()?;

        let database = Text::new("Database name:")
            .with_help_message("The name of the database to backup/restore")
            .prompt()?;

        let username = Text::new("Username:").prompt()?;

        let password = Password::new("Password:")
            .with_help_message("Leave empty if no password required")
            .without_confirmation()
            .prompt_skippable()?;

        let use_ssh = Confirm::new("Use SSH tunnel?")
            .with_default(false)
            .prompt()?;

        let ssh_tunnel = if use_ssh {
            Some(self.setup_ssh_tunnel_interactive()?)
        } else {
            None
        };

        Ok(DatabaseConfig {
            connection_type: match db_type {
                DatabaseType::PostgreSQL => ConnectionType::PostgreSql,
                DatabaseType::MySQL => ConnectionType::MySql,
            },
            database: database.clone(),
            id: "".into(),
            name: database,
            host,
            port,
            username,
            password,
            ssh_tunnel,
        })
    }

    fn setup_ssh_tunnel_interactive(&self) -> Result<SshTunnelConfig> {
        let host = Text::new("SSH Host:").prompt()?;

        let username = Text::new("SSH Username:").prompt()?;

        let key_path = Text::new("SSH Private Key Path:")
            .with_help_message("Path to your SSH private key file")
            .prompt()?;

        Ok(SshTunnelConfig {
            port: 22,
            host,
            username,
            auth_method: SshAuthMethod::PrivateKey {
                key_path,
                passphrase_key: None,
            },
        })
    }

    async fn setup_storage_interactive(&self) -> Result<StorageConfig> {
        let storage_type =
            Select::new("Storage type:", vec![StorageType::Local, StorageType::S3]).prompt()?;

        let name = Text::new("Storage name:")
            .with_default("default")
            .prompt()?;

        let location = Text::new("Location:")
            .with_help_message("Directory path for local storage or prefix for S3")
            .with_default("backups")
            .prompt()?;

        match storage_type {
            StorageType::Local => Ok(StorageConfig::Local(LocalStorageConfig {
                name,
                id: "".into(),
                location,
            })),
            StorageType::S3 => {
                let bucket = Text::new("S3 Bucket:").prompt()?;

                let region = Text::new("S3 Region:").with_default("us-east-1").prompt()?;

                let endpoint = Text::new("S3 Endpoint:")
                    .with_help_message("Custom S3 endpoint (optional for AWS)")
                    .prompt_skippable()?;

                let access_key = Text::new("Access Key ID:").prompt()?;

                let secret_key = Password::new("Secret Access Key:")
                    .without_confirmation()
                    .prompt()?;

                Ok(StorageConfig::S3(S3StorageConfig {
                    name,
                    bucket,
                    region,
                    endpoint,
                    access_key,
                    secret_key,
                    location,
                    id: "".into(),
                }))
            }
        }
    }

    fn select_workspace(&self, collection: &WorkspaceCollection) -> Result<String> {
        let workspaces: Vec<_> = collection.list_workspaces();
        let workspace_names: Vec<String> = workspaces.iter().map(|w| w.name.clone()).collect();

        let selected = Select::new("Select workspace:", workspace_names).prompt()?;
        Ok(selected)
    }

    async fn manage_workspaces(&self, collection: &mut WorkspaceCollection) -> Result<()> {
        if collection.workspaces.is_empty() {
            println!("{}", "[ERROR] No workspaces available.".red());
            return Ok(());
        }

        let action = Select::new(
            "Workspace management:",
            vec![
                WorkspaceAction::List,
                WorkspaceAction::Delete,
                WorkspaceAction::Back,
            ],
        )
        .prompt()?;

        match action {
            WorkspaceAction::List => {
                println!("\nAvailable workspaces:");
                for workspace in collection.list_workspaces() {
                    let active_marker =
                        if Some(&workspace.name) == collection.active_workspace.as_ref() {
                            " (active)"
                        } else {
                            ""
                        };
                    if active_marker.is_empty() {
                        println!("  - {}", workspace.name);
                    } else {
                        println!(
                            "  - {} {}",
                            workspace.name.green().bold(),
                            "(active)".green()
                        );
                    }
                }
            }
            WorkspaceAction::Delete => {
                let workspace_name = self.select_workspace(collection)?;
                let confirm = Confirm::new(&format!("Delete workspace '{}'?", workspace_name))
                    .with_default(false)
                    .prompt()?;

                if confirm {
                    collection.remove_workspace(&workspace_name);

                    let mut spinner = Spinner::new("Deleting workspace...");
                    spinner.start();
                    match self.workspace_manager.save(collection) {
                        Ok(_) => {
                            spinner.success(format!(
                                "Workspace '{}' deleted",
                                workspace_name.green().bold()
                            ));
                        }
                        Err(e) => {
                            spinner.error("Failed to save workspace configuration");
                            return Err(e);
                        }
                    }
                }
            }
            WorkspaceAction::Back => {}
        }

        Ok(())
    }

    async fn run_backup(&self, workspace: &Workspace) -> Result<()> {
        use dbkp_core::{databases::DatabaseConnection, storage::provider::StorageProvider, DbBkp};

        let mut spinner = Spinner::new(format!(
            "Starting backup for workspace '{}'...",
            workspace.name
        ));
        spinner.start();

        let database_connection = match DatabaseConnection::new(workspace.database.clone()).await {
            Ok(conn) => {
                spinner.update_message("Database connection established, connecting to storage...");
                conn
            }
            Err(e) => {
                spinner.error("Failed to connect to database");
                return Err(e);
            }
        };

        let storage_provider = match StorageProvider::new(workspace.storage.clone()) {
            Ok(provider) => {
                spinner.update_message("Storage connection established, testing connections...");
                provider
            }
            Err(e) => {
                spinner.error("Failed to connect to storage");
                return Err(e);
            }
        };

        let core = DbBkp::new(database_connection, storage_provider);

        // Test connections
        match core.test().await {
            Ok(_) => spinner.update_message("Connections verified, starting backup..."),
            Err(e) => {
                spinner.error("Connection test failed");
                return Err(e);
            }
        }

        match core.backup().await {
            Ok(file) => {
                spinner.success(format!("Backup completed successfully: {}", file));
            }
            Err(e) => {
                spinner.error("Backup failed");
                return Err(e);
            }
        }
        Ok(())
    }

    async fn run_restore(&self, workspace: &Workspace) -> Result<()> {
        use dbkp_core::{
            common::extract_timestamp_from_filename,
            databases::DatabaseConnection,
            storage::provider::{ListOptions, StorageProvider},
            DbBkp, RestoreOptions,
        };

        let mut spinner = Spinner::new(format!(
            "Preparing restore for workspace '{}'...",
            workspace.name
        ));
        spinner.start();

        let database_connection = match DatabaseConnection::new(workspace.database.clone()).await {
            Ok(conn) => {
                spinner.update_message("Database connection established, connecting to storage...");
                conn
            }
            Err(e) => {
                spinner.error("Failed to connect to database");
                return Err(e);
            }
        };

        let storage_provider = match StorageProvider::new(workspace.storage.clone()) {
            Ok(provider) => {
                spinner.update_message("Storage connection established, fetching backup list...");
                provider
            }
            Err(e) => {
                spinner.error("Failed to connect to storage");
                return Err(e);
            }
        };

        // List available backups
        let entries = match storage_provider
            .list_with_options(ListOptions {
                latest_only: Some(false),
                limit: Some(50),
            })
            .await
        {
            Ok(entries) => {
                spinner.stop();
                entries
            }
            Err(e) => {
                spinner.error("Failed to fetch backup list");
                return Err(e);
            }
        };

        if entries.is_empty() {
            println!("{}", "[ERROR] No backups found in storage".red());
            return Ok(());
        }

        println!("\nAvailable backups (newest first):");

        // Create formatted backup options for selection
        let mut backup_options: Vec<String> = Vec::new();
        let mut backup_names: Vec<String> = Vec::new();

        for (index, entry) in entries.iter().enumerate() {
            let filename = &entry.metadata.name;
            let size = entry.metadata.content_length;

            // Format file size
            let size_str = if size < 1024 {
                format!("{}B", size)
            } else if size < 1024 * 1024 {
                format!("{:.2}KB", size as f64 / 1024.0)
            } else if size < 1024 * 1024 * 1024 {
                format!("{:.2}MB", size as f64 / (1024.0 * 1024.0))
            } else {
                format!("{:.2}GB", size as f64 / (1024.0 * 1024.0 * 1024.0))
            };

            // Try to extract and format timestamp
            let date_str = match extract_timestamp_from_filename(filename) {
                Ok(timestamp) => timestamp.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
                Err(_) => "Unknown date".to_string(),
            };

            // Create display string with index, date, size, and filename
            let display_option = format!(
                "{:2}. {} | {} | {}",
                index + 1,
                date_str,
                size_str,
                filename
            );

            backup_options.push(display_option);
            backup_names.push(filename.to_string());
        }

        let selected_display =
            Select::new("Select backup to restore:", backup_options.clone()).prompt()?;

        // Find the actual backup name based on the selected display string
        let selected_backup = backup_names
            .iter()
            .enumerate()
            .find(|(i, _)| backup_options[*i] == selected_display)
            .map(|(_, name)| name.clone())
            .ok_or_else(|| anyhow!("Failed to find selected backup"))?;

        let drop_database = Confirm::new("Drop database before restore?")
            .with_default(false)
            .with_help_message("This will delete all existing data in the database")
            .prompt()?;

        let mut spinner = Spinner::new("Testing connections...");
        spinner.start();

        let core = DbBkp::new(database_connection, storage_provider);

        // Test connections
        match core.test().await {
            Ok(_) => {
                spinner.update_message(format!("Starting restore of '{}'...", selected_backup))
            }
            Err(e) => {
                spinner.error("Connection test failed");
                return Err(e);
            }
        }

        match core
            .restore(RestoreOptions {
                name: selected_backup.clone(),
                compression_format: None,
                drop_database_first: Some(drop_database),
            })
            .await
        {
            Ok(_) => {
                spinner.success(format!(
                    "Restore completed successfully: {}",
                    selected_backup
                ));
            }
            Err(e) => {
                spinner.error("Restore failed");
                return Err(e);
            }
        }
        Ok(())
    }

    async fn run_list(&self, workspace: &Workspace) -> Result<()> {
        use dbkp_core::{
            common::extract_timestamp_from_filename,
            storage::provider::{ListOptions, StorageProvider},
        };

        let mut spinner = Spinner::new(format!(
            "Fetching backup list for workspace '{}'...",
            workspace.name
        ));
        spinner.start();

        let storage_provider = match StorageProvider::new(workspace.storage.clone()) {
            Ok(provider) => {
                spinner.update_message("Storage connection established, testing connection...");
                provider
            }
            Err(e) => {
                spinner.error("Failed to connect to storage");
                return Err(e);
            }
        };

        match storage_provider.test().await {
            Ok(_) => spinner.update_message("Connection verified, fetching backup list..."),
            Err(e) => {
                spinner.error("Storage connection test failed");
                return Err(e);
            }
        }

        let entries = match storage_provider
            .list_with_options(ListOptions {
                latest_only: Some(false),
                limit: Some(50),
            })
            .await
        {
            Ok(entries) => {
                spinner.stop();
                entries
            }
            Err(e) => {
                spinner.error("Failed to fetch backup list");
                return Err(e);
            }
        };

        if entries.is_empty() {
            println!("{}", "[ERROR] No backups found".red());
            return Ok(());
        }

        println!("\nAvailable backups (newest first):");
        for (index, entry) in entries.iter().enumerate() {
            let filename = &entry.metadata.name;
            let size = entry.metadata.content_length;

            // Format file size
            let size_str = if size < 1024 {
                format!("{}B", size)
            } else if size < 1024 * 1024 {
                format!("{:.2}KB", size as f64 / 1024.0)
            } else if size < 1024 * 1024 * 1024 {
                format!("{:.2}MB", size as f64 / (1024.0 * 1024.0))
            } else {
                format!("{:.2}GB", size as f64 / (1024.0 * 1024.0 * 1024.0))
            };

            // Try to extract and format timestamp
            let date_str = match extract_timestamp_from_filename(filename) {
                Ok(timestamp) => timestamp.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
                Err(_) => "Unknown date".to_string(),
            };

            // Display formatted backup info
            println!(
                "  {:2}. {} | {} | {}",
                index + 1,
                date_str,
                size_str,
                filename
            );
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
enum MainAction {
    CreateWorkspace,
    UseWorkspace,
    BackupDatabase,
    RestoreDatabase,
    ListBackups,
    ManageWorkspaces,
    Exit,
}

impl std::fmt::Display for MainAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MainAction::CreateWorkspace => write!(f, "Create new workspace"),
            MainAction::UseWorkspace => write!(f, "Switch workspace"),
            MainAction::BackupDatabase => write!(f, "Backup database"),
            MainAction::RestoreDatabase => write!(f, "Restore database"),
            MainAction::ListBackups => write!(f, "List backups"),
            MainAction::ManageWorkspaces => write!(f, "Manage workspaces"),
            MainAction::Exit => write!(f, "Exit"),
        }
    }
}

#[derive(Debug, Clone)]
enum DatabaseType {
    PostgreSQL,
    MySQL,
}

impl std::fmt::Display for DatabaseType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DatabaseType::PostgreSQL => write!(f, "PostgreSQL"),
            DatabaseType::MySQL => write!(f, "MySQL"),
        }
    }
}

#[derive(Debug, Clone)]
enum StorageType {
    Local,
    S3,
}

impl std::fmt::Display for StorageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageType::Local => write!(f, "Local filesystem"),
            StorageType::S3 => write!(f, "Amazon S3 / Compatible"),
        }
    }
}

#[derive(Debug, Clone)]
enum WorkspaceAction {
    List,
    Delete,
    Back,
}

impl std::fmt::Display for WorkspaceAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkspaceAction::List => write!(f, "List workspaces"),
            WorkspaceAction::Delete => write!(f, "Delete workspace"),
            WorkspaceAction::Back => write!(f, "Back to main menu"),
        }
    }
}
