use std::path::Path;
use std::fs;
use std::time::{Duration, Instant, SystemTime};
use anyhow::{Context, Result};
use rayon::prelude::*;
use crossbeam_channel::{bounded, Sender};

use crate::db::{Database, Index};

/// Scans a directory and collects file indices without metadata.
///
/// Uses parallel processing with work-stealing for efficient scanning.
/// Results are sent to the database buffer channel as they are discovered.
///
/// # Arguments
/// * `root` - Root directory to scan
/// * `db` - Database instance to write indices to
/// * `batch_size` - Number of indices to batch before writing (recommended: 1000-10000)
///
/// # Returns
/// Duration of the scan operation
pub fn scan_idxs<P: AsRef<Path>>(
    root: P,
    db: &Database,
    batch_size: usize,
) -> Result<Duration> {
    let start = Instant::now();
    let root = root.as_ref();
    
    if !root.exists() {
        anyhow::bail!("Root path does not exist: {}", root.display());
    }
    
    // Channel for collecting indices from parallel workers
    let (tx, rx) = bounded::<Index>(batch_size * 2);
    
    // Clone db for the writer thread
    let db_clone = db.clone();
    
    // Spawn writer thread to batch insert indices
    let writer_handle = std::thread::spawn(move || {
        write_indices_batched(rx, &db_clone, batch_size)
    });
    
    // Parallel scanning
    let scan_result = scan_directory_parallel(root, tx);
    
    // Wait for writer to finish
    let write_result = writer_handle.join()
        .map_err(|_| anyhow::anyhow!("Writer thread panicked"))?;
    
    scan_result?;
    write_result?;
    
    Ok(start.elapsed())
}

/// Scans a directory and collects file indices with metadata (mtime, size).
///
/// Uses parallel processing with work-stealing for efficient scanning.
/// Extracts file modification time and size from filesystem metadata.
///
/// # Arguments
/// * `root` - Root directory to scan
/// * `db` - Database instance to write indices to
/// * `batch_size` - Number of indices to batch before writing (recommended: 1000-10000)
///
/// # Returns
/// Duration of the scan operation
pub fn scan_idxs_with_metadata<P: AsRef<Path>>(
    root: P,
    db: &Database,
    batch_size: usize,
) -> Result<Duration> {
    let start = Instant::now();
    let root = root.as_ref();
    
    if !root.exists() {
        anyhow::bail!("Root path does not exist: {}", root.display());
    }
    
    let (tx, rx) = bounded::<Index>(batch_size * 2);
    let db_clone = db.clone();
    
    let writer_handle = std::thread::spawn(move || {
        write_indices_batched(rx, &db_clone, batch_size)
    });
    
    let scan_result = scan_directory_parallel_with_metadata(root, tx);
    
    let write_result = writer_handle.join()
        .map_err(|_| anyhow::anyhow!("Writer thread panicked"))?;
    
    scan_result?;
    write_result?;
    
    Ok(start.elapsed())
}

/// Recursively scans directory in parallel without metadata.
fn scan_directory_parallel<P: AsRef<Path>>(
    root: P,
    tx: Sender<Index>,
) -> Result<()> {
    let root = root.as_ref();
    
    // Read entries in current directory
    let entries: Vec<_> = fs::read_dir(root)
        .with_context(|| format!("Failed to read directory: {}", root.display()))?
        .filter_map(|e| e.ok())
        .collect();
    
    // Separate files and directories
    let (files, dirs): (Vec<_>, Vec<_>) = entries.into_iter()
        .partition(|entry| entry.path().is_file());
    
    // Process files in parallel
    files.par_iter().try_for_each(|entry| {
        let path = entry.path();
        let path_str = path.to_string_lossy().to_string();
        
        let name = entry.file_name()
            .to_string_lossy()
            .to_string();
        
        let idx = Index::new(path_str, name);
        
        tx.send(idx)
            .map_err(|_| anyhow::anyhow!("Failed to send index to channel"))
    })?;
    
    // Recursively scan subdirectories in parallel
    dirs.par_iter().try_for_each(|entry| {
        scan_directory_parallel(entry.path(), tx.clone())
    })?;
    
    Ok(())
}

/// Recursively scans directory in parallel with metadata extraction.
fn scan_directory_parallel_with_metadata<P: AsRef<Path>>(
    root: P,
    tx: Sender<Index>,
) -> Result<()> {
    let root = root.as_ref();
    
    let entries: Vec<_> = fs::read_dir(root)
        .with_context(|| format!("Failed to read directory: {}", root.display()))?
        .filter_map(|e| e.ok())
        .collect();
    
    let (files, dirs): (Vec<_>, Vec<_>) = entries.into_iter()
        .partition(|entry| entry.path().is_file());
    
    // Process files with metadata in parallel
    files.par_iter().try_for_each(|entry| {
        let path = entry.path();
        let path_str = path.to_string_lossy().to_string();
        
        let name = entry.file_name()
            .to_string_lossy()
            .to_string();
        
        // Extract metadata
        let idx = match extract_metadata(&path) {
            Ok((mtime, size)) => {
                Index::with_metadata(path_str, name, mtime, size)
            }
            Err(_) => {
                // Fallback to index without metadata if extraction fails
                Index::new(path_str, name)
            }
        };
        
        tx.send(idx)
            .map_err(|_| anyhow::anyhow!("Failed to send index to channel"))
    })?;
    
    // Recursively scan subdirectories
    dirs.par_iter().try_for_each(|entry| {
        scan_directory_parallel_with_metadata(entry.path(), tx.clone())
    })?;
    
    Ok(())
}

/// Extracts file metadata (modification time and size).
fn extract_metadata<P: AsRef<Path>>(path: P) -> Result<(f64, i64)> {
    let metadata = fs::metadata(path.as_ref())
        .context("Failed to read file metadata")?;
    
    let mtime = metadata.modified()
        .context("Failed to get modification time")?
        .duration_since(SystemTime::UNIX_EPOCH)
        .context("Invalid modification time")?
        .as_secs_f64();
    
    let size = metadata.len() as i64;
    
    Ok((mtime, size))
}

/// Batches indices and writes them to database.
fn write_indices_batched(
    rx: crossbeam_channel::Receiver<Index>,
    db: &Database,
    batch_size: usize,
) -> Result<()> {
    let mut batch = Vec::with_capacity(batch_size);
    
    for idx in rx {
        batch.push(idx);
        
        if batch.len() >= batch_size {
            db.add_idxs(&batch)
                .context("Failed to write batch to database")?;
            batch.clear();
        }
    }
    
    // Write remaining indices
    if !batch.is_empty() {
        db.add_idxs(&batch)
            .context("Failed to write final batch to database")?;
    }
    
    Ok(())
}

/// Gets file metadata as a tuple (mtime, size).
///
/// # Arguments
/// * `path` - Path to the file
///
/// # Returns
/// Tuple of (modification_time_unix_timestamp, file_size_bytes)
pub fn get_file_metadata<P: AsRef<Path>>(path: P) -> Result<(f64, i64)> {
    extract_metadata(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    fn create_test_directory() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let base = temp_dir.path();
        
        // Create directory structure
        fs::create_dir_all(base.join("dir1")).unwrap();
        fs::create_dir_all(base.join("dir2/subdir")).unwrap();
        
        // Create files
        File::create(base.join("file1.txt")).unwrap().write_all(b"test1").unwrap();
        File::create(base.join("file2.txt")).unwrap().write_all(b"test2").unwrap();
        File::create(base.join("dir1/file3.txt")).unwrap().write_all(b"test3").unwrap();
        File::create(base.join("dir2/file4.txt")).unwrap().write_all(b"test4").unwrap();
        File::create(base.join("dir2/subdir/file5.txt")).unwrap().write_all(b"test5").unwrap();
        
        temp_dir
    }

    #[test]
    fn test_scan_idxs_basic() {
        let temp_dir = create_test_directory();
        let db_path = std::env::temp_dir().join(format!("test_scan_basic_{}.reminex.db", std::process::id()));
        let db = Database::init(&db_path).unwrap();
        
        let duration = scan_idxs(temp_dir.path(), &db, 100).unwrap();
        
        assert!(duration.as_millis() > 0, "Scan should take some time");
        
        // Verify files were indexed
        let count = db.batch_operation(|conn| {
            let count: i64 = conn.query_row("SELECT COUNT(*) FROM files", [], |row| row.get(0))?;
            Ok(count)
        }).unwrap();
        
        assert_eq!(count, 5, "Should have indexed 5 files");
        
        let _ = fs::remove_file(db_path);
    }

    #[test]
    fn test_scan_idxs_with_metadata() {
        let temp_dir = create_test_directory();
        let db_path = std::env::temp_dir().join(format!("test_scan_meta_{}.reminex.db", std::process::id()));
        let db = Database::init(&db_path).unwrap();
        
        let duration = scan_idxs_with_metadata(temp_dir.path(), &db, 100).unwrap();
        
        assert!(duration.as_millis() > 0);
        
        // Verify files have metadata
        let has_metadata = db.batch_operation(|conn| {
            let count: i64 = conn.query_row(
                "SELECT COUNT(*) FROM files WHERE mtime IS NOT NULL AND size IS NOT NULL",
                [],
                |row| row.get(0)
            )?;
            Ok(count)
        }).unwrap();
        
        assert_eq!(has_metadata, 5, "All files should have metadata");
        
        let _ = fs::remove_file(db_path);
    }

    #[test]
    fn test_get_file_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"Hello, World!").unwrap();
        drop(file);
        
        let (mtime, size) = get_file_metadata(&file_path).unwrap();
        
        assert!(mtime > 0.0, "mtime should be positive");
        assert_eq!(size, 13, "File size should be 13 bytes");
    }

    #[test]
    fn test_scan_nonexistent_path() {
        let db_path = std::env::temp_dir().join("nonexistent_test.reminex.db");
        let db = Database::init(&db_path).unwrap();
        
        let result = scan_idxs("/nonexistent/path", &db, 100);
        assert!(result.is_err(), "Should fail for nonexistent path");
        
        let _ = fs::remove_file(db_path);
    }

    #[test]
    fn test_large_batch_size() {
        let temp_dir = create_test_directory();
        let db_path = std::env::temp_dir().join(format!("test_large_batch_{}.reminex.db", std::process::id()));
        let db = Database::init(&db_path).unwrap();
        
        // Use very large batch size
        let duration = scan_idxs(temp_dir.path(), &db, 10000).unwrap();
        
        assert!(duration.as_millis() > 0);
        
        let count = db.batch_operation(|conn| {
            let count: i64 = conn.query_row("SELECT COUNT(*) FROM files", [], |row| row.get(0))?;
            Ok(count)
        }).unwrap();
        
        assert_eq!(count, 5);
        
        let _ = fs::remove_file(db_path);
    }
}
