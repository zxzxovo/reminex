# Reminex

[![Rust](https://img.shields.io/badge/rust-1.83%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

**Reminex** 是一个高性能的文件索引与搜索工具，专为需要快速查找大量文件的场景设计。它通过将文件元数据索引到 SQLite 数据库中，实现毫秒级的文件搜索速度。

## 📋 目录

- [核心特性](#核心特性)
- [使用场景](#使用场景)
- [快速开始](#快速开始)
- [功能详解](#功能详解)
- [命令行参数](#命令行参数)
- [性能优化](#性能优化)
- [架构设计](#架构设计)
- [开发指南](#开发指南)
- [测试](#测试)

## 🚀 核心特性

- **高速索引**：基于 rayon 的多线程并行扫描，支持批量数据库写入
- **快速搜索**：使用 SQLite 全文索引，支持多关键词搜索
- **元数据提取**：自动记录文件大小、修改时间等元信息
- **树形展示**：搜索结果支持层级目录树状显示
- **增量更新**：支持全量和增量两种索引模式
- **交互式搜索**：内置交互式搜索界面，无需重复输入数据库路径
- **数据库优化**：WAL 模式 + 2GB 缓存 + 批量事务处理

## 💡 使用场景

Reminex 特别适合以下场景：

1. **NAS/网络存储搜索**：网络驱动器访问慢，本地索引可实现秒级搜索
2. **大容量文件管理**：数十万文件的快速定位与管理
3. **归档数据检索**：历史文件、备份数据的快速查询
4. **文档分类整理**：按文件类型、修改时间等维度快速筛选

## 🎯 快速开始

### 安装

```bash
# 克隆仓库
git clone https://github.com/yourusername/reminex.git
cd reminex

# 编译发布版本
cargo build --release

# 可执行文件位于
# target/release/reminex.exe (Windows)
# target/release/reminex (Linux/macOS)
```

### 基本使用

#### 1. 创建索引

```bash
# 索引单个目录（不包含元数据，速度最快）
reminex index -p /path/to/directory -d myfiles.reminex.db

# 索引并提取元数据（包含文件大小和修改时间）
reminex index -p /path/to/directory -d myfiles.reminex.db --full

# 增量更新（仅扫描新增和修改的文件）
reminex index -p /path/to/directory -d myfiles.reminex.db --no-metadata
```

#### 2. 搜索文件

```bash
# 基本搜索
reminex search -d myfiles.reminex.db keyword

# 多关键词搜索（支持 ; 或空格分隔）
reminex search -d myfiles.reminex.db "photo;2024"
reminex search -d myfiles.reminex.db photo 2024

# 树形显示搜索结果
reminex search -d myfiles.reminex.db -t keyword

# 自定义树形根节点
reminex search -d myfiles.reminex.db -t --root-name "我的文件" --root-path "." keyword

# 交互式搜索模式
reminex search -d myfiles.reminex.db
> photo;video
> report
> exit
```

## 📖 功能详解

### 索引模式

**快速模式（默认）**
```bash
reminex index -p /data -d files.db
```
- 仅索引文件路径和名称
- 速度最快，适合首次建立索引
- 测试数据：129 文件/秒

**完整模式（--full）**
```bash
reminex index -p /data -d files.db --full
```
- 提取文件大小、修改时间等元数据
- 支持按大小、时间范围搜索
- 适合需要详细信息的场景

**增量模式（--no-metadata）**
```bash
reminex index -p /data -d files.db --no-metadata
```
- 跳过元数据提取，仅更新路径
- 适合频繁更新的目录

### 搜索功能

**基础搜索**
```bash
# 单关键词
reminex search -d files.db photo

# 多关键词（AND 逻辑）
reminex search -d files.db "photo;vacation;2024"
```

**高级选项**
```bash
# 限制结果数量
reminex search -d files.db -l 10 keyword

# 仅搜索文件名（不搜索路径）
reminex search -d files.db -N keyword

# 区分大小写
reminex search -d files.db -c Keyword
```

**树形展示**
```bash
# 基础树形显示
reminex search -d files.db -t photo

# 自定义根节点
reminex search -d files.db -t --root-name "搜索结果" --root-path "/data" photo
```

输出示例：
```
搜索结果 (/data)
├── photos/
│   ├── summer.jpg
│   └── winter.jpg
└── documents/
    └── report.pdf
```

## 🔧 命令行参数

### Index 命令

```bash
reminex index [OPTIONS]
```

| 参数 | 简写 | 说明 | 默认值 |
|------|------|------|--------|
| `--path <PATH>` | `-p` | 要索引的目录路径 | **必需** |
| `--db <DATABASE>` | `-d` | 数据库文件路径 | **必需** |
| `--full` | `-f` | 提取完整元数据（大小、时间） | false |
| `--no-metadata` | `-n` | 不提取元数据（增量模式） | false |
| `--batch-size <SIZE>` | `-b` | 批量插入大小 | 1000 |

### Search 命令

```bash
reminex search [OPTIONS] [KEYWORDS]...
```

| 参数 | 简写 | 说明 | 默认值 |
|------|------|------|--------|
| `--db <DATABASE>` | `-d` | 数据库文件路径 | **必需** |
| `<KEYWORDS>...` | - | 搜索关键词（可选，无则进入交互模式） | - |
| `--limit <NUM>` | `-l` | 最大结果数量 | 无限制 |
| `--tree` | `-t` | 树形显示结果 | false |
| `--name-only` | `-N` | 仅搜索文件名 | false |
| `--case-sensitive` | `-c` | 区分大小写 | false |
| `--root-name <NAME>` | - | 树形根节点名称 | "Root" |
| `--root-path <PATH>` | - | 树形根节点路径 | "." |

## ⚡ 性能优化

### 数据库优化

Reminex 使用以下 SQLite 优化策略：

```sql
-- WAL 模式（Write-Ahead Logging）
PRAGMA journal_mode = WAL;

-- 异步写入
PRAGMA synchronous = OFF;

-- 2GB 缓存
PRAGMA cache_size = -2000000;

-- 内存临时存储
PRAGMA temp_store = MEMORY;
```

### 并行处理

- **多线程扫描**：使用 rayon 工作窃取调度器
- **生产者-消费者模式**：crossbeam-channel 解耦扫描与写入
- **批量事务**：默认 1000 条记录一次事务提交

### 性能基准

| 操作 | 速度 | 备注 |
|------|------|------|
| 索引速度 | 129 文件/秒 | 测试环境，实际速度取决于磁盘 I/O |
| 搜索延迟 | < 100ms | 10 万文件规模 |
| 数据库大小 | ~100 字节/文件 | 不含元数据 |

## 🏗️ 架构设计

### 项目结构

```
src/
├── main.rs       # CLI 接口入口
├── lib.rs        # 库导出
├── db.rs         # 数据库抽象层
├── indexer.rs    # 并行索引扫描
└── searcher.rs   # 搜索与展示
```

### 核心模块

**db.rs - 数据库层**
```rust
pub struct Database {
    path: PathBuf,
}

pub struct Index {
    pub path: String,
    pub name: String,
    pub mtime: Option<f64>,
    pub size: Option<i64>,
}

impl Database {
    pub fn init(path: impl AsRef<Path>) -> Result<Self>;
    pub fn add_idx(&self, idx: &Index) -> Result<()>;
    pub fn add_idxs(&self, idxs: &[Index]) -> Result<()>;
    pub fn batch_operation<F, R>(&self, f: F) -> Result<R>;
}
```

**indexer.rs - 索引模块**
```rust
pub fn scan_idxs<P: AsRef<Path>>(
    root: P,
    db: &Database,
    batch_size: usize,
) -> Result<Duration>;

pub fn scan_idxs_with_metadata<P: AsRef<Path>>(
    root: P,
    db: &Database,
    batch_size: usize,
) -> Result<Duration>;
```

**searcher.rs - 搜索模块**
```rust
pub struct SearchConfig {
    pub max_results: Option<usize>,
    pub search_in_path: bool,
    pub case_sensitive: bool,
}

pub fn search_by_keyword(
    db: &Database,
    keyword: &str,
    config: &SearchConfig,
) -> Result<Vec<SearchResult>>;

pub fn print_tree(
    results: &[SearchResult],
    root_name: &str,
    root_path: &str,
);
```

### 技术栈

| 组件 | 技术 | 版本 | 用途 |
|------|------|------|------|
| 数据库 | rusqlite | 0.37.0 | SQLite 绑定 |
| 并行处理 | rayon | 1.11.0 | 数据并行 |
| 通道通信 | crossbeam-channel | 0.5.15 | MPSC 通道 |
| CLI 解析 | clap | 4.5.53 | 命令行参数 |
| 错误处理 | anyhow | 1.0.100 | 错误传播 |
| 测试工具 | tempfile | 3.23.0 | 临时目录 |

## 🧪 测试

### 运行测试

```bash
# 运行所有测试
cargo test

# 详细输出
cargo test -- --nocapture

# 运行特定模块测试
cargo test db::tests
cargo test indexer::tests
cargo test searcher::tests
```

### 测试覆盖

- **db.rs**: 23 个单元测试
- **indexer.rs**: 5 个单元测试
- **searcher.rs**: 9 个单元测试
- **总计**: 37 个测试，100% 通过

### 代码质量

```bash
# Clippy 检查
cargo clippy --all-targets

# 格式化检查
cargo fmt --check
```

## 🛠️ 开发指南

### 环境要求

- Rust 1.83+ (Edition 2024)
- Cargo
- SQLite 3.x

### 编译

```bash
# 开发版本
cargo build

# 发布版本（优化）
cargo build --release

# 检查代码
cargo check
```

### 添加新功能

1. **修改数据库模式**：编辑 `db.rs` 中的 `init()` 方法
2. **添加索引逻辑**：在 `indexer.rs` 中实现扫描逻辑
3. **增强搜索**：在 `searcher.rs` 中添加过滤/排序
4. **扩展 CLI**：在 `main.rs` 中添加新命令或参数

### 代码风格

- 遵循 Rust 官方风格指南
- 使用 `cargo fmt` 格式化代码
- 使用 `cargo clippy` 检查代码质量
- 为新功能添加单元测试

## 📝 数据库架构

### 表结构

```sql
CREATE TABLE IF NOT EXISTS files (
    path TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    mtime REAL,
    size INTEGER
);

CREATE INDEX IF NOT EXISTS idx_name ON files(name);
```

### 字段说明

| 字段 | 类型 | 说明 | 是否必需 |
|------|------|------|----------|
| path | TEXT | 文件完整路径（主键） | 是 |
| name | TEXT | 文件名 | 是 |
| mtime | REAL | 修改时间（Unix 时间戳） | 否 |
| size | INTEGER | 文件大小（字节） | 否 |

## 🤝 贡献指南

欢迎提交 Issue 和 Pull Request！

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交改动 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 开启 Pull Request

## 📄 许可证

本项目采用 MIT 许可证 - 详见 [LICENSE](LICENSE) 文件

## 🔮 未来计划

- [ ] 增量更新优化（基于 mtime 比较）
- [ ] 进度条显示（使用 indicatif）
- [ ] 配置文件支持（TOML）
- [ ] Web 界面
- [ ] 文件内容全文搜索
- [ ] 导出搜索结果（CSV/JSON）
- [ ] 多数据库合并查询

## 📧 联系方式

- 项目主页: https://github.com/yourusername/reminex
- 问题反馈: https://github.com/yourusername/reminex/issues

---

**Reminex** - 让文件搜索如同索引一样快速 ⚡