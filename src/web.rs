use axum::{
    Router,
    extract::{Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Json},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tower_http::services::ServeDir;

use crate::db::Database;
use crate::indexer;
use crate::searcher::{SearchConfig, SearchResult, TreeNode, build_tree, search_in_selected_database, parse_search_keywords};

/// Web server state
#[derive(Clone)]
pub struct AppState {
    pub db_paths: Vec<PathBuf>,
}

/// Search request from web client
#[derive(Debug, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    #[serde(default = "default_selected_db")]
    pub selected_db: String,
    #[serde(default)]
    pub limit: Option<usize>,
    #[serde(default)]
    pub name_only: bool,
    #[serde(default)]
    pub case_sensitive: bool,
    #[serde(default)]
    pub root_path: Option<String>,
    #[serde(default)]
    pub include_filters: Option<String>,
    #[serde(default)]
    pub exclude_filters: Option<String>,
}

fn default_selected_db() -> String {
    "all".to_string()
}

/// Search response to web client
#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub success: bool,
    pub results: Vec<KeywordResults>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Results for a single keyword
#[derive(Debug, Serialize)]
pub struct KeywordResults {
    pub keyword: String,
    pub count: usize,
    pub tree: TreeNodeJson,
}

/// Index request from web client
#[derive(Debug, Deserialize)]
pub struct IndexRequest {
    pub root_path: String,
    pub db_path: String,
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,
    #[serde(default)]
    pub with_metadata: bool,
    #[serde(default)]
    pub incremental: bool,
}

fn default_batch_size() -> usize {
    5000
}

/// Index response to web client
#[derive(Debug, Serialize)]
pub struct IndexResponse {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_secs: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skipped_paths: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// JSON-serializable tree node
#[derive(Debug, Serialize, Clone)]
pub struct TreeNodeJson {
    pub name: String,
    pub path: String,
    pub is_leaf: bool,
    pub children: Vec<TreeNodeJson>,
}

impl From<&TreeNode> for TreeNodeJson {
    fn from(node: &TreeNode) -> Self {
        TreeNodeJson {
            name: node.name.clone(),
            path: node.path.to_string_lossy().to_string(),
            is_leaf: node.is_leaf(),
            children: node.children.iter().map(TreeNodeJson::from).collect(),
        }
    }
}

/// Parse filter keywords from a string (comma or space separated)
fn parse_filter_keywords(input: &str) -> Vec<String> {
    input
        .split([',', '，', ';', '；'])
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

/// Apply root path replacement to search results
///
/// This function replaces the original root path in database with a new one.
/// Useful when database is moved between different machines or mount points.
///
/// For Windows: Supports drive letter replacement (e.g., F:\ -> D:\)
/// For all systems: Supports full path prefix replacement
fn apply_root_path_replacement(
    results: Vec<(String, Vec<SearchResult>)>,
    new_root: &str,
) -> Vec<(String, Vec<SearchResult>)> {
    // Try to detect the common prefix from the first result
    let common_prefix = results
        .iter()
        .flat_map(|(_, items)| items.first())
        .map(|item| &item.path)
        .next()
        .and_then(|first_path| detect_root_prefix(first_path));

    if common_prefix.is_none() {
        return results; // No results or couldn't detect prefix
    }

    let old_prefix = common_prefix.unwrap();
    let new_root = new_root.trim_end_matches(['/', '\\']);

    results
        .into_iter()
        .map(|(keyword, items)| {
            let replaced_items = items
                .into_iter()
                .map(|mut item| {
                    item.path = replace_path_prefix(&item.path, &old_prefix, new_root);
                    item
                })
                .collect();
            (keyword, replaced_items)
        })
        .collect()
}

/// Detect the root prefix from a file path
/// For Windows: Returns drive letter + colon (e.g., "F:")
/// For Unix-like: Returns the first path component
fn detect_root_prefix(path: &str) -> Option<String> {
    // Windows drive letter detection
    if path.len() >= 2 && path.chars().nth(1) == Some(':') {
        let drive = path.chars().next()?;
        if drive.is_ascii_alphabetic() {
            return Some(format!("{}:", drive.to_ascii_uppercase()));
        }
    }

    // Unix-like: try to get the first component
    let path_obj = Path::new(path);
    path_obj
        .components()
        .next()
        .map(|c| c.as_os_str().to_string_lossy().to_string())
}

/// Replace the prefix of a path
fn replace_path_prefix(path: &str, old_prefix: &str, new_prefix: &str) -> String {
    if let Some(remainder) = path.strip_prefix(old_prefix) {
        // Handle both forward and backward slashes
        let remainder = remainder.trim_start_matches(['/', '\\']);

        if remainder.is_empty() {
            new_prefix.to_string()
        } else {
            format!("{}\\{}", new_prefix, remainder)
        }
    } else {
        path.to_string()
    }
}

/// Search handler
async fn search_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SearchRequest>,
) -> impl IntoResponse {
    // Configure search
    let config = SearchConfig {
        max_results: params.limit.unwrap_or(2000),
        search_in_path: !params.name_only,
        case_sensitive: params.case_sensitive,
        include_filters: params
            .include_filters
            .as_ref()
            .map(|s| parse_filter_keywords(s))
            .unwrap_or_default(),
        exclude_filters: params
            .exclude_filters
            .as_ref()
            .map(|s| parse_filter_keywords(s))
            .unwrap_or_default(),
    };

    // Parse keywords
    let keywords = parse_search_keywords(&params.query);

    // Perform multi-database search
    let results = match search_in_selected_database(&state.db_paths, &params.selected_db, &keywords, &config) {
        Ok(results) => results,
        Err(e) => {
            return Json(SearchResponse {
                success: false,
                results: vec![],
                error: Some(format!("Search failed: {}", e)),
            });
        }
    };

    // Group results by keyword (merge across databases if searching all)
    let mut keyword_map: std::collections::HashMap<String, Vec<SearchResult>> = std::collections::HashMap::new();
    
    for (_db_name, keyword, items) in results {
        keyword_map.entry(keyword).or_insert_with(Vec::new).extend(items);
    }

    // Apply root path replacement if specified
    let processed_results: Vec<(String, Vec<SearchResult>)> = keyword_map.into_iter().collect();
    let processed_results = if let Some(ref new_root) = params.root_path {
        apply_root_path_replacement(processed_results, new_root)
    } else {
        processed_results
    };

    // Build trees for each keyword
    let mut keyword_results = Vec::new();
    for (keyword, items) in processed_results {
        if items.is_empty() {
            keyword_results.push(KeywordResults {
                keyword,
                count: 0,
                tree: TreeNodeJson {
                    name: "无结果".to_string(),
                    path: ".".to_string(),
                    is_leaf: true,
                    children: vec![],
                },
            });
            continue;
        }

        let tree = build_tree(&items, &keyword);
        let tree_json = TreeNodeJson::from(&tree);

        keyword_results.push(KeywordResults {
            keyword,
            count: items.len(),
            tree: tree_json,
        });
    }

    Json(SearchResponse {
        success: true,
        results: keyword_results,
        error: None,
    })
}

/// Index handler - process indexing request
async fn index_handler(
    Json(req): Json<IndexRequest>,
) -> Result<Json<IndexResponse>, (StatusCode, Json<IndexResponse>)> {
    // Spawn blocking task for indexing (I/O intensive)
    let result = tokio::task::spawn_blocking(move || {
        // Open database
        let db = Database::new(&req.db_path);

        // Perform indexing based on mode
        let index_result = if req.incremental {
            indexer::scan_idxs_with_metadata(&req.root_path, &db, req.batch_size)
                .map_err(|e| format!("Indexing failed: {}", e))?
        } else if req.with_metadata {
            indexer::scan_idxs_with_metadata(&req.root_path, &db, req.batch_size)
                .map_err(|e| format!("Indexing failed: {}", e))?
        } else {
            indexer::scan_idxs(&req.root_path, &db, req.batch_size)
                .map_err(|e| format!("Indexing failed: {}", e))?
        };

        Ok::<_, String>(index_result)
    })
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(IndexResponse {
                success: false,
                message: String::new(),
                duration_secs: None,
                skipped_paths: None,
                error: Some(format!("Task join error: {}", e)),
            }),
        )
    })?;

    match result {
        Ok(index_result) => {
            let message = if index_result.skipped_paths.is_empty() {
                "Indexing completed successfully".to_string()
            } else {
                format!(
                    "Indexing completed with {} paths skipped due to permissions",
                    index_result.skipped_paths.len()
                )
            };

            Ok(Json(IndexResponse {
                success: true,
                message,
                duration_secs: Some(index_result.duration.as_secs_f64()),
                skipped_paths: if index_result.skipped_paths.is_empty() {
                    None
                } else {
                    Some(index_result.skipped_paths)
                },
                error: None,
            }))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(IndexResponse {
                success: false,
                message: String::new(),
                duration_secs: None,
                skipped_paths: None,
                error: Some(e),
            }),
        )),
    }
}

/// Root handler - serve the main HTML page
async fn root_handler() -> Html<&'static str> {
    Html(include_str!("../static/index.html"))
}

/// Indexer page handler - serve the indexer HTML page
async fn indexer_handler() -> Html<&'static str> {
    Html(include_str!("../static/indexer.html"))
}

/// Health check endpoint
async fn health_handler() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

/// Database list response
#[derive(Debug, Serialize)]
pub struct DatabaseListResponse {
    pub databases: Vec<DatabaseInfo>,
}

#[derive(Debug, Serialize)]
pub struct DatabaseInfo {
    pub name: String,
    pub path: String,
}

/// List available databases
async fn list_databases_handler(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let databases = state
        .db_paths
        .iter()
        .map(|path| DatabaseInfo {
            name: path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string(),
            path: path.to_string_lossy().to_string(),
        })
        .collect();

    Json(DatabaseListResponse { databases })
}

/// Create and configure the web application router
pub fn create_app(db_paths: Vec<PathBuf>) -> Router {
    let state = Arc::new(AppState { db_paths });

    Router::new()
        .route("/", get(root_handler))
        .route("/indexer", get(indexer_handler))
        .route("/api/search", get(search_handler))
        .route("/api/index", post(index_handler))
        .route("/api/databases", get(list_databases_handler))
        .route("/health", get(health_handler))
        .nest_service("/static", ServeDir::new("static"))
        .with_state(state)
}

/// Start the web server
pub async fn run_server(db_paths: Vec<PathBuf>, port: u16) -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let app = create_app(db_paths);

    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    tracing::info!("🌐 Web server running at http://localhost:{}", port);
    tracing::info!("📂 Press Ctrl+C to stop");

    axum::serve(listener, app).await?;

    Ok(())
}
