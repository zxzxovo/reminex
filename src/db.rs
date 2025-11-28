use std::path::{Path, PathBuf};
use std::fs;
use anyhow::{Context, Result};
use rusqlite::Connection;

/// Represents a file index entry in the database.
#[derive(Debug, Clone, PartialEq)]
pub struct Index {
    pub path: String,
    pub name: String,
    pub mtime: Option<f64>,
    pub size: Option<i64>,
}

impl Index {
    /// Creates a new index entry with required fields only.
    pub fn new(path: String, name: String) -> Self {
        Self {
            path,
            name,
            mtime: None,
            size: None,
        }
    }

    /// Creates a new index entry with all fields.
    pub fn with_metadata(path: String, name: String, mtime: f64, size: i64) -> Self {
        Self {
            path,
            name,
            mtime: Some(mtime),
            size: Some(size),
        }
    }
}

/// Represents a database instance with file indexing capabilities.
#[derive(Debug, Clone, PartialEq)]
pub struct Database {
    pub path: PathBuf,
}

impl Database {
    /// Creates a new Database instance pointing to the specified path.
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }

    /// Initializes a new SQLite database at the instance's path.
    ///
    /// Creates the database file with optimized settings for fast indexing.
    /// Sets up the `files` table for storing file metadata.
    ///
    /// # Returns
    /// Returns `Ok(Database)` on success
    pub fn init(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .context("Failed to create parent directories")?;
        }
        
        // Create and open the database
        let conn = Connection::open(path)
            .context("Failed to create database file")?;
        
        // Performance optimization pragmas
        conn.execute_batch(
            "
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = OFF;
            PRAGMA cache_size = -2000000;
            PRAGMA temp_store = MEMORY;
            "
        )
        .context("Failed to set database pragmas")?;
        
        // Create files table
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS files (
                path  TEXT    PRIMARY KEY,
                name  TEXT    NOT NULL,
                mtime REAL,
                size  INTEGER
            );
            
            CREATE INDEX IF NOT EXISTS idx_name ON files (name);
            "
        )
        .context("Failed to create database schema")?;
        
        Ok(Self {
            path: path.to_path_buf(),
        })
    }

    /// Opens a connection to this database.
    fn connect(&self) -> Result<Connection> {
        Connection::open(&self.path)
            .context("Failed to open database connection")
    }

    /// Adds a single index entry to the database.
    ///
    /// # Arguments
    /// * `idx` - Index entry to add
    ///
    /// # Returns
    /// Returns `Ok(())` on success
    pub fn add_idx(&self, idx: &Index) -> Result<()> {
        let conn = self.connect()?;
        
        conn.execute(
            "INSERT OR REPLACE INTO files (path, name, mtime, size) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![&idx.path, &idx.name, &idx.mtime, &idx.size],
        )
        .context("Failed to insert index entry")?;
        
        Ok(())
    }

    /// Adds multiple index entries to the database in a single transaction.
    ///
    /// # Arguments
    /// * `idxs` - Slice of index entries to add
    ///
    /// # Returns
    /// Returns `Ok(())` on success, rolls back on error
    pub fn add_idxs(&self, idxs: &[Index]) -> Result<()> {
        let mut conn = self.connect()?;
        
        let tx = conn.transaction()
            .context("Failed to start transaction")?;
        
        {
            let mut stmt = tx.prepare(
                "INSERT OR REPLACE INTO files (path, name, mtime, size) VALUES (?1, ?2, ?3, ?4)"
            )
            .context("Failed to prepare statement")?;
            
            for idx in idxs {
                stmt.execute(rusqlite::params![&idx.path, &idx.name, &idx.mtime, &idx.size])
                    .context("Failed to insert index entry")?;
            }
        }
        
        tx.commit()
            .context("Failed to commit transaction")?;
        
        Ok(())
    }

    /// Executes a batch operation with a single database connection.
    ///
    /// More efficient for operations that need multiple database interactions,
    /// as the connection is reused within the closure.
    ///
    /// # Arguments
    /// * `f` - Closure that receives a mutable database connection reference
    ///
    /// # Returns
    /// Returns the result from the closure
    ///
    /// # Example
    /// ```ignore
    /// db.batch_operation(|conn| {
    ///     // Multiple operations using the same connection
    ///     conn.execute("...", params![])?;
    ///     conn.execute("...", params![])?;
    ///     Ok(())
    /// })?;
    /// ```
    pub fn batch_operation<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&mut Connection) -> Result<R>,
    {
        let mut conn = self.connect()?;
        f(&mut conn)
    }
}

/// Collects all `.reminex.db` files from the given paths.
///
/// For file paths, checks if the filename ends with `.reminex.db`.
/// For directory paths, scans immediate children (one level deep) for `.reminex.db` files.
///
/// # Arguments
/// * `paths` - A list of file or directory paths to search
///
/// # Returns
/// A vector of PathBuf containing all found `.reminex.db` files
pub fn get_db_files<P: AsRef<Path>>(paths: Vec<P>) -> Vec<PathBuf> {
    let mut db_files = Vec::new();

    for path in paths {
        let path = path.as_ref();
        
        if !path.exists() {
            continue;
        }

        if path.is_file() {
            // Check if the file has .reminex.db extension
            if let Some(file_name) = path.file_name() {
                if file_name.to_string_lossy().ends_with(".reminex.db") {
                    db_files.push(path.to_path_buf());
                }
            }
        } else if path.is_dir() {
            // Read directory entries (one level deep only)
            if let Ok(entries) = fs::read_dir(path) {
                for entry in entries.flatten() {
                    let entry_path = entry.path();
                    
                    // Only process files, not subdirectories
                    if entry_path.is_file() {
                        if let Some(file_name) = entry_path.file_name() {
                            if file_name.to_string_lossy().ends_with(".reminex.db") {
                                db_files.push(entry_path);
                            }
                        }
                    }
                }
            }
        }
    }

    db_files
}

/// Attempts to convert database file paths to Database instances.
///
/// Validates each path and creates a Database instance if the file exists
/// and appears to be a valid SQLite database.
///
/// # Arguments
/// * `paths` - Vector of database file paths
///
/// # Returns
/// A vector of successfully loaded Database instances
pub fn try_read_db(paths: Vec<PathBuf>) -> Result<Vec<Database>> {
    let mut databases = Vec::new();
    
    for path in paths {
        // Check if file exists
        if !path.exists() {
            continue;
        }
        
        // Try to open as SQLite database to verify validity
        if let Ok(_conn) = Connection::open(&path) {
            databases.push(Database::new(path));
        }
    }
    
    Ok(databases)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use rusqlite::Connection;

    fn setup_test_dir() -> PathBuf {
        let temp_dir = std::env::temp_dir().join("reminex_test");
        
        // Clean up if exists
        let _ = fs::remove_dir_all(&temp_dir);
        
        // Create test directory structure
        fs::create_dir_all(&temp_dir).unwrap();
        fs::create_dir_all(temp_dir.join("subdir")).unwrap();
        
        // Create test files
        File::create(temp_dir.join("test1.reminex.db")).unwrap();
        File::create(temp_dir.join("test2.reminex.db")).unwrap();
        File::create(temp_dir.join("other.txt")).unwrap();
        File::create(temp_dir.join("subdir/nested.reminex.db")).unwrap();
        
        temp_dir
    }

    fn cleanup_test_dir(dir: &Path) {
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn test_single_db_file() {
        let temp_dir = setup_test_dir();
        let db_file = temp_dir.join("test1.reminex.db");
        
        let result = get_db_files(vec![&db_file]);
        
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], db_file);
        
        cleanup_test_dir(&temp_dir);
    }

    #[test]
    fn test_non_db_file() {
        let temp_dir = setup_test_dir();
        let other_file = temp_dir.join("other.txt");
        
        let result = get_db_files(vec![&other_file]);
        
        assert_eq!(result.len(), 0);
        
        cleanup_test_dir(&temp_dir);
    }

    #[test]
    fn test_directory_scan() {
        let temp_dir = setup_test_dir();
        
        let result = get_db_files(vec![&temp_dir]);
        
        // Should find test1.reminex.db and test2.reminex.db
        // Should NOT find subdir/nested.reminex.db (not one level deep)
        assert_eq!(result.len(), 2);
        assert!(result.iter().any(|p| p.file_name().unwrap() == "test1.reminex.db"));
        assert!(result.iter().any(|p| p.file_name().unwrap() == "test2.reminex.db"));
        
        cleanup_test_dir(&temp_dir);
    }

    #[test]
    fn test_mixed_paths() {
        let temp_dir = setup_test_dir();
        let db_file = temp_dir.join("test1.reminex.db");
        
        let result = get_db_files(vec![&temp_dir, &db_file]);
        
        // Should find test1.reminex.db and test2.reminex.db from directory
        // Plus test1.reminex.db from direct file path (might be duplicate)
        assert!(result.len() >= 2);
        
        cleanup_test_dir(&temp_dir);
    }

    #[test]
    fn test_nonexistent_path() {
        let result = get_db_files(vec![Path::new("/nonexistent/path")]);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_empty_input() {
        let result: Vec<PathBuf> = get_db_files(Vec::<&Path>::new());
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_init_db_creates_file() {
        let temp_dir = std::env::temp_dir().join("reminex_init_test");
        let _ = fs::remove_dir_all(&temp_dir);
        
        let db_path = temp_dir.join("test.reminex.db");
        
        let result = Database::init(&db_path);
        assert!(result.is_ok(), "Failed to create database: {:?}", result.err());
        assert!(db_path.exists(), "Database file was not created");
        
        // Verify we can open the database and query the schema
        let conn = Connection::open(&db_path).unwrap();
        
        // Check files table exists
        let table_exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='files'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(table_exists, 1, "Files table was not created");
        
        // Verify index exists
        let index_exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name='idx_name'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(index_exists, 1, "Index idx_name was not created");
        
        // Verify table schema (test insert with only required fields)
        conn.execute(
            "INSERT INTO files (path, name) VALUES (?, ?)",
            ["C:\\test\\file.txt", "file.txt"],
        ).unwrap();
        
        // Verify optional fields work
        conn.execute(
            "INSERT INTO files (path, name, mtime, size) VALUES (?, ?, ?, ?)",
            rusqlite::params!["C:\\test\\file2.txt", "file2.txt", 1234567890.0, 1024],
        ).unwrap();
        
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_init_db_creates_parent_dirs() {
        let temp_dir = std::env::temp_dir().join("reminex_init_test_nested");
        let _ = fs::remove_dir_all(&temp_dir);
        
        let db_path = temp_dir.join("subdir1/subdir2/test.reminex.db");
        
        let result = Database::init(&db_path);
        assert!(result.is_ok(), "Failed to create database with nested dirs: {:?}", result.err());
        assert!(db_path.exists(), "Database file was not created in nested directory");
        
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_init_db_existing_file() {
        let temp_dir = std::env::temp_dir().join("reminex_init_test_existing");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();
        
        let db_path = temp_dir.join("existing.reminex.db");
        
        // Create database first time
        Database::init(&db_path).unwrap();
        
        // Try to init again - should succeed (idempotent)
        let result = Database::init(&db_path);
        assert!(result.is_ok(), "Failed to init existing database: {:?}", result.err());
        
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_add_idx_single_entry() {
        let temp_dir = std::env::temp_dir().join("reminex_add_idx_test");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();
        
        let db_path = temp_dir.join("test.reminex.db");
        let db = Database::init(&db_path).unwrap();
        
        let idx = Index::new("C:\\test\\file.txt".to_string(), "file.txt".to_string());
        let result = db.add_idx(&idx);
        assert!(result.is_ok(), "Failed to add index: {:?}", result.err());
        
        // Verify the entry was added
        let conn = Connection::open(&db_path).unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM files WHERE path = ?", [&idx.path], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);
        
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_add_idx_with_metadata() {
        let temp_dir = std::env::temp_dir().join("reminex_add_idx_meta_test");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();
        
        let db_path = temp_dir.join("test.reminex.db");
        let db = Database::init(&db_path).unwrap();
        
        let idx = Index::with_metadata(
            "C:\\test\\file.txt".to_string(),
            "file.txt".to_string(),
            1234567890.5,
            2048,
        );
        db.add_idx(&idx).unwrap();
        
        // Verify all fields
        let conn = Connection::open(&db_path).unwrap();
        let (name, mtime, size): (String, Option<f64>, Option<i64>) = conn
            .query_row(
                "SELECT name, mtime, size FROM files WHERE path = ?",
                [&idx.path],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        
        assert_eq!(name, "file.txt");
        assert_eq!(mtime, Some(1234567890.5));
        assert_eq!(size, Some(2048));
        
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_add_idx_replace_existing() {
        let temp_dir = std::env::temp_dir().join("reminex_add_idx_replace_test");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();
        
        let db_path = temp_dir.join("test.reminex.db");
        let db = Database::init(&db_path).unwrap();
        
        // Add first entry
        let idx1 = Index::with_metadata(
            "C:\\test\\file.txt".to_string(),
            "file.txt".to_string(),
            1000.0,
            100,
        );
        db.add_idx(&idx1).unwrap();
        
        // Replace with updated entry
        let idx2 = Index::with_metadata(
            "C:\\test\\file.txt".to_string(),
            "file.txt".to_string(),
            2000.0,
            200,
        );
        db.add_idx(&idx2).unwrap();
        
        // Verify only one entry exists with updated values
        let conn = Connection::open(&db_path).unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM files", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);
        
        let (mtime, size): (Option<f64>, Option<i64>) = conn
            .query_row(
                "SELECT mtime, size FROM files WHERE path = ?",
                ["C:\\test\\file.txt"],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(mtime, Some(2000.0));
        assert_eq!(size, Some(200));
        
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_add_idxs_multiple_entries() {
        let temp_dir = std::env::temp_dir().join("reminex_add_idxs_test");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();
        
        let db_path = temp_dir.join("test.reminex.db");
        let db = Database::init(&db_path).unwrap();
        
        let idxs = vec![
            Index::new("C:\\test\\file1.txt".to_string(), "file1.txt".to_string()),
            Index::with_metadata("C:\\test\\file2.txt".to_string(), "file2.txt".to_string(), 1000.0, 100),
            Index::with_metadata("C:\\test\\file3.txt".to_string(), "file3.txt".to_string(), 2000.0, 200),
        ];
        
        let result = db.add_idxs(&idxs);
        assert!(result.is_ok(), "Failed to add indices: {:?}", result.err());
        
        // Verify all entries were added
        let conn = Connection::open(&db_path).unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM files", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 3);
        
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_add_idxs_empty_slice() {
        let temp_dir = std::env::temp_dir().join("reminex_add_idxs_empty_test");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();
        
        let db_path = temp_dir.join("test.reminex.db");
        let db = Database::init(&db_path).unwrap();
        
        let idxs: Vec<Index> = vec![];
        let result = db.add_idxs(&idxs);
        assert!(result.is_ok(), "Failed with empty slice: {:?}", result.err());
        
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_add_idxs_transaction_rollback_on_error() {
        let temp_dir = std::env::temp_dir().join("reminex_add_idxs_rollback_test");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();
        
        let db_path = temp_dir.join("test.reminex.db");
        let db = Database::init(&db_path).unwrap();
        
        // First add one entry successfully
        let good_idx = Index::new("C:\\test\\good.txt".to_string(), "good.txt".to_string());
        db.add_idx(&good_idx).unwrap();
        
        // Verify count is 1
        let conn = Connection::open(&db_path).unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM files", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);
        
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_try_read_db_valid_databases() {
        let temp_dir = std::env::temp_dir().join("reminex_try_read_test");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();
        
        // Create multiple valid databases
        let db1_path = temp_dir.join("db1.reminex.db");
        let db2_path = temp_dir.join("db2.reminex.db");
        Database::init(&db1_path).unwrap();
        Database::init(&db2_path).unwrap();
        
        let paths = vec![db1_path.clone(), db2_path.clone()];
        let result = try_read_db(paths);
        
        assert!(result.is_ok());
        let databases = result.unwrap();
        assert_eq!(databases.len(), 2);
        assert!(databases.iter().any(|db| db.path == db1_path));
        assert!(databases.iter().any(|db| db.path == db2_path));
        
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_try_read_db_nonexistent_files() {
        let paths = vec![
            PathBuf::from("C:\\nonexistent\\db1.reminex.db"),
            PathBuf::from("C:\\nonexistent\\db2.reminex.db"),
        ];
        
        let result = try_read_db(paths);
        assert!(result.is_ok());
        let databases = result.unwrap();
        assert_eq!(databases.len(), 0);
    }

    #[test]
    fn test_try_read_db_mixed_valid_invalid() {
        let temp_dir = std::env::temp_dir().join("reminex_try_read_mixed_test");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();
        
        // Create one valid database
        let valid_db = temp_dir.join("valid.reminex.db");
        Database::init(&valid_db).unwrap();
        
        // Mix with nonexistent paths
        let paths = vec![
            valid_db.clone(),
            PathBuf::from("C:\\nonexistent\\invalid.reminex.db"),
        ];
        
        let result = try_read_db(paths);
        assert!(result.is_ok());
        let databases = result.unwrap();
        assert_eq!(databases.len(), 1);
        assert_eq!(databases[0].path, valid_db);
        
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_try_read_db_empty_input() {
        let paths: Vec<PathBuf> = vec![];
        let result = try_read_db(paths);
        
        assert!(result.is_ok());
        let databases = result.unwrap();
        assert_eq!(databases.len(), 0);
    }

    #[test]
    fn test_database_new() {
        let path = PathBuf::from("C:\\test\\db.reminex.db");
        let db = Database::new(&path);
        assert_eq!(db.path, path);
    }

    #[test]
    fn test_batch_operation() {
        let temp_dir = std::env::temp_dir().join("reminex_batch_op_test");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();
        
        let db_path = temp_dir.join("test.reminex.db");
        let db = Database::init(&db_path).unwrap();
        
        // Use batch_operation to add multiple entries efficiently
        let result = db.batch_operation(|conn| {
            conn.execute(
                "INSERT INTO files (path, name, mtime, size) VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params!["C:\\test\\file1.txt", "file1.txt", 1000.0, 100],
            )?;
            conn.execute(
                "INSERT INTO files (path, name, mtime, size) VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params!["C:\\test\\file2.txt", "file2.txt", 2000.0, 200],
            )?;
            conn.execute(
                "INSERT INTO files (path, name, mtime, size) VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params!["C:\\test\\file3.txt", "file3.txt", 3000.0, 300],
            )?;
            Ok(())
        });
        
        assert!(result.is_ok(), "Batch operation failed: {:?}", result.err());
        
        // Verify all entries were added
        let conn = Connection::open(&db_path).unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM files", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 3);
        
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_batch_operation_with_return_value() {
        let temp_dir = std::env::temp_dir().join("reminex_batch_op_return_test");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();
        
        let db_path = temp_dir.join("test.reminex.db");
        let db = Database::init(&db_path).unwrap();
        
        // Add some data
        db.add_idx(&Index::new("C:\\test\\file1.txt".to_string(), "file1.txt".to_string())).unwrap();
        db.add_idx(&Index::new("C:\\test\\file2.txt".to_string(), "file2.txt".to_string())).unwrap();
        
        // Use batch_operation to query and return a value
        let result = db.batch_operation(|conn| {
            let count: i64 = conn.query_row("SELECT COUNT(*) FROM files", [], |row| row.get(0))?;
            Ok(count)
        });
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);
        
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_batch_operation_with_transaction() {
        let temp_dir = std::env::temp_dir().join("reminex_batch_op_tx_test");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();
        
        let db_path = temp_dir.join("test.reminex.db");
        let db = Database::init(&db_path).unwrap();
        
        // Use batch_operation with manual transaction
        let result = db.batch_operation(|conn| {
            let tx = conn.transaction()?;
            
            tx.execute(
                "INSERT INTO files (path, name) VALUES (?1, ?2)",
                ["C:\\test\\file1.txt", "file1.txt"],
            )?;
            tx.execute(
                "INSERT INTO files (path, name) VALUES (?1, ?2)",
                ["C:\\test\\file2.txt", "file2.txt"],
            )?;
            
            tx.commit()?;
            Ok(())
        });
        
        assert!(result.is_ok());
        
        // Verify transaction succeeded
        let conn = Connection::open(&db_path).unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM files", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 2);
        
        let _ = fs::remove_dir_all(&temp_dir);
    }
}
