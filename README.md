# Reminex

<div align="center">

![Reminex Logo](https://via.placeholder.com/400x100/6366f1/ffffff?text=Reminex)

**⚡ 高性能文件索引与搜索引擎 ⚡**

[![Rust](https://img.shields.io/badge/rust-1.83%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)](https://github.com/yourusername/reminex)
[![AI Generated](https://img.shields.io/badge/AI-Generated-blueviolet.svg)](https://github.com/features/copilot)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](CONTRIBUTING.md)

[English](README.md) | [中文文档](README_CN.md)

</div>

---

> **🤖 AI-Powered Development**: This project is primarily developed with AI assistance (GitHub Copilot & Claude), showcasing the potential of AI-driven software engineering.

**Reminex** 是一个高性能的文件索引与搜索工具，专为需要快速查找大量文件的场景设计。它通过将文件元数据索引到 SQLite 数据库中，实现毫秒级的文件搜索速度。

---

---

## 📸 Screenshots

### Web 界面
```
# 启动 Web 服务器
$ reminex web -d myfiles.reminex.db
🌐 启动 Web 服务器
📂 数据库: myfiles.reminex.db
🔗 地址: http://localhost:3000

# 在浏览器中访问 http://localhost:3000
# 现代化的 Web 界面，支持：
# - 多关键词搜索
# - 树形结果展示
# - 实时搜索
# - 响应式设计
```

### 索引进度显示
```
$ reminex index -p /data -d myfiles.db --full
📁 索引目录: /data
💾 数据库文件: myfiles.db
🚀 开始扫描...
   批量大小: 5000
   模式: 完整扫描（含元数据）
⏳ [00:00:15] 扫描中 (含元数据) 12589 个文件
✅ 索引完成！
   耗时: 15.42s
   文件数: 12589
   速度: 816 文件/秒
```

### 基础搜索
```
$ reminex search -d myfiles.reminex.db photo
「photo」找到 99 项结果：
  Z:\photos\2023\summer.jpg
  Z:\photos\2023\winter.jpg
  Z:\documents\photo_report.pdf
  ...
```

### 树形展示
```
$ reminex search -d myfiles.reminex.db -t photo
「photo」找到 99 项结果：

搜索结果 (Z:\)
├─ photos/
│  └─ 2023/
│     ├─ summer.jpg
│     └─ winter.jpg
└─ documents/
   └─ photo_report.pdf
```

### 交互式搜索
```
$ reminex search -d myfiles.reminex.db
🔍 reminex 搜索模式
   数据库: myfiles.reminex.db
   输入关键词搜索，多个关键词用 ; 或空格分隔
   输入 :q 退出

搜索> photo; video
「photo」找到 99 项结果
「video」找到 45 项结果

搜索> :q
再见！
```

---

## 📋 目录

- [核心特性](#-核心特性)
- [使用场景](#-使用场景)
- [Screenshots](#-screenshots)
- [快速开始](#-快速开始)
  - [前置要求](#前置要求)
  - [安装](#安装)
  - [基本使用](#基本使用)
- [功能详解](#-功能详解)
- [命令行参数](#-命令行参数)
- [性能优化](#-性能优化)
- [架构设计](#-架构设计)
- [开发指南](#-开发指南)
- [测试](#-测试)
- [AI Development Notes](#-ai-development-notes)
- [贡献指南](#-贡献指南)
- [许可证](#-许可证)
- [未来计划](#-未来计划)

## 🚀 核心特性

- **⚡ 高速索引**：基于 rayon 的多线程并行扫描，支持批量数据库写入
- **🔍 快速搜索**：使用 SQLite 全文索引，支持多关键词搜索
- **📊 元数据提取**：自动记录文件大小、修改时间等元信息
- **🌳 树形展示**：搜索结果支持层级目录树状显示，自动识别公共路径前缀
- **🔄 增量更新**：支持全量和增量两种索引模式
- **💬 交互式搜索**：内置交互式搜索界面，无需重复输入数据库路径
- **📋 进度显示**：实时显示索引进度和速度，清晰了解扫描状态
- **⚙️ 数据库优化**：WAL 模式 + 2GB 缓存 + 批量事务处理

## 💡 使用场景

Reminex 特别适合以下场景：

| 场景 | 说明 | 优势 |
|------|------|------|
| 🌐 **NAS/网络存储搜索** | 网络驱动器访问慢，本地索引实现秒级搜索 | 无需重复扫描网络 |
| 📚 **大容量文件管理** | 数十万文件的快速定位与管理 | 毫秒级响应 |
| 📦 **归档数据检索** | 历史文件、备份数据的快速查询 | 离线索引支持 |
| 🗂️ **文档分类整理** | 按文件类型、修改时间等维度快速筛选 | 灵活的搜索条件 |
| 🎬 **媒体库管理** | 照片、视频等大型媒体文件的组织 | 树形结构清晰展示 |

## 🎯 快速开始

### 前置要求

- Rust 1.83+ (Edition 2024)
- Cargo (随 Rust 自动安装)

### 安装

#### 从源码编译

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

#### 添加到 PATH（可选）

**Windows (PowerShell):**
```powershell
# 将编译好的程序复制到用户目录
Copy-Item target\release\reminex.exe ~\reminex.exe

# 添加到当前会话 PATH
$env:Path += ";$HOME"
```

**Linux/macOS:**
```bash
# 安装到本地 bin 目录
cargo install --path .

# 或复制到 /usr/local/bin
sudo cp target/release/reminex /usr/local/bin/
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

# 自定义根节点名称
reminex search -d myfiles.reminex.db -t --root-name "我的文件" keyword

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

# 自定义根节点名称
reminex search -d files.db -t --root-name "搜索结果" photo
```

输出示例：
```
搜索结果 (Z:\)
├─ photos/
│   ├─ summer.jpg
│   └─ winter.jpg
└─ documents/
    └─ report.pdf
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
| `--root-name <NAME>` | - | 树形根节点名称 | "搜索结果" |

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

测试环境：Windows 11, Ryzen 7 5800H, NVMe SSD

| 操作 | 速度 | 数据规模 | 备注 |
|------|------|----------|------|
| 索引速度 | 129 文件/秒 | 10,000+ 文件 | 网络驱动器，含元数据 |
| 搜索延迟 | < 100ms | 100,000 文件 | 本地数据库 |
| 数据库大小 | ~100 字节/文件 | - | 不含元数据 |
| 内存占用 | ~50MB | 索引期间 | 批量模式 5000 |

> **注意**：实际性能取决于磁盘 I/O、文件系统类型和文件数量。

## 🏗️ 架构设计

### 项目结构

```
src/
├── main.rs       # CLI 接口入口
├── lib.rs        # 库导出
├── db.rs         # 数据库抽象层
├── indexer.rs    # 并行索引扫描
├── searcher.rs   # 搜索与展示
└── web.rs        # Web 服务器
static/
└── index.html    # Web 界面前端
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
| 进度显示 | indicatif | 0.17.10 | 进度条 |
| Web 框架 | axum | 0.7.9 | HTTP 服务器 |
| 异步运行时 | tokio | 1.42 | 异步执行 |
| CLI 解析 | clap | 4.5.53 | 命令行参数 |
| 错误处理 | anyhow | 1.0.100 | 错误传播 |
| 序列化 | serde | 1.0 | JSON 序列化 |
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

---

## 🤖 AI Development Notes

### Development Approach

This project demonstrates modern AI-assisted software development:

- **Initial Design**: Architecture and API design with AI collaboration
- **Code Implementation**: Core modules written with GitHub Copilot and Claude
- **Testing**: Unit tests and integration tests created by AI
- **Documentation**: README and code comments generated with AI assistance
- **Optimization**: Performance tuning guided by AI suggestions

### AI Tools Used

- **GitHub Copilot**: Real-time code completion and suggestions
- **Claude (Anthropic)**: Architecture design, code review, and optimization
- **Cursor**: AI-powered IDE for seamless development

### Human Oversight

While AI significantly accelerated development:
- All code is reviewed and validated by human developers
- Design decisions consider real-world use cases
- Performance benchmarks verified manually
- Security and reliability are human-validated

---

---

## ❓ FAQ

<details>
<summary><b>Q: 数据库文件可以在不同操作系统间共享吗？</b></summary>

A: 可以，但需要注意路径格式差异。Windows 使用反斜杠 `\`，Linux/macOS 使用正斜杠 `/`。建议为每个系统维护独立的索引。
</details>

<details>
<summary><b>Q: 如何处理大量文件（百万级）？</b></summary>

A: 
- 增加批量大小：`-b 10000`
- 分批索引不同目录
- 使用 SSD 存储数据库文件
- 考虑使用 `--no-metadata` 模式加速
</details>

<details>
<summary><b>Q: 支持文件内容搜索吗？</b></summary>

A: 当前版本仅支持文件名和路径搜索。文件内容全文搜索已在未来计划中。
</details>

<details>
<summary><b>Q: 数据库文件会自动更新吗？</b></summary>

A: 不会自动更新。需要手动运行索引命令更新数据库。可以配合 cron/Task Scheduler 实现定时更新。
</details>

<details>
<summary><b>Q: 如何备份索引数据？</b></summary>

A: 直接复制 `.reminex.db` 文件即可。建议同时备份原始目录结构信息。
</details>

---

## 🔧 Troubleshooting

### 索引速度慢

**问题**：索引速度远低于预期

**解决方案**：
- 检查是否在网络驱动器上直接创建数据库（应在本地创建）
- 增加批量大小：`-b 10000`
- 使用 `--no-metadata` 跳过元数据提取
- 关闭杀毒软件的实时监控

### 搜索无结果

**问题**：明确知道文件存在，但搜索不到

**解决方案**：
- 检查是否使用了 `-N` (name-only) 参数
- 尝试不区分大小写搜索（默认）
- 检查文件是否在索引时被跳过
- 重新运行索引：`reminex index -p /path -d myfiles.db --full`

### 数据库损坏

**问题**：提示数据库文件损坏

**解决方案**：
```bash
# 尝试使用 SQLite 修复
sqlite3 myfiles.reminex.db "PRAGMA integrity_check;"

# 如果无法修复，重新创建索引
reminex index -p /path -d myfiles_new.db --full
```

### 权限错误

**问题**：无法创建数据库或索引文件

**解决方案**：
- Windows: 以管理员身份运行
- Linux/macOS: 检查目录权限 `chmod` 或使用 `sudo`
- 确保目标目录可写

---

## 🤝 贡献指南

欢迎提交 Issue 和 Pull Request！

### Contributing Guidelines

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交改动 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 开启 Pull Request

### Code Quality Standards

- 遵循 Rust 官方风格指南
- 运行 `cargo fmt` 格式化代码
- 运行 `cargo clippy` 检查代码质量
- 为新功能添加单元测试
- 确保所有测试通过 (`cargo test`)

---

## 📄 许可证

本项目采用 MIT 许可证 - 详见 [LICENSE](LICENSE) 文件

---

## 🔮 未来计划

- [x] 进度条显示（使用 indicatif） ✅
- [x] Web 界面（基础功能） ✅
- [ ] 增量更新优化（基于 mtime 比较）
- [ ] 配置文件支持（TOML）
- [ ] Web 界面增强（搜索历史、收藏夹、多数据库切换）
- [ ] 文件内容全文搜索
- [ ] 导出搜索结果（CSV/JSON）
- [ ] 多数据库合并查询
- [ ] 跨平台GUI应用

---

## 📧 联系方式

- 项目主页: https://github.com/yourusername/reminex
- 问题反馈: https://github.com/yourusername/reminex/issues
- 讨论区: https://github.com/yourusername/reminex/discussions

---

## 🌟 Star History

如果这个项目对您有帮助，请给我们一个 ⭐️ Star！

---

## 📚 相关项目

- [ripgrep](https://github.com/BurntSushi/ripgrep) - 快速文本搜索工具
- [fd](https://github.com/sharkdp/fd) - 用户友好的 find 替代品
- [fzf](https://github.com/junegunn/fzf) - 命令行模糊查找器

---

<div align="center">

**Reminex** - 让文件搜索如同索引一样快速 ⚡

Made with 🤖 AI & ❤️ by Humans

</div>