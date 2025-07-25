use anyhow::Result;
use inquire::{Confirm, Password, Select, Text};
use vprs3bkp_core::{
    databases::{
        ssh_tunnel::{SshAuthMethod, SshTunnelConfig},
        ConnectionType, DatabaseConfig,
    },
    storage::provider::{LocalStorageConfig, S3StorageConfig, StorageConfig},
};

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

        let mut collection = self.workspace_manager.load()?;

        loop {
            let action = self.select_main_action(&collection)?;

            match action {
                MainAction::CreateWorkspace => {
                    let workspace = self.create_workspace_interactive().await?;
                    collection.add_workspace(workspace.clone());
                    collection.set_active(&workspace.name)?;
                    self.workspace_manager.save(&collection)?;
                    println!(
                        "[SUCCESS] Workspace '{}' created and activated!",
                        workspace.name
                    );
                }
                MainAction::UseWorkspace => {
                    if collection.workspaces.is_empty() {
                        println!("[ERROR] No workspaces available. Create one first.");
                        continue;
                    }
                    let workspace_name = self.select_workspace(&collection)?;
                    collection.set_active(&workspace_name)?;
                    self.workspace_manager.save(&collection)?;
                    println!("[SUCCESS] Switched to workspace '{}'", workspace_name);
                }
                MainAction::BackupDatabase => {
                    if let Some(workspace) = collection.get_active() {
                        self.run_backup(workspace).await?;
                    } else {
                        println!("[ERROR] No active workspace. Please create or select one first.");
                    }
                }
                MainAction::RestoreDatabase => {
                    if let Some(workspace) = collection.get_active() {
                        self.run_restore(workspace).await?;
                    } else {
                        println!("[ERROR] No active workspace. Please create or select one first.");
                    }
                }
                MainAction::ListBackups => {
                    if let Some(workspace) = collection.get_active() {
                        self.run_list(workspace).await?;
                    } else {
                        println!("[ERROR] No active workspace. Please create or select one first.");
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
            println!("Active workspace: {}", active_workspace.name);
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

        let workspace = Workspace {
            name,
            database: database_config,
            storage: storage_config,
            created_at: chrono::Utc::now().to_rfc3339(),
            last_used: None,
        };

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
            println!("[ERROR] No workspaces available.");
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
                    println!("  - {}{}", workspace.name, active_marker);
                }
            }
            WorkspaceAction::Delete => {
                let workspace_name = self.select_workspace(collection)?;
                let confirm = Confirm::new(&format!("Delete workspace '{}'?", workspace_name))
                    .with_default(false)
                    .prompt()?;

                if confirm {
                    collection.remove_workspace(&workspace_name);
                    self.workspace_manager.save(collection)?;
                    println!("[SUCCESS] Workspace '{}' deleted", workspace_name);
                }
            }
            WorkspaceAction::Back => {}
        }

        Ok(())
    }

    async fn run_backup(&self, workspace: &Workspace) -> Result<()> {
        use vprs3bkp_core::{
            databases::DatabaseConnection, storage::provider::StorageProvider, DbBkp,
        };

        println!(
            "[INFO] Starting backup for workspace '{}'...",
            workspace.name
        );

        let database_connection = DatabaseConnection::new(workspace.database.clone()).await?;
        let storage_provider = StorageProvider::new(workspace.storage.clone())?;

        let core = DbBkp::new(database_connection, storage_provider);

        // Test connections
        core.test().await?;

        let backup_file = core.backup().await?;

        println!("[SUCCESS] Backup completed successfully: {}", backup_file);
        Ok(())
    }

    async fn run_restore(&self, workspace: &Workspace) -> Result<()> {
        use vprs3bkp_core::{
            databases::DatabaseConnection,
            storage::provider::{ListOptions, StorageProvider},
            DbBkp, RestoreOptions,
        };

        println!(
            "[INFO] Starting restore for workspace '{}'...",
            workspace.name
        );

        let database_connection = DatabaseConnection::new(workspace.database.clone()).await?;
        let storage_provider = StorageProvider::new(workspace.storage.clone())?;

        // List available backups
        let entries = storage_provider
            .list_with_options(ListOptions {
                latest_only: Some(false),
                limit: Some(20),
            })
            .await?;

        if entries.is_empty() {
            println!("[ERROR] No backups found in storage");
            return Ok(());
        }

        let backup_names: Vec<String> = entries.iter().map(|e| e.name().to_string()).collect();
        let selected_backup = Select::new("Select backup to restore:", backup_names).prompt()?;

        let drop_database = Confirm::new("Drop database before restore?")
            .with_default(false)
            .with_help_message("This will delete all existing data in the database")
            .prompt()?;

        let core = DbBkp::new(database_connection, storage_provider);

        // Test connections
        core.test().await?;

        core.restore(RestoreOptions {
            name: selected_backup.clone(),
            compression_format: None,
            drop_database_first: Some(drop_database),
        })
        .await?;

        println!(
            "[SUCCESS] Restore completed successfully: {}",
            selected_backup
        );
        Ok(())
    }

    async fn run_list(&self, workspace: &Workspace) -> Result<()> {
        use vprs3bkp_core::storage::provider::{ListOptions, StorageProvider};

        println!(
            "[INFO] Listing backups for workspace '{}'...",
            workspace.name
        );

        let storage_provider = StorageProvider::new(workspace.storage.clone())?;
        storage_provider.test().await?;

        let entries = storage_provider
            .list_with_options(ListOptions {
                latest_only: Some(false),
                limit: Some(50),
            })
            .await?;

        if entries.is_empty() {
            println!("[ERROR] No backups found");
            return Ok(());
        }

        println!("\nAvailable backups:");
        for entry in entries {
            let filename = entry.name();
            let size = entry.metadata().content_length();
            let size_str = if size < 1024 {
                format!("{}B", size)
            } else if size < 1024 * 1024 {
                format!("{:.2}KB", size as f64 / 1024.0)
            } else if size < 1024 * 1024 * 1024 {
                format!("{:.2}MB", size as f64 / (1024.0 * 1024.0))
            } else {
                format!("{:.2}GB", size as f64 / (1024.0 * 1024.0 * 1024.0))
            };

            println!("  - {} ({})", filename, size_str);
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
