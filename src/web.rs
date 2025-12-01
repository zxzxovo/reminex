use axum::{
    Router,
    extract::{Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Json},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tower_http::services::ServeDir;

use crate::db::Database;
use crate::indexer;
use crate::searcher::{SearchConfig, TreeNode, build_tree, search_from_input};

/// Web server state
#[derive(Clone)]
pub struct AppState {
    pub db_path: PathBuf,
}

/// Search request from web client
#[derive(Debug, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    #[serde(default)]
    pub limit: Option<usize>,
    #[serde(default)]
    pub name_only: bool,
    #[serde(default)]
    pub case_sensitive: bool,
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

/// Search handler
async fn search_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SearchRequest>,
) -> impl IntoResponse {
    // Open database
    let db = Database::new(&state.db_path);

    // Configure search
    let config = SearchConfig {
        max_results: params.limit.unwrap_or(2000),
        search_in_path: !params.name_only,
        case_sensitive: params.case_sensitive,
    };

    // Perform search
    let results = match search_from_input(&db, &params.query, &config) {
        Ok(results) => results,
        Err(e) => {
            return Json(SearchResponse {
                success: false,
                results: vec![],
                error: Some(format!("Search failed: {}", e)),
            });
        }
    };

    // Build trees for each keyword
    let mut keyword_results = Vec::new();
    for (keyword, items) in results {
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
        let duration = if req.incremental {
            indexer::scan_idxs_with_metadata(&req.root_path, &db, req.batch_size)
                .map_err(|e| format!("Indexing failed: {}", e))?
        } else if req.with_metadata {
            indexer::scan_idxs_with_metadata(&req.root_path, &db, req.batch_size)
                .map_err(|e| format!("Indexing failed: {}", e))?
        } else {
            indexer::scan_idxs(&req.root_path, &db, req.batch_size)
                .map_err(|e| format!("Indexing failed: {}", e))?
        };

        Ok::<_, String>(duration)
    })
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(IndexResponse {
                success: false,
                message: String::new(),
                duration_secs: None,
                error: Some(format!("Task join error: {}", e)),
            }),
        )
    })?;

    match result {
        Ok(duration) => Ok(Json(IndexResponse {
            success: true,
            message: "Indexing completed successfully".to_string(),
            duration_secs: Some(duration.as_secs_f64()),
            error: None,
        })),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(IndexResponse {
                success: false,
                message: String::new(),
                duration_secs: None,
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

/// Create and configure the web application router
pub fn create_app(db_path: PathBuf) -> Router {
    let state = Arc::new(AppState { db_path });

    Router::new()
        .route("/", get(root_handler))
        .route("/indexer", get(indexer_handler))
        .route("/api/search", get(search_handler))
        .route("/api/index", post(index_handler))
        .route("/health", get(health_handler))
        .nest_service("/static", ServeDir::new("static"))
        .with_state(state)
}

/// Start the web server
pub async fn run_server(db_path: PathBuf, port: u16) -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let app = create_app(db_path);

    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    tracing::info!("🌐 Web server running at http://localhost:{}", port);
    tracing::info!("📂 Press Ctrl+C to stop");

    axum::serve(listener, app).await?;

    Ok(())
}
