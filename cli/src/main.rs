use anyhow::{anyhow, Result};
use clap::Parser;
use cli::{
    database_config_from_cli, parse_retention, storage_from_cli, Cli, Commands, WorkspaceCommands,
};
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
            let (database_config, storage_config) = resolve_configs_for_backup(&args).await?;

            let database_connection = DatabaseConnection::new(database_config).await?;
            let storage_provider = StorageProvider::new(storage_config)?;

            let core = DbBkp::new(database_connection, storage_provider);

            // Test database & storage connection
            core.test().await?;

            let backup_file = core.backup().await?;

            println!("Backup completed successfully: {}", backup_file);
        }
        Commands::List(args) => {
            let storage_config = resolve_storage_config(&args.workspace, &args.storage).await?;
            let storage_provider = StorageProvider::new(storage_config)?;
            storage_provider.test().await?;

            let entries = storage_provider
                .list_with_options(ListOptions {
                    latest_only: Some(args.latest_only),
                    limit: args.limit,
                })
                .await?;

            println!("Available backups:");

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

                println!("  {} ({})", filename, size_str);
            }
        }
        Commands::Restore(args) => {
            let (database_config, storage_config) = resolve_configs_for_restore(&args).await?;
            let backup_name = resolve_backup_name(&args, &storage_config).await?;

            let database_connection = DatabaseConnection::new(database_config).await?;
            let storage_provider = StorageProvider::new(storage_config)?;

            let core = DbBkp::new(database_connection, storage_provider);

            // Test database & storage connection
            core.test().await?;

            core.restore(RestoreOptions {
                name: backup_name.clone(),
                compression_format: None,
                drop_database_first: Some(args.drop_database),
            })
            .await?;

            println!("Restore completed successfully: {}", backup_name);
        }
        Commands::Cleanup(args) => {
            let storage_config = resolve_storage_config(&args.workspace, &args.storage).await?;
            let storage = StorageProvider::new(storage_config)?;

            // Test storage connection
            storage.test().await?;

            let (entries_deleted, storage_reclaimed) = storage
                .cleanup(parse_retention(&args.retention)?, args.dry_run)
                .await?;

            println!(
                "{} Entries deleted, {} Storage reclaimed",
                entries_deleted, storage_reclaimed
            );
        }
    };

    Ok(())
}

async fn handle_workspace_command(command: WorkspaceCommands) -> Result<()> {
    let workspace_manager = WorkspaceManager::new()?;
    let mut collection = workspace_manager.load()?;

    match command {
        WorkspaceCommands::List => {
            if collection.workspaces.is_empty() {
                println!("No workspaces found.");
            } else {
                println!("Available workspaces:");
                for workspace in collection.list_workspaces() {
                    let active_marker =
                        if Some(&workspace.name) == collection.active_workspace.as_ref() {
                            " (active)"
                        } else {
                            ""
                        };
                    println!("  • {}{}", workspace.name, active_marker);
                }
            }
        }
        WorkspaceCommands::Create { name: _ } => {
            println!("Interactive workspace creation not implemented yet.");
            println!("Use 'dbkp interactive' for guided workspace setup.");
        }
        WorkspaceCommands::Delete { name } => {
            if collection.remove_workspace(&name).is_some() {
                workspace_manager.save(&collection)?;
                println!("Workspace '{}' deleted.", name);
            } else {
                println!("Workspace '{}' not found.", name);
            }
        }
        WorkspaceCommands::Use { name } => {
            if collection.set_active(&name).is_ok() {
                workspace_manager.save(&collection)?;
                println!("Switched to workspace '{}'.", name);
            } else {
                println!("Workspace '{}' not found.", name);
            }
        }
        WorkspaceCommands::Active => {
            if let Some(workspace) = collection.get_active() {
                println!("Active workspace: {}", workspace.name);
            } else {
                println!("No active workspace set.");
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
        let database_config = args
            .database_config
            .as_ref()
            .ok_or_else(|| anyhow!("Database configuration required when not using workspace"))?;
        let storage_config = args
            .storage_config
            .as_ref()
            .ok_or_else(|| anyhow!("Storage configuration required when not using workspace"))?;
        Ok((
            database_config_from_cli(database_config)?,
            storage_from_cli(storage_config)?,
        ))
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
        let database_config = args
            .database_config
            .as_ref()
            .ok_or_else(|| anyhow!("Database configuration required when not using workspace"))?;
        let storage_config = args
            .storage_config
            .as_ref()
            .ok_or_else(|| anyhow!("Storage configuration required when not using workspace"))?;
        Ok((
            database_config_from_cli(database_config)?,
            storage_from_cli(storage_config)?,
        ))
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
        let storage_config = storage_args
            .as_ref()
            .ok_or_else(|| anyhow!("Storage configuration required when not using workspace"))?;
        storage_from_cli(storage_config)
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
