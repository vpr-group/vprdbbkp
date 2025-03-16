// use anyhow::{Context, Result};
// use aws_sdk_s3::{primitives::ByteStream, Client as S3Client};
// use chrono::Utc;
// use globset::{Glob, GlobSet, GlobSetBuilder};
// use log::{debug, info, warn};
// use std::path::Path;
// use std::sync::Arc;
// use tokio::fs::File;
// use tokio::io::AsyncReadExt;
// use tokio::sync::Semaphore;
// use tokio::task;
// use walkdir::WalkDir;

// /// Statistics about the backup process
// pub struct BackupStats {
//     pub total_files: usize,
//     pub files_processed: usize,
//     pub files_skipped: usize,
//     pub files_failed: usize,
//     pub total_bytes: u64,
// }

// /// Process a folder and upload its contents to S3
// pub async fn backup_folder(
//     client: &S3Client,
//     bucket: &str,
//     prefix: &str,
//     folder_path: &str,
//     compress: bool,
//     compression_level: u8,
//     concurrency: u8,
//     include_patterns: Option<Vec<String>>,
//     exclude_patterns: Option<Vec<String>>,
//     skip_larger_than: Option<u32>,
//     add_timestamp: bool,
// ) -> Result<BackupStats> {
//     // Validate the folder path
//     let folder_path = Path::new(folder_path);
//     if !folder_path.exists() {
//         return Err(anyhow::anyhow!(
//             "Folder does not exist: {}",
//             folder_path.display()
//         ));
//     }
//     if !folder_path.is_dir() {
//         return Err(anyhow::anyhow!(
//             "Path is not a directory: {}",
//             folder_path.display()
//         ));
//     }

//     // Prepare S3 prefix with optional timestamp
//     let s3_prefix = if add_timestamp {
//         let now = Utc::now();
//         let date_str = now.format("%Y-%m-%d-%H%M%S");
//         format!("{}/{}", prefix, date_str)
//     } else {
//         prefix.to_string()
//     };

//     info!("Starting backup of folder: {}", folder_path.display());
//     info!("Target: s3://{}/{}", bucket, s3_prefix);

//     // Compile include/exclude glob patterns
//     let include_set = build_glob_set(include_patterns)?;
//     let exclude_set = build_glob_set(exclude_patterns)?;

//     // Calculate max file size in bytes if specified
//     let max_file_size = skip_larger_than.map(|size| size as u64 * 1024 * 1024);

//     // List all files in the directory recursively
//     let mut files = Vec::new();
//     for entry in WalkDir::new(folder_path) {
//         let entry = entry?;
//         if entry.file_type().is_file() {
//             files.push(entry.path().to_path_buf());
//         }
//     }

//     info!("Found {} files to process", files.len());

//     // Set up concurrency control
//     let concurrency = concurrency.clamp(1, 100) as usize;
//     let semaphore = Arc::new(Semaphore::new(concurrency));

//     // Initialize statistics
//     let mut stats = BackupStats {
//         total_files: files.len(),
//         files_processed: 0,
//         files_skipped: 0,
//         files_failed: 0,
//         total_bytes: 0,
//     };

//     // Process files with controlled concurrency
//     let mut tasks = Vec::new();

//     for file_path in files {
//         // Apply filters
//         if should_skip_file(
//             &file_path,
//             folder_path,
//             max_file_size,
//             &include_set,
//             &exclude_set,
//         )? {
//             stats.files_skipped += 1;
//             continue;
//         }

//         // Get relative path for S3 key
//         let rel_path = file_path
//             .strip_prefix(folder_path)
//             .unwrap_or(&file_path)
//             .to_string_lossy()
//             .replace("\\", "/"); // Normalize path separators for S3

//         let s3_key = format!("{}/{}", s3_prefix, rel_path);

//         // Clone references for async task
//         let semaphore = Arc::clone(&semaphore);
//         let bucket = bucket.to_string();
//         let file_path_clone = file_path.clone();
//         let client = client.clone();

//         // Spawn task for this file
//         let task = task::spawn(async move {
//             // Acquire semaphore permit
//             let _permit = semaphore.acquire().await.unwrap();

//             let result = process_file(
//                 &client,
//                 &bucket,
//                 &s3_key,
//                 &file_path_clone,
//                 compress,
//                 compression_level,
//             )
//             .await;

//             match result {
//                 Ok(size) => (true, size),
//                 Err(e) => {
//                     warn!("Failed to upload {}: {}", file_path_clone.display(), e);
//                     (false, 0)
//                 }
//             }
//         });

//         tasks.push(task);
//     }

//     // Wait for all tasks to complete
//     for task in tasks {
//         match task.await {
//             Ok((success, size)) => {
//                 if success {
//                     stats.files_processed += 1;
//                     stats.total_bytes += size;
//                 } else {
//                     stats.files_failed += 1;
//                 }
//             }
//             Err(e) => {
//                 warn!("Task join error: {}", e);
//                 stats.files_failed += 1;
//             }
//         }
//     }

//     info!("Backup complete: {} files processed, {} files skipped, {} files failed, {} bytes transferred",
//         stats.files_processed, stats.files_skipped, stats.files_failed, stats.total_bytes);

//     Ok(stats)
// }

// /// Check if a file should be skipped based on filters
// fn should_skip_file(
//     file_path: &Path,
//     base_path: &Path,
//     max_size: Option<u64>,
//     include_set: &Option<GlobSet>,
//     exclude_set: &Option<GlobSet>,
// ) -> Result<bool> {
//     // Check file size if limit is set
//     if let Some(max_size) = max_size {
//         let metadata = std::fs::metadata(file_path)?;
//         if metadata.len() > max_size {
//             debug!(
//                 "Skipping large file: {} ({} bytes)",
//                 file_path.display(),
//                 metadata.len()
//             );
//             return Ok(true);
//         }
//     }

//     // Get relative path for pattern matching
//     let rel_path = file_path
//         .strip_prefix(base_path)
//         .unwrap_or(file_path)
//         .to_string_lossy();

//     // Check exclude patterns
//     if let Some(exclude_set) = exclude_set {
//         if exclude_set.is_match(&*rel_path) {
//             debug!("Skipping excluded file: {}", rel_path);
//             return Ok(true);
//         }
//     }

//     // Check include patterns if specified
//     if let Some(include_set) = include_set {
//         if !include_set.is_match(&*rel_path) {
//             debug!("Skipping non-included file: {}", rel_path);
//             return Ok(true);
//         }
//     }

//     Ok(false)
// }

// /// Process a single file and upload it to S3
// async fn process_file(
//     client: &S3Client,
//     bucket: &str,
//     s3_key: &str,
//     file_path: &Path,
//     compress: bool,
//     compression_level: u8,
// ) -> Result<u64> {
//     debug!("Processing file: {}", file_path.display());

//     let mut file = File::open(file_path).await?;
//     let mut contents = Vec::new();
//     let bytes_read = file.read_to_end(&mut contents).await?;

//     // Compress if requested
//     // if compress {
//     //     debug!("Compressing file with level {}", compression_level);
//     //     let mut encoder = flate2::write::GzEncoder::new(
//     //         Vec::new(),
//     //         flate2::Compression::new(compression_level.into()),
//     //     );
//     //     std::io::Write::write_all(&mut encoder, &contents)?;
//     //     contents = encoder.finish()?;

//     //     // Add .gz extension if not already present
//     //     if !s3_key.ends_with(".gz") {
//     //         let s3_key = format!("{}.gz", s3_key);
//     //         debug!("Uploading to s3://{}/{}", bucket, s3_key);
//     //         upload_to_s3(client, bucket, &s3_key, ByteStream::from(contents.clone())).await?;
//     //     } else {
//     //         debug!("Uploading to s3://{}/{}", bucket, s3_key);
//     //         upload_to_s3(client, bucket, s3_key, ByteStream::from(contents.clone())).await?;
//     //     }
//     // } else {
//     //     debug!("Uploading to s3://{}/{}", bucket, s3_key);
//     //     upload_to_s3(client, bucket, s3_key, ByteStream::from(contents.clone())).await?;
//     // }

//     Ok(bytes_read as u64)
// }

// /// Build a GlobSet from a vector of glob patterns
// fn build_glob_set(patterns: Option<Vec<String>>) -> Result<Option<GlobSet>> {
//     if let Some(patterns) = patterns {
//         if patterns.is_empty() {
//             return Ok(None);
//         }

//         let mut builder = GlobSetBuilder::new();
//         for pattern in patterns {
//             let glob = Glob::new(&pattern).context(format!("Invalid glob pattern: {}", pattern))?;
//             builder.add(glob);
//         }

//         let glob_set = builder.build().context("Failed to build glob set")?;
//         Ok(Some(glob_set))
//     } else {
//         Ok(None)
//     }
// }
