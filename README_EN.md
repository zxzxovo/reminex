# Reminex

<div align="center">

![Reminex Logo](https://via.placeholder.com/400x100/6366f1/ffffff?text=Reminex)

**⚡ High-Performance File Indexing & Search Engine ⚡**

[![Rust](https://img.shields.io/badge/rust-1.83%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)](https://github.com/yourusername/reminex)
[![AI Generated](https://img.shields.io/badge/AI-Generated-blueviolet.svg)](https://github.com/features/copilot)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](CONTRIBUTING.md)

[English](README_EN.md) | [中文文档](README.md)

</div>

---

> **🤖 AI-Powered Development**: This project is primarily developed with AI assistance (GitHub Copilot & Claude), showcasing the potential of AI-driven software engineering.

**Reminex** is a high-performance file indexing and searching tool designed for scenarios requiring fast file lookups across large datasets. By indexing file metadata into a SQLite database, it achieves millisecond-level search speeds.

---

## 🚀 Key Features

- **⚡ High-Speed Indexing**: Multi-threaded parallel scanning with Rayon and batch database writes
- **🔍 Fast Search**: SQLite full-text indexing with multi-keyword search support
- **📊 Metadata Extraction**: Automatically records file size, modification time, and other metadata
- **🌳 Tree Display**: Hierarchical directory tree visualization with automatic common path prefix detection
- **🔄 Incremental Updates**: Support for both full and incremental indexing modes
- **💬 Interactive Search**: Built-in interactive search interface
- **📋 Progress Display**: Real-time indexing progress and speed display for clear status tracking
- **⚙️ Database Optimization**: WAL mode + 2GB cache + batch transaction processing

---

## 💡 Use Cases

| Scenario | Description | Advantages |
|----------|-------------|-----------|
| 🌐 **NAS/Network Storage** | Slow network drive access, local indexing enables instant search | No repeated network scanning |
| 📚 **Large File Management** | Quick location and management of hundreds of thousands of files | Millisecond response time |
| 📦 **Archive Retrieval** | Fast querying of historical files and backup data | Offline indexing support |
| 🗂️ **Document Organization** | Quick filtering by file type, modification time, etc. | Flexible search criteria |
| 🎬 **Media Library** | Organization of large media files like photos and videos | Clear tree structure display |

---

## 🎯 Quick Start

### Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/reminex.git
cd reminex

# Build release version
cargo build --release

# Executable is located at:
# target/release/reminex.exe (Windows)
# target/release/reminex (Linux/macOS)
```

### Basic Usage

#### 1. Create Index

```bash
# Index a directory (fast mode, no metadata)
reminex index -p /path/to/directory -d myfiles.reminex.db

# Index with metadata (includes file size and modification time)
reminex index -p /path/to/directory -d myfiles.reminex.db --full

# Incremental update
reminex index -p /path/to/directory -d myfiles.reminex.db --no-metadata
```

#### 2. Search Files

```bash
# Basic search
reminex search -d myfiles.reminex.db keyword

# Multi-keyword search (supports ; or space delimiter)
reminex search -d myfiles.reminex.db "photo;2024"

# Tree display
reminex search -d myfiles.reminex.db -t keyword

# Interactive search mode
reminex search -d myfiles.reminex.db
```

---

## 📸 Screenshots

### Tree Display
```
$ reminex search -d myfiles.reminex.db -t photo

「photo」Found 99 results:

Search Results (Z:\)
├─ photos/
│  └─ 2023/
│     ├─ summer.jpg
│     └─ winter.jpg
└─ documents/
   └─ photo_report.pdf
```

---

## ⚡ Performance

Test Environment: Windows 11, Ryzen 7 5800H, NVMe SSD

| Operation | Speed | Scale | Notes |
|-----------|-------|-------|-------|
| Indexing | 129 files/sec | 10,000+ files | Network drive with metadata |
| Search | < 100ms | 100,000 files | Local database |
| DB Size | ~100 bytes/file | - | Without metadata |
| Memory | ~50MB | Indexing | Batch size 5000 |

---

## 🏗️ Architecture

```
src/
├── main.rs       # CLI entry point
├── lib.rs        # Library exports
├── db.rs         # Database abstraction layer
├── indexer.rs    # Parallel index scanning
└── searcher.rs   # Search and display
```

### Tech Stack

- **Database**: rusqlite 0.37.0
- **Parallelism**: rayon 1.11.0
- **Channels**: crossbeam-channel 0.5.15
- **Progress**: indicatif 0.17.10
- **CLI**: clap 4.5.53
- **Error Handling**: anyhow 1.0.100

---

## 🤖 AI Development Notes

This project demonstrates modern AI-assisted software development:

- **Initial Design**: Architecture and API design with AI collaboration
- **Code Implementation**: Core modules written with GitHub Copilot and Claude
- **Testing**: Unit tests and integration tests created by AI
- **Documentation**: README and code comments generated with AI assistance

**AI Tools Used:**
- GitHub Copilot - Real-time code completion
- Claude (Anthropic) - Architecture design and optimization
- Cursor - AI-powered IDE

**Human Oversight:**
- All code is reviewed and validated by humans
- Design decisions consider real-world use cases
- Performance benchmarks verified manually

---

## 🤝 Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for details.

---

## 📄 License

This project is licensed under the MIT License - see [LICENSE](LICENSE) file for details.

---

## 🔮 Roadmap

- [x] Progress bar display (using indicatif) ✅
- [ ] Incremental update optimization (based on mtime comparison)
- [ ] Configuration file support (TOML)
- [ ] Web interface
- [ ] Full-text content search
- [ ] Export results (CSV/JSON)
- [ ] Multi-database merged queries
- [ ] Cross-platform GUI application

---

<div align="center">

**Reminex** - File Search as Fast as Indexing ⚡

Made with 🤖 AI & ❤️ by Humans

[⭐ Star this project](https://github.com/yourusername/reminex) | [🐛 Report Bug](https://github.com/yourusername/reminex/issues) | [💡 Request Feature](https://github.com/yourusername/reminex/issues)

</div>
