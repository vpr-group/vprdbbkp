use anyhow::{anyhow, Result};
use clap::Parser;
use cli::{
    database_config_from_cli, parse_retention, storage_from_cli, Cli, Commands, WorkspaceCommands,
};
use colored::*;
use vprs3bkp_core::{
    databases::DatabaseConnection,
    storage::provider::{ListOptions, StorageProvider},
    DbBkp, RestoreOptions,
};

mod cli;
mod interactive;
mod spinner;
mod tests;
mod workspace;

use interactive::InteractiveSetup;
use spinner::Spinner;
use workspace::WorkspaceManager;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command.unwrap_or(Commands::Interactive) {
        Commands::Interactive => {
            let interactive = InteractiveSetup::new()?;
            interactive.run().await?;
        }
        Commands::Workspace { command } => {
            handle_workspace_command(command).await?;
        }
        Commands::Backup(args) => {
            let mut spinner = Spinner::new("Resolving configuration...");
            spinner.start();

            let (database_config, storage_config) = match resolve_configs_for_backup(&args).await {
                Ok(configs) => {
                    spinner.update_message("Configuration resolved, connecting to database...");
                    configs
                }
                Err(e) => {
                    spinner.error("Failed to resolve configuration");
                    return Err(e);
                }
            };

            let database_connection = match DatabaseConnection::new(database_config).await {
                Ok(conn) => {
                    spinner.update_message("Database connected, connecting to storage...");
                    conn
                }
                Err(e) => {
                    spinner.error("Failed to connect to database");
                    return Err(e);
                }
            };

            let storage_provider = match StorageProvider::new(storage_config) {
                Ok(provider) => {
                    spinner.update_message("Storage connected, testing connections...");
                    provider
                }
                Err(e) => {
                    spinner.error("Failed to connect to storage");
                    return Err(e);
                }
            };

            let core = DbBkp::new(database_connection, storage_provider);

            // Test database & storage connection
            match core.test().await {
                Ok(_) => spinner.update_message("Connections verified, starting backup..."),
                Err(e) => {
                    spinner.error("Connection test failed");
                    return Err(e);
                }
            }

            match core.backup().await {
                Ok(backup_file) => {
                    spinner.success(format!("Backup completed successfully: {}", backup_file));
                }
                Err(e) => {
                    spinner.error("Backup failed");
                    return Err(e);
                }
            }
        }
        Commands::List(args) => {
            let mut spinner = Spinner::new("Resolving storage configuration...");
            spinner.start();

            let storage_config =
                match resolve_storage_config(&args.workspace, &Some(args.storage)).await {
                    Ok(config) => {
                        spinner.update_message("Storage configuration resolved, connecting...");
                        config
                    }
                    Err(e) => {
                        spinner.error("Failed to resolve storage configuration");
                        return Err(e);
                    }
                };

            let storage_provider = match StorageProvider::new(storage_config) {
                Ok(provider) => {
                    spinner.update_message("Storage connected, testing connection...");
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
                    latest_only: Some(args.latest_only),
                    limit: args.limit,
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
                println!("{}", "[INFO] No backups found".cyan());
                return Ok(());
            }

            println!("\n{}:", "Available backups".green().bold());

            for (index, entry) in entries.iter().enumerate() {
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

                // Try to extract and format timestamp
                let date_str =
                    match vprs3bkp_core::common::extract_timestamp_from_filename(filename) {
                        Ok(timestamp) => timestamp.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
                        Err(_) => "Unknown date".to_string(),
                    };

                println!(
                    "  {:2}. {} | {} | {}",
                    index + 1,
                    date_str,
                    size_str,
                    filename
                );
            }
        }
        Commands::Restore(args) => {
            let mut spinner = Spinner::new("Resolving configuration...");
            spinner.start();

            let (database_config, storage_config) = match resolve_configs_for_restore(&args).await {
                Ok(configs) => {
                    spinner.update_message("Configuration resolved, determining backup name...");
                    configs
                }
                Err(e) => {
                    spinner.error("Failed to resolve configuration");
                    return Err(e);
                }
            };

            let backup_name = match resolve_backup_name(&args, &storage_config).await {
                Ok(name) => {
                    spinner.update_message("Backup identified, connecting to database...");
                    name
                }
                Err(e) => {
                    spinner.error("Failed to resolve backup name");
                    return Err(e);
                }
            };

            let database_connection = match DatabaseConnection::new(database_config).await {
                Ok(conn) => {
                    spinner.update_message("Database connected, connecting to storage...");
                    conn
                }
                Err(e) => {
                    spinner.error("Failed to connect to database");
                    return Err(e);
                }
            };

            let storage_provider = match StorageProvider::new(storage_config) {
                Ok(provider) => {
                    spinner.update_message("Storage connected, testing connections...");
                    provider
                }
                Err(e) => {
                    spinner.error("Failed to connect to storage");
                    return Err(e);
                }
            };

            let core = DbBkp::new(database_connection, storage_provider);

            // Test database & storage connection
            match core.test().await {
                Ok(_) => spinner.update_message(format!(
                    "Connections verified, starting restore of '{}'...",
                    backup_name
                )),
                Err(e) => {
                    spinner.error("Connection test failed");
                    return Err(e);
                }
            }

            match core
                .restore(RestoreOptions {
                    name: backup_name.clone(),
                    compression_format: None,
                    drop_database_first: Some(args.drop_database),
                })
                .await
            {
                Ok(_) => {
                    spinner.success(format!("Restore completed successfully: {}", backup_name));
                }
                Err(e) => {
                    spinner.error("Restore failed");
                    return Err(e);
                }
            }
        }
        Commands::Cleanup(args) => {
            let mut spinner = Spinner::new("Resolving storage configuration...");
            spinner.start();

            let storage_config =
                match resolve_storage_config(&args.workspace, &Some(args.storage)).await {
                    Ok(config) => {
                        spinner.update_message("Storage configuration resolved, connecting...");
                        config
                    }
                    Err(e) => {
                        spinner.error("Failed to resolve storage configuration");
                        return Err(e);
                    }
                };

            let storage = match StorageProvider::new(storage_config) {
                Ok(provider) => {
                    spinner.update_message("Storage connected, testing connection...");
                    provider
                }
                Err(e) => {
                    spinner.error("Failed to connect to storage");
                    return Err(e);
                }
            };

            // Test storage connection
            match storage.test().await {
                Ok(_) => {
                    let action = if args.dry_run {
                        "analyzing"
                    } else {
                        "cleaning up"
                    };
                    spinner.update_message(format!("Connection verified, {} backups...", action));
                }
                Err(e) => {
                    spinner.error("Storage connection test failed");
                    return Err(e);
                }
            }

            match storage
                .cleanup(parse_retention(&args.retention)?, args.dry_run)
                .await
            {
                Ok((entries_deleted, storage_reclaimed)) => {
                    if args.dry_run {
                        spinner.success(format!(
                            "Dry run completed: {} entries would be deleted, {} storage would be reclaimed",
                            entries_deleted, storage_reclaimed
                        ));
                    } else {
                        spinner.success(format!(
                            "Cleanup completed: {} entries deleted, {} storage reclaimed",
                            entries_deleted, storage_reclaimed
                        ));
                    }
                }
                Err(e) => {
                    spinner.error("Cleanup failed");
                    return Err(e);
                }
            }
        }
    };

    Ok(())
}

async fn handle_workspace_command(command: WorkspaceCommands) -> Result<()> {
    let mut spinner = Spinner::new("Loading workspaces...");
    spinner.start();

    let workspace_manager = match WorkspaceManager::new() {
        Ok(manager) => manager,
        Err(e) => {
            spinner.error("Failed to initialize workspace manager");
            return Err(e);
        }
    };

    let mut collection = match workspace_manager.load() {
        Ok(collection) => {
            spinner.stop();
            collection
        }
        Err(e) => {
            spinner.error("Failed to load workspaces");
            return Err(e);
        }
    };

    match command {
        WorkspaceCommands::List => {
            if collection.workspaces.is_empty() {
                println!("{}", "[INFO] No workspaces found.".cyan());
            } else {
                println!("\n{}:", "Available workspaces".green().bold());
                for workspace in collection.list_workspaces() {
                    let active_marker =
                        if Some(&workspace.name) == collection.active_workspace.as_ref() {
                            " (active)".green().to_string()
                        } else {
                            "".to_string()
                        };
                    if active_marker.is_empty() {
                        println!("  - {}", workspace.name);
                    } else {
                        println!("  - {} {}", workspace.name.green().bold(), active_marker);
                    }
                }
            }
        }
        WorkspaceCommands::Create { name: _ } => {
            println!("Interactive workspace creation not implemented yet.");
            println!("Use 'dbkp interactive' for guided workspace setup.");
        }
        WorkspaceCommands::Delete { name } => {
            if collection.remove_workspace(&name).is_some() {
                let mut spinner = Spinner::new("Deleting workspace...");
                spinner.start();
                match workspace_manager.save(&collection) {
                    Ok(_) => {
                        spinner.success(format!("Workspace '{}' deleted.", name.green().bold()));
                    }
                    Err(e) => {
                        spinner.error("Failed to save workspace configuration");
                        return Err(e);
                    }
                }
            } else {
                println!(
                    "{}",
                    format!("[ERROR] Workspace '{}' not found.", name).red()
                );
            }
        }
        WorkspaceCommands::Use { name } => {
            if collection.set_active(&name).is_ok() {
                let mut spinner = Spinner::new("Switching workspace...");
                spinner.start();
                match workspace_manager.save(&collection) {
                    Ok(_) => {
                        spinner
                            .success(format!("Switched to workspace '{}'.", name.green().bold()));
                    }
                    Err(e) => {
                        spinner.error("Failed to save workspace configuration");
                        return Err(e);
                    }
                }
            } else {
                println!(
                    "{}",
                    format!("[ERROR] Workspace '{}' not found.", name).red()
                );
            }
        }
        WorkspaceCommands::Active => {
            if let Some(workspace) = collection.get_active() {
                println!("Active workspace: {}", workspace.name.green().bold());
            } else {
                println!("{}", "[INFO] No active workspace set.".cyan());
            }
        }
    }

    Ok(())
}

async fn resolve_configs_for_backup(
    args: &cli::BackupArgs,
) -> Result<(
    vprs3bkp_core::databases::DatabaseConfig,
    vprs3bkp_core::storage::provider::StorageConfig,
)> {
    if let Some(workspace_name) = &args.workspace {
        let workspace_manager = WorkspaceManager::new()?;
        let collection = workspace_manager.load()?;
        let workspace = collection
            .get_workspace(workspace_name)
            .ok_or_else(|| anyhow!("Workspace '{}' not found", workspace_name))?;
        Ok((workspace.database.clone(), workspace.storage.clone()))
    } else {
        // Check if we have direct CLI parameters
        let database_config = if has_database_config(&args.database_config) {
            database_config_from_cli(&args.database_config)?
        } else {
            return Err(anyhow!(
                "Either --workspace or database configuration parameters are required.\n\
                Database parameters: --database-type, --database, --host, --port, --username\n\
                Use 'dbkp backup --help' for more details."
            ));
        };

        let storage_config = if has_storage_config(&args.storage_config) {
            storage_from_cli(&args.storage_config)?
        } else {
            return Err(anyhow!(
                "Either --workspace or storage configuration parameters are required.\n\
                Storage parameters: --storage-type, --location (and for S3: --bucket, --endpoint, --access-key, --secret-key)\n\
                Use 'dbkp backup --help' for more details."
            ));
        };

        Ok((database_config, storage_config))
    }
}

async fn resolve_configs_for_restore(
    args: &cli::RestoreArgs,
) -> Result<(
    vprs3bkp_core::databases::DatabaseConfig,
    vprs3bkp_core::storage::provider::StorageConfig,
)> {
    if let Some(workspace_name) = &args.workspace {
        let workspace_manager = WorkspaceManager::new()?;
        let collection = workspace_manager.load()?;
        let workspace = collection
            .get_workspace(workspace_name)
            .ok_or_else(|| anyhow!("Workspace '{}' not found", workspace_name))?;
        Ok((workspace.database.clone(), workspace.storage.clone()))
    } else {
        // Check if we have direct CLI parameters
        let database_config = if has_database_config(&args.database_config) {
            database_config_from_cli(&args.database_config)?
        } else {
            return Err(anyhow!(
                "Either --workspace or database configuration parameters are required.\n\
                Database parameters: --database-type, --database, --host, --port, --username\n\
                Use 'dbkp restore --help' for more details."
            ));
        };

        let storage_config = if has_storage_config(&args.storage_config) {
            storage_from_cli(&args.storage_config)?
        } else {
            return Err(anyhow!(
                "Either --workspace or storage configuration parameters are required.\n\
                Storage parameters: --storage-type, --location (and for S3: --bucket, --endpoint, --access-key, --secret-key)\n\
                Use 'dbkp restore --help' for more details."
            ));
        };

        Ok((database_config, storage_config))
    }
}

async fn resolve_storage_config(
    workspace_name: &Option<String>,
    storage_args: &Option<cli::StorageArgs>,
) -> Result<vprs3bkp_core::storage::provider::StorageConfig> {
    if let Some(workspace_name) = workspace_name {
        let workspace_manager = WorkspaceManager::new()?;
        let collection = workspace_manager.load()?;
        let workspace = collection
            .get_workspace(workspace_name)
            .ok_or_else(|| anyhow!("Workspace '{}' not found", workspace_name))?;
        Ok(workspace.storage.clone())
    } else {
        if let Some(storage_config) = storage_args {
            if has_storage_config(storage_config) {
                storage_from_cli(storage_config)
            } else {
                Err(anyhow!(
                    "Either --workspace or storage configuration parameters are required.\n\
                    Storage parameters: --storage-type, --location (and for S3: --bucket, --endpoint, --access-key, --secret-key)\n\
                    Use command --help for more details."
                ))
            }
        } else {
            Err(anyhow!(
                "Either --workspace or storage configuration parameters are required.\n\
                Storage parameters: --storage-type, --location (and for S3: --bucket, --endpoint, --access-key, --secret-key)\n\
                Use command --help for more details."
            ))
        }
    }
}

async fn resolve_backup_name(
    args: &cli::RestoreArgs,
    storage_config: &vprs3bkp_core::storage::provider::StorageConfig,
) -> Result<String> {
    if let Some(name) = &args.name {
        Ok(name.clone())
    } else if args.latest {
        // Get the latest backup
        let storage_provider = StorageProvider::new(storage_config.clone())?;
        let entries = storage_provider
            .list_with_options(ListOptions {
                latest_only: Some(true),
                limit: Some(1),
            })
            .await?;

        if let Some(entry) = entries.first() {
            Ok(entry.name().to_string())
        } else {
            Err(anyhow!("No backups found"))
        }
    } else {
        Err(anyhow!("Either --name or --latest must be specified"))
    }
}

fn has_database_config(args: &cli::DatabaseArgs) -> bool {
    args.database_type.is_some()
        && args.database.is_some()
        && args.host.is_some()
        && args.port.is_some()
        && args.username.is_some()
}

fn has_storage_config(args: &cli::StorageArgs) -> bool {
    // For local storage, we need at least location
    if args.storage_type.as_deref() == Some("local") || args.storage_type.is_none() {
        args.location.is_some()
    } else if args.storage_type.as_deref() == Some("s3") {
        // For S3, we need bucket, endpoint, access_key, secret_key, and location
        args.bucket.is_some()
            && args.endpoint.is_some()
            && args.access_key.is_some()
            && args.secret_key.is_some()
            && args.location.is_some()
    } else {
        false
    }
}
