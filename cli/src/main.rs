use anyhow::Result;
use clap::Parser;
use cli::{parse_retention, source_from_cli, storage_from_cli, Cli, Commands};
use vprs3bkp_core::{backup, list, restore, storage::storage::Storage};

mod cli;
mod tests;

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let cli = Cli::parse();

    // Process commands
    match &cli.command {
        Commands::Backup(args) => {
            let source_config = source_from_cli(&args.source)?;
            let storage_config = storage_from_cli(&args.storage)?;

            if let Some(retention) = &args.retention {
                let storage = Storage::new(&storage_config).await?;
                let (entries_deleted, storage_reclaimed) =
                    storage.cleanup(parse_retention(retention)?, false).await?;

                println!(
                    "{} Entries deleted, {} Storage reclaimed",
                    entries_deleted, storage_reclaimed
                );
            }

            let path = backup(&source_config, &storage_config).await?;
            println!("Backup completed successfully: {}", path);
        }

        Commands::Restore(args) => {
            let source_config = source_from_cli(&args.source)?;
            let storage_config = storage_from_cli(&args.storage)?;

            let drop_database = match args.drop_database {
                Some(value) => value,
                None => false,
            };

            if let Some(filename) = &args.filename {
                // Use the provided filename
                println!("Restoring from backup: {}", filename);
                restore(&source_config, &storage_config, filename, drop_database).await?;
            } else {
                return Err(anyhow::anyhow!(
                    "Either --filename or --latest must be specified for restore"
                ));
            }

            println!("Restore completed successfully");
        }

        Commands::List(args) => {
            let storage_config = storage_from_cli(&args.storage)?;

            let entries = crate::list(&storage_config).await?;
            let storage = Storage::new(&storage_config).await?;

            // Filter entries if database is specified
            let mut filtered_entries = if let Some(db_name) = &args.database {
                entries
                    .iter()
                    .filter(|e| {
                        let path = e.path();
                        let filename = storage.get_filename_from_path(path);
                        filename.contains(db_name)
                    })
                    .collect::<Vec<_>>()
            } else {
                entries.iter().collect::<Vec<_>>()
            };

            // Sort by path (most recent first)
            filtered_entries.sort_by(|a, b| b.path().cmp(a.path()));

            if args.latest_only && !filtered_entries.is_empty() {
                // For latest only, we can just take the first entry since they're already sorted
                filtered_entries = vec![filtered_entries[0]];
            }

            // Limit number of results
            let limited_entries = filtered_entries.iter().take(args.limit);

            println!("Available backups:");
            for entry in limited_entries {
                let path = entry.path();
                let filename = storage.get_filename_from_path(path);
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

        Commands::Cleanup(args) => {
            let storage_config = storage_from_cli(&args.storage)?;
            let storage = Storage::new(&storage_config).await?;
            let (entries_deleted, storage_reclaimed) = storage
                .cleanup(parse_retention(&args.retention)?, args.dry_run)
                .await?;

            println!(
                "{} Entries deleted, {} Storage reclaimed",
                entries_deleted, storage_reclaimed
            );
        }
    }

    Ok(())
}
