use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// 搜索历史记录项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHistoryItem {
    /// 搜索查询字符串
    pub query: String,
    /// 选择的数据库
    pub selected_db: String,
    /// 搜索时间
    pub timestamp: DateTime<Utc>,
    /// 结果数量
    pub result_count: usize,
    /// 是否仅搜索文件名
    #[serde(default)]
    pub name_only: bool,
    /// 是否区分大小写
    #[serde(default)]
    pub case_sensitive: bool,
}

/// 搜索历史管理器
pub struct SearchHistory {
    history_file: PathBuf,
    max_entries: usize,
}

impl SearchHistory {
    /// 创建新的历史管理器
    pub fn new(history_file: PathBuf, max_entries: usize) -> Self {
        Self {
            history_file,
            max_entries,
        }
    }

    /// 获取默认历史文件路径
    pub fn default_path() -> PathBuf {
        if let Some(config_dir) = dirs::config_dir() {
            config_dir.join("reminex").join("search_history.json")
        } else {
            PathBuf::from(".reminex_history.json")
        }
    }

    /// 添加搜索记录
    pub fn add_entry(&self, item: SearchHistoryItem) -> Result<()> {
        let mut history = self.load_history()?;

        // 插入到开头
        history.insert(0, item);

        // 保持最大数量限制
        if history.len() > self.max_entries {
            history.truncate(self.max_entries);
        }

        self.save_history(&history)
    }

    /// 获取所有历史记录
    pub fn get_all(&self) -> Result<Vec<SearchHistoryItem>> {
        self.load_history()
    }

    /// 获取最近N条记录
    pub fn get_recent(&self, limit: usize) -> Result<Vec<SearchHistoryItem>> {
        let history = self.load_history()?;
        Ok(history.into_iter().take(limit).collect())
    }

    /// 清空历史记录
    pub fn clear(&self) -> Result<()> {
        self.save_history(&[])
    }

    /// 删除指定索引的记录
    pub fn remove(&self, index: usize) -> Result<()> {
        let mut history = self.load_history()?;
        if index < history.len() {
            history.remove(index);
            self.save_history(&history)?;
        }
        Ok(())
    }

    /// 加载历史记录
    fn load_history(&self) -> Result<Vec<SearchHistoryItem>> {
        if !self.history_file.exists() {
            return Ok(vec![]);
        }

        let content = fs::read_to_string(&self.history_file)?;
        let history: Vec<SearchHistoryItem> = serde_json::from_str(&content)?;
        Ok(history)
    }

    /// 保存历史记录
    fn save_history(&self, history: &[SearchHistoryItem]) -> Result<()> {
        // 确保目录存在
        if let Some(parent) = self.history_file.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(history)?;
        fs::write(&self.history_file, content)?;
        Ok(())
    }
}

// 需要添加 dirs crate 来获取配置目录
// 暂时使用简化版本
mod dirs {
    use std::path::PathBuf;

    pub fn config_dir() -> Option<PathBuf> {
        if cfg!(target_os = "windows") {
            std::env::var("APPDATA").ok().map(PathBuf::from)
        } else {
            std::env::var("HOME")
                .ok()
                .map(|home| PathBuf::from(home).join(".config"))
        }
    }
}
