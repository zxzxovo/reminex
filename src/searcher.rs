use anyhow::{Context, Result};
use rusqlite::params;
use std::path::{Path, PathBuf};

use crate::db::Database;

/// Represents a search result item.
#[derive(Debug, Clone, PartialEq)]
pub struct SearchResult {
    pub path: String,
    pub name: String,
}

/// Represents a tree node for hierarchical display of search results.
#[derive(Debug, Clone)]
pub struct TreeNode {
    pub name: String,
    pub path: PathBuf,
    pub children: Vec<TreeNode>,
}

impl TreeNode {
    /// Creates a new tree node.
    pub fn new(name: String, path: PathBuf) -> Self {
        Self {
            name,
            path,
            children: Vec::new(),
        }
    }

    /// Checks if this is a leaf node (file).
    pub fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }

    /// Sorts children recursively by name (case-insensitive).
    pub fn sort_children(&mut self) {
        self.children.sort_by_key(|c| c.name.to_lowercase());
        for child in &mut self.children {
            child.sort_children();
        }
    }
}

/// Configuration for search operations.
#[derive(Debug, Clone)]
pub struct SearchConfig {
    /// Maximum number of results to return per keyword
    pub max_results: usize,
    /// Whether to search in path (true) or only filename (false)
    pub search_in_path: bool,
    /// Case sensitive search
    pub case_sensitive: bool,
    /// Include only results containing these keywords (AND logic)
    pub include_filters: Vec<String>,
    /// Exclude results containing these keywords (OR logic)
    pub exclude_filters: Vec<String>,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            max_results: 2000,
            search_in_path: true,
            case_sensitive: false,
            include_filters: Vec::new(),
            exclude_filters: Vec::new(),
        }
    }
}

/// Splits user input into multiple search keywords.
///
/// Supports multiple delimiters: semicolon (;；), space, comma, and combinations.
///
/// # Arguments
/// * `input` - User input string containing one or more keywords
///
/// # Returns
/// Vector of trimmed keywords
///
/// # Example
/// ```
/// use reminex::searcher::parse_search_keywords;
///
/// let keywords = parse_search_keywords("photo; video image");
/// assert_eq!(keywords, vec!["photo", "video", "image"]);
/// ```
pub fn parse_search_keywords(input: &str) -> Vec<String> {
    input
        .split([';', '；', ' ', ',', '，', '\t'])
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

/// Apply include and exclude filters to search results.
///
/// # Arguments
/// * `results` - Search results to filter
/// * `config` - Search configuration containing filters
///
/// # Returns
/// Filtered search results
///
/// # Logic
/// - Include filters: Result must contain ALL include keywords (AND logic)
/// - Exclude filters: Result must NOT contain ANY exclude keywords (OR logic)
fn apply_filters(results: Vec<SearchResult>, config: &SearchConfig) -> Vec<SearchResult> {
    if config.include_filters.is_empty() && config.exclude_filters.is_empty() {
        return results;
    }

    results
        .into_iter()
        .filter(|result| {
            // Combine path and name for filtering
            let full_text = if config.case_sensitive {
                format!("{} {}", result.path, result.name)
            } else {
                format!("{} {}", result.path, result.name).to_lowercase()
            };

            // Check include filters (must match ALL)
            let includes_match = if config.include_filters.is_empty() {
                true
            } else {
                config.include_filters.iter().all(|filter| {
                    let filter_text = if config.case_sensitive {
                        filter.clone()
                    } else {
                        filter.to_lowercase()
                    };
                    full_text.contains(&filter_text)
                })
            };

            // Check exclude filters (must NOT match ANY)
            let excludes_match = config.exclude_filters.iter().any(|filter| {
                let filter_text = if config.case_sensitive {
                    filter.clone()
                } else {
                    filter.to_lowercase()
                };
                full_text.contains(&filter_text)
            });

            includes_match && !excludes_match
        })
        .collect()
}

/// Searches for files matching a single keyword.
///
/// # Arguments
/// * `db` - Database instance to search in
/// * `keyword` - Search keyword (will be wrapped with % for LIKE query)
/// * `config` - Search configuration
///
/// # Returns
/// Vector of search results matching the keyword
pub fn search_by_keyword(
    db: &Database,
    keyword: &str,
    config: &SearchConfig,
) -> Result<Vec<SearchResult>> {
    if keyword.trim().is_empty() {
        return Ok(Vec::new());
    }

    db.batch_operation(|conn| {
        let like_pattern = format!("%{}%", keyword);
        let query = if config.search_in_path {
            format!(
                "SELECT path, name FROM files WHERE name LIKE ?1 OR path LIKE ?1 ORDER BY path LIMIT {}",
                config.max_results
            )
        } else {
            format!(
                "SELECT path, name FROM files WHERE name LIKE ?1 ORDER BY path LIMIT {}",
                config.max_results
            )
        };

        let mut stmt = conn.prepare(&query)
            .context("Failed to prepare search query")?;

        let rows = stmt.query_map(params![like_pattern], |row| {
            Ok(SearchResult {
                path: row.get(0)?,
                name: row.get(1)?,
            })
        })
        .context("Failed to execute search query")?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }

        Ok(results)
    }).map(|results| apply_filters(results, config))
}

/// Searches for files matching multiple keywords.
///
/// Each keyword is searched independently, and results are combined.
///
/// # Arguments
/// * `db` - Database instance to search in
/// * `keywords` - Vector of search keywords
/// * `config` - Search configuration
///
/// # Returns
/// Vector of tuples (keyword, results) for each keyword
pub fn search_multiple_keywords(
    db: &Database,
    keywords: &[String],
    config: &SearchConfig,
) -> Result<Vec<(String, Vec<SearchResult>)>> {
    let mut all_results = Vec::new();

    for keyword in keywords {
        let results = search_by_keyword(db, keyword, config)?;
        all_results.push((keyword.clone(), results));
    }

    Ok(all_results)
}

/// Searches databases from user input string.
///
/// Convenience function that combines keyword parsing and searching.
///
/// # Arguments
/// * `db` - Database instance to search in
/// * `input` - Raw user input (may contain multiple keywords)
/// * `config` - Search configuration
///
/// # Returns
/// Vector of tuples (keyword, results) for each parsed keyword
pub fn search_from_input(
    db: &Database,
    input: &str,
    config: &SearchConfig,
) -> Result<Vec<(String, Vec<SearchResult>)>> {
    let keywords = parse_search_keywords(input);

    if keywords.is_empty() {
        return Ok(Vec::new());
    }

    search_multiple_keywords(db, &keywords, config)
}

/// Builds a tree structure from search results.
///
/// Automatically identifies the common prefix path from all results.
///
/// # Arguments
/// * `results` - Search results to build tree from
/// * `root_name` - Display name for root node (e.g., "搜索结果")
///
/// # Returns
/// Root TreeNode containing the hierarchical structure
pub fn build_tree(results: &[SearchResult], root_name: &str) -> TreeNode {
    if results.is_empty() {
        return TreeNode::new(root_name.to_string(), PathBuf::new());
    }

    // Find common prefix from all paths
    let common_prefix = find_common_prefix(results);
    let mut root = TreeNode::new(
        format!("{} ({})", root_name, common_prefix.display()),
        common_prefix.clone(),
    );

    for result in results {
        insert_path_into_tree(&mut root, &PathBuf::from(&result.path));
    }

    root.sort_children();
    root
}

/// Finds the common directory prefix for all search results.
///
/// Returns the deepest common directory shared by all paths.
fn find_common_prefix(results: &[SearchResult]) -> PathBuf {
    if results.is_empty() {
        return PathBuf::from(".");
    }

    if results.len() == 1 {
        let path = PathBuf::from(&results[0].path);
        return path.parent().unwrap_or(Path::new(".")).to_path_buf();
    }

    // Start with the first path's parent directory
    let first_path = PathBuf::from(&results[0].path);
    let mut common = first_path.parent().unwrap_or(Path::new(".")).to_path_buf();

    // Iterate through all results to find common prefix
    for result in results.iter().skip(1) {
        let path = PathBuf::from(&result.path);
        let parent = path.parent().unwrap_or(Path::new("."));

        // Find common path between current common and this path
        common = find_common_path(&common, parent);

        // If we've reduced to root or current dir, no point continuing
        if common == Path::new(".") || common == Path::new("/") || common == Path::new("") {
            break;
        }
    }

    common
}

/// Finds the common path between two paths.
fn find_common_path(path1: &Path, path2: &Path) -> PathBuf {
    let components1: Vec<_> = path1.components().collect();
    let components2: Vec<_> = path2.components().collect();

    let mut common = PathBuf::new();
    let min_len = components1.len().min(components2.len());

    for i in 0..min_len {
        if components1[i] == components2[i] {
            common.push(components1[i]);
        } else {
            break;
        }
    }

    if common.as_os_str().is_empty() {
        PathBuf::from(".")
    } else {
        common
    }
}

/// Inserts a file path into the tree structure.
fn insert_path_into_tree(root: &mut TreeNode, target_path: &Path) {
    let Ok(relative) = target_path.strip_prefix(&root.path) else {
        // If strip_prefix fails, use the full path
        insert_full_path_into_tree(root, target_path);
        return;
    };

    if relative == Path::new("") {
        return;
    }

    let mut current = root;
    for comp in relative.components() {
        let part_str = comp.as_os_str().to_string_lossy().to_string();
        let child_path = current.path.join(&part_str);

        let child_index = current.children.iter().position(|c| c.path == child_path);
        if let Some(idx) = child_index {
            current = &mut current.children[idx];
        } else {
            let new_node = TreeNode::new(part_str, child_path);
            current.children.push(new_node);
            let len = current.children.len();
            current = &mut current.children[len - 1];
        }
    }
}

/// Inserts a full file path into the tree structure (fallback method).
fn insert_full_path_into_tree(root: &mut TreeNode, target_path: &Path) {
    let mut current = root;

    for comp in target_path.components() {
        let part_str = comp.as_os_str().to_string_lossy().to_string();
        let child_path = if current.path.as_os_str().is_empty() {
            PathBuf::from(&part_str)
        } else {
            current.path.join(&part_str)
        };

        let child_index = current.children.iter().position(|c| c.name == part_str);
        if let Some(idx) = child_index {
            current = &mut current.children[idx];
        } else {
            let new_node = TreeNode::new(part_str, child_path);
            current.children.push(new_node);
            let len = current.children.len();
            current = &mut current.children[len - 1];
        }
    }
}

/// Formats a tree node as a string with tree-style display.
///
/// Uses box-drawing characters for a clean hierarchical view.
///
/// # Arguments
/// * `node` - Tree node to format
/// * `prefix` - Current prefix for indentation
/// * `is_last` - Whether this is the last child of its parent
///
/// # Returns
/// Formatted string representation
pub fn format_tree_node(node: &TreeNode, prefix: &str, is_last: bool) -> String {
    let mut output = String::new();

    let connector = if is_last { "└─ " } else { "├─ " };
    let display_name = if node.is_leaf() {
        node.name.clone()
    } else {
        format!("{}/", node.name)
    };

    output.push_str(&format!("{}{}{}\n", prefix, connector, display_name));

    let new_prefix = format!("{}{}", prefix, if is_last { "   " } else { "│  " });
    for (i, child) in node.children.iter().enumerate() {
        let is_last_child = i == node.children.len() - 1;
        output.push_str(&format_tree_node(child, &new_prefix, is_last_child));
    }

    output
}

/// Prints a tree structure to stdout.
///
/// Convenience function for displaying search results in tree format.
///
/// # Arguments
/// * `root` - Root node of the tree
pub fn print_tree(root: &TreeNode) {
    println!("{}", root.name);
    for (i, child) in root.children.iter().enumerate() {
        let is_last = i == root.children.len() - 1;
        print!("{}", format_tree_node(child, "", is_last));
    }
}

/// Search across multiple databases
///
/// # Arguments
/// * `db_paths` - Vector of database file paths
/// * `keywords` - Vector of search keywords
/// * `config` - Search configuration
///
/// # Returns
/// Vector of tuples (database_name, keyword, results) for each database and keyword
pub fn search_multiple_databases(
    db_paths: &[PathBuf],
    keywords: &[String],
    config: &SearchConfig,
) -> Result<Vec<(String, String, Vec<SearchResult>)>> {
    let mut all_results = Vec::new();

    for db_path in db_paths {
        let db_name = db_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let db = Database::new(db_path);

        for keyword in keywords {
            let results = search_by_keyword(&db, keyword, config)?;
            all_results.push((db_name.clone(), keyword.clone(), results));
        }
    }

    Ok(all_results)
}

/// Search in a specific database from multiple available databases
///
/// # Arguments
/// * `db_paths` - Vector of available database file paths
/// * `db_name` - Name of the database to search in (or "all" for all databases)
/// * `keywords` - Vector of search keywords
/// * `config` - Search configuration
///
/// # Returns
/// Vector of tuples (database_name, keyword, results)
pub fn search_in_selected_database(
    db_paths: &[PathBuf],
    db_name: &str,
    keywords: &[String],
    config: &SearchConfig,
) -> Result<Vec<(String, String, Vec<SearchResult>)>> {
    if db_name == "all" {
        return search_multiple_databases(db_paths, keywords, config);
    }

    // Find the specific database
    let db_path = db_paths
        .iter()
        .find(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n == db_name)
                .unwrap_or(false)
        })
        .ok_or_else(|| anyhow::anyhow!("数据库不存在: {}", db_name))?;

    let db = Database::new(db_path);
    let mut results = Vec::new();

    for keyword in keywords {
        let search_results = search_by_keyword(&db, keyword, config)?;
        results.push((db_name.to_string(), keyword.clone(), search_results));
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Index;
    use tempfile::TempDir;

    fn create_test_db_with_data() -> (TempDir, Database) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.reminex.db");
        let db = Database::init(&db_path).unwrap();

        // Insert test data
        let indices = vec![
            Index::new(
                "Z:\\photos\\2023\\summer.jpg".to_string(),
                "summer.jpg".to_string(),
            ),
            Index::new(
                "Z:\\photos\\2023\\winter.jpg".to_string(),
                "winter.jpg".to_string(),
            ),
            Index::new(
                "Z:\\documents\\report.pdf".to_string(),
                "report.pdf".to_string(),
            ),
            Index::new(
                "Z:\\videos\\summer_vacation.mp4".to_string(),
                "summer_vacation.mp4".to_string(),
            ),
            Index::new(
                "Z:\\music\\summer_hits.mp3".to_string(),
                "summer_hits.mp3".to_string(),
            ),
        ];
        db.add_idxs(&indices).unwrap();

        (temp_dir, db)
    }

    #[test]
    fn test_parse_search_keywords() {
        assert_eq!(
            parse_search_keywords("photo;video;music"),
            vec!["photo", "video", "music"]
        );

        assert_eq!(
            parse_search_keywords("photo video music"),
            vec!["photo", "video", "music"]
        );

        assert_eq!(
            parse_search_keywords("photo; video, music"),
            vec!["photo", "video", "music"]
        );

        assert_eq!(
            parse_search_keywords("photo；video，music"),
            vec!["photo", "video", "music"]
        );

        assert_eq!(
            parse_search_keywords("  photo  ;  video  "),
            vec!["photo", "video"]
        );

        assert_eq!(parse_search_keywords(""), Vec::<String>::new());
    }

    #[test]
    fn test_search_by_keyword() {
        let (_temp, db) = create_test_db_with_data();
        let config = SearchConfig::default();

        let results = search_by_keyword(&db, "summer", &config).unwrap();
        assert_eq!(results.len(), 3); // summer.jpg, summer_vacation.mp4, summer_hits.mp3

        let results = search_by_keyword(&db, "winter", &config).unwrap();
        assert_eq!(results.len(), 1);

        let results = search_by_keyword(&db, "nonexistent", &config).unwrap();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_search_multiple_keywords() {
        let (_temp, db) = create_test_db_with_data();
        let config = SearchConfig::default();
        let keywords = vec!["summer".to_string(), "winter".to_string()];

        let results = search_multiple_keywords(&db, &keywords, &config).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0, "summer");
        assert_eq!(results[0].1.len(), 3);
        assert_eq!(results[1].0, "winter");
        assert_eq!(results[1].1.len(), 1);
    }

    #[test]
    fn test_search_from_input() {
        let (_temp, db) = create_test_db_with_data();
        let config = SearchConfig::default();

        let results = search_from_input(&db, "summer; winter", &config).unwrap();
        assert_eq!(results.len(), 2);

        let results = search_from_input(&db, "", &config).unwrap();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_search_config() {
        let (_temp, db) = create_test_db_with_data();

        // Test max_results limit
        let config = SearchConfig {
            max_results: 1,
            ..Default::default()
        };
        let results = search_by_keyword(&db, "summer", &config).unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_build_tree() {
        // Use platform-independent path construction
        use std::path::MAIN_SEPARATOR;
        let sep = MAIN_SEPARATOR.to_string();

        let base = if cfg!(windows) {
            "Z:".to_string()
        } else {
            "".to_string()
        };

        let results = vec![
            SearchResult {
                path: format!("{}{sep}photos{sep}2023{sep}summer.jpg", base),
                name: "summer.jpg".to_string(),
            },
            SearchResult {
                path: format!("{}{sep}photos{sep}2023{sep}winter.jpg", base),
                name: "winter.jpg".to_string(),
            },
            SearchResult {
                path: format!("{}{sep}documents{sep}report.pdf", base),
                name: "report.pdf".to_string(),
            },
        ];

        let tree = build_tree(&results, "搜索结果");

        assert!(tree.name.contains("搜索结果"));

        // The tree structure depends on the platform and common prefix detection
        // Just verify we have a valid tree structure
        assert!(!tree.children.is_empty(), "Tree should have children");

        // Find photos folder (might be nested under platform-specific root)
        fn find_node_recursive<'a>(node: &'a TreeNode, name: &str) -> Option<&'a TreeNode> {
            if node.name == name {
                return Some(node);
            }
            for child in &node.children {
                if let Some(found) = find_node_recursive(child, name) {
                    return Some(found);
                }
            }
            None
        }

        let photos = find_node_recursive(&tree, "photos").expect("Should find photos folder");
        assert_eq!(photos.children.len(), 1); // 2023 folder

        let year_2023 = &photos.children[0];
        assert_eq!(year_2023.name, "2023");
        assert_eq!(year_2023.children.len(), 2); // summer.jpg and winter.jpg
    }

    #[test]
    fn test_tree_node_is_leaf() {
        let mut node = TreeNode::new("file.txt".to_string(), PathBuf::from("Z:\\file.txt"));
        assert!(node.is_leaf());

        node.children.push(TreeNode::new(
            "child".to_string(),
            PathBuf::from("Z:\\child"),
        ));
        assert!(!node.is_leaf());
    }

    #[test]
    fn test_format_tree_node() {
        let mut root = TreeNode::new("root".to_string(), PathBuf::from("Z:\\"));
        root.children.push(TreeNode::new(
            "file1.txt".to_string(),
            PathBuf::from("Z:\\file1.txt"),
        ));
        root.children.push(TreeNode::new(
            "file2.txt".to_string(),
            PathBuf::from("Z:\\file2.txt"),
        ));

        let output = format_tree_node(&root.children[0], "", false);
        assert!(output.contains("├─ file1.txt"));

        let output = format_tree_node(&root.children[1], "", true);
        assert!(output.contains("└─ file2.txt"));
    }

    #[test]
    fn test_search_empty_keyword() {
        let (_temp, db) = create_test_db_with_data();
        let config = SearchConfig::default();

        let results = search_by_keyword(&db, "", &config).unwrap();
        assert_eq!(results.len(), 0);

        let results = search_by_keyword(&db, "   ", &config).unwrap();
        assert_eq!(results.len(), 0);
    }
}
