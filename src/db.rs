use std::path::{Path, PathBuf};
use std::fs;

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;

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
}
