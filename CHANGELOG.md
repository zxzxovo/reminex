# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Real-time progress bar display during indexing using indicatif
- File count and scanning speed shown in progress indicator
- Visual feedback for both fast and full metadata scanning modes

### Changed
- Nothing yet

### Deprecated
- Nothing yet

### Removed
- Nothing yet

### Fixed
- Nothing yet

### Security
- Nothing yet

## [0.1.0] - 2025-12-01

### Added
- Initial release of Reminex
- File indexing with metadata support (size, modification time)
- Fast file search using SQLite FTS
- Multi-keyword search with delimiter support (`;`, space, comma)
- Tree-style search results display with automatic common prefix detection
- Interactive search mode
- Batch database operations for performance
- Three indexing modes: fast, full, and incremental
- Comprehensive CLI with short and long commands
- Parallel directory scanning using Rayon
- Producer-consumer pattern for efficient indexing
- WAL mode and optimized SQLite pragmas
- 37 unit tests with 100% pass rate

### Performance
- Indexing speed: ~129 files/second (network drive with metadata)
- Search latency: <100ms for 100K files
- Database size: ~100 bytes per file (without metadata)

### Documentation
- Comprehensive README with usage examples
- API documentation with rustdoc
- Contributing guidelines
- Security policy
- MIT License

---

## Release Notes

### v0.1.0 - Initial Release

This is the first public release of Reminex, a high-performance file indexing and searching tool.

**Highlights:**
- 🚀 Fast parallel indexing with Rayon
- 🔍 Millisecond-level search using SQLite
- 🌳 Beautiful tree-style result display
- 💬 Interactive search mode
- 🤖 Developed with AI assistance

**Known Limitations:**
- No file content search (planned for future)
- No automatic index updates (manual re-indexing required)
- No GUI (CLI only)

**Future Plans:**
- Progress bars during indexing
- Configuration file support (TOML)
- Web interface
- Full-text content search
- Export results to CSV/JSON

---

[Unreleased]: https://github.com/yourusername/reminex/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/yourusername/reminex/releases/tag/v0.1.0
