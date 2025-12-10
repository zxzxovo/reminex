use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// 导出的搜索结果（TOML格式）
#[derive(Debug, Serialize, Deserialize)]
pub struct ExportedSearchResults {
    /// 导出元数据
    pub metadata: ExportMetadata,
    /// 搜索参数
    pub search_params: SearchParams,
    /// 搜索结果（按关键词分组）
    pub results: Vec<KeywordGroup>,
}

/// 导出元数据
#[derive(Debug, Serialize, Deserialize)]
pub struct ExportMetadata {
    /// 导出时间
    pub exported_at: DateTime<Utc>,
    /// Reminex 版本
    pub reminex_version: String,
    /// 结果总数
    pub total_count: usize,
}

/// 搜索参数
#[derive(Debug, Serialize, Deserialize)]
pub struct SearchParams {
    /// 搜索查询
    pub query: String,
    /// 选择的数据库
    pub selected_db: String,
    /// 是否仅搜索文件名
    #[serde(default)]
    pub name_only: bool,
    /// 是否区分大小写
    #[serde(default)]
    pub case_sensitive: bool,
    /// 结果限制
    #[serde(default)]
    pub limit: Option<usize>,
    /// 包含过滤器
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub include_filters: Vec<String>,
    /// 排除过滤器
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exclude_filters: Vec<String>,
}

/// 关键词结果组
#[derive(Debug, Serialize, Deserialize)]
pub struct KeywordGroup {
    /// 关键词
    pub keyword: String,
    /// 该关键词的结果数量
    pub count: usize,
    /// 文件路径列表
    pub files: Vec<FileEntry>,
}

/// 文件条目
#[derive(Debug, Serialize, Deserialize)]
pub struct FileEntry {
    /// 文件路径
    pub path: String,
    /// 文件大小（字节）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<i64>,
    /// 修改时间
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified: Option<String>,
}

impl ExportedSearchResults {
    /// 创建新的导出结果
    pub fn new(
        query: String,
        selected_db: String,
        name_only: bool,
        case_sensitive: bool,
        limit: Option<usize>,
        include_filters: Vec<String>,
        exclude_filters: Vec<String>,
    ) -> Self {
        Self {
            metadata: ExportMetadata {
                exported_at: Utc::now(),
                reminex_version: env!("CARGO_PKG_VERSION").to_string(),
                total_count: 0,
            },
            search_params: SearchParams {
                query,
                selected_db,
                name_only,
                case_sensitive,
                limit,
                include_filters,
                exclude_filters,
            },
            results: vec![],
        }
    }

    /// 添加关键词结果组
    pub fn add_keyword_group(&mut self, keyword: String, files: Vec<FileEntry>) {
        let count = files.len();
        self.metadata.total_count += count;
        self.results.push(KeywordGroup {
            keyword,
            count,
            files,
        });
    }

    /// 导出为 TOML 字符串
    pub fn to_toml(&self) -> Result<String> {
        Ok(toml::to_string_pretty(self)?)
    }

    /// 从 TOML 字符串导入
    pub fn from_toml(toml_str: &str) -> Result<Self> {
        Ok(toml::from_str(toml_str)?)
    }

    /// 导出到文件
    pub fn export_to_file(&self, path: &Path) -> Result<()> {
        let toml_content = self.to_toml()?;
        fs::write(path, toml_content)?;
        Ok(())
    }

    /// 从文件导入
    pub fn import_from_file(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        Self::from_toml(&content)
    }
}

/// 搜索结果转换参数
#[derive(Debug)]
pub struct ConvertParams {
    pub query: String,
    pub selected_db: String,
    pub name_only: bool,
    pub case_sensitive: bool,
    pub limit: Option<usize>,
    pub include_filters: Vec<String>,
    pub exclude_filters: Vec<String>,
    pub keyword_results: Vec<crate::web::KeywordResults>,
}

/// 从 Web API 的搜索结果转换为导出格式
pub fn convert_from_web_results(params: ConvertParams) -> ExportedSearchResults {
    let mut export = ExportedSearchResults::new(
        params.query,
        params.selected_db,
        params.name_only,
        params.case_sensitive,
        params.limit,
        params.include_filters,
        params.exclude_filters,
    );

    for kr in params.keyword_results {
        let files = flatten_tree_to_files(&kr.tree);
        export.add_keyword_group(kr.keyword, files);
    }

    export
}

/// 将树形结构扁平化为文件列表
fn flatten_tree_to_files(tree: &crate::web::TreeNodeJson) -> Vec<FileEntry> {
    let mut files = Vec::new();
    collect_files_recursive(tree, &mut files);
    files
}

fn collect_files_recursive(node: &crate::web::TreeNodeJson, files: &mut Vec<FileEntry>) {
    if node.is_leaf {
        files.push(FileEntry {
            path: node.path.clone(),
            size: None,
            modified: None,
        });
    } else {
        for child in &node.children {
            collect_files_recursive(child, files);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_import_roundtrip() {
        let mut export = ExportedSearchResults::new(
            "test query".to_string(),
            "test.db".to_string(),
            false,
            false,
            Some(100),
            vec![],
            vec![],
        );

        export.add_keyword_group(
            "keyword1".to_string(),
            vec![
                FileEntry {
                    path: "/path/to/file1.txt".to_string(),
                    size: Some(1024),
                    modified: Some("2024-01-01".to_string()),
                },
                FileEntry {
                    path: "/path/to/file2.txt".to_string(),
                    size: None,
                    modified: None,
                },
            ],
        );

        let toml_str = export.to_toml().unwrap();
        let imported = ExportedSearchResults::from_toml(&toml_str).unwrap();

        assert_eq!(imported.search_params.query, "test query");
        assert_eq!(imported.metadata.total_count, 2);
        assert_eq!(imported.results.len(), 1);
        assert_eq!(imported.results[0].keyword, "keyword1");
        assert_eq!(imported.results[0].files.len(), 2);
    }
}
