# 根路径替换功能使用说明

## 功能概述

根路径替换功能允许您在搜索时将数据库中存储的原始路径替换为新的根路径。这在以下场景非常有用：

- 📁 数据库在不同机器间迁移
- 💽 Windows 下盘符变更（如 F:\ → D:\）
- 🔄 网络驱动器挂载点变化
- 🐧 跨平台使用（Windows ↔ Linux）

## 使用方法

### Web 界面

1. 访问搜索页面 `http://localhost:3000`
2. 在搜索框右侧找到 **"显示根路径"** 输入框
3. 输入新的根路径，例如：
   - Windows: `D:\` 或 `E:\MyFiles`
   - Linux/Mac: `/mnt/data` 或 `/home/user/documents`
4. 执行搜索，结果将显示替换后的路径

### 示例

#### 场景 1: Windows 盘符替换

**数据库原始路径:**
```
F:\Documents\report.pdf
F:\Photos\2024\vacation.jpg
```

**输入显示根路径:** `D:\`

**搜索结果显示:**
```
D:\Documents\report.pdf
D:\Photos\2024\vacation.jpg
```

#### 场景 2: 完整路径替换

**数据库原始路径:**
```
F:\NAS\SharedFiles\project\code.rs
F:\NAS\SharedFiles\docs\readme.md
```

**输入显示根路径:** `Z:\MyNAS`

**搜索结果显示:**
```
Z:\MyNAS\SharedFiles\project\code.rs
Z:\MyNAS\SharedFiles\docs\readme.md
```

#### 场景 3: 跨平台路径映射

**数据库原始路径:** (Windows)
```
F:\Data\dataset.csv
```

**输入显示根路径:** `/mnt/windows_f`

**搜索结果显示:**
```
/mnt/windows_f\Data\dataset.csv
```

## 技术细节

### 自动检测原始根路径

系统会自动从搜索结果中检测原始根路径：

- **Windows**: 检测盘符（如 `F:`）
- **Unix-like**: 检测第一个路径组件

### 替换逻辑

1. 从第一个搜索结果中提取原始根路径前缀
2. 将所有结果中的原始前缀替换为用户指定的新根路径
3. 保持子路径结构不变
4. 统一使用反斜杠 `\` 作为路径分隔符（Windows 风格）

### API 参数

搜索 API 端点 `/api/search` 支持可选参数 `root_path`:

```
GET /api/search?query=keyword&root_path=D:\
```

**参数说明:**
- `root_path` (可选): 新的根路径字符串
- 如果不提供，则显示原始数据库中的路径

## 注意事项

⚠️ **仅影响显示，不修改数据库**

- 路径替换只在搜索结果显示时生效
- 原始数据库文件完全不受影响
- 每次搜索可以使用不同的显示根路径

⚠️ **路径分隔符**

- 当前实现统一使用反斜杠 `\`
- 即使输入 Linux 风格路径，内部也会转换为 `\`

⚠️ **空值处理**

- 如果留空 "显示根路径" 输入框，则使用原始路径
- 不会影响搜索功能本身

## 实现代码

核心实现位于 `src/web.rs`:

```rust
fn apply_root_path_replacement(
    results: Vec<(String, Vec<SearchResult>)>,
    new_root: &str,
) -> Vec<(String, Vec<SearchResult>)>
```

该函数会：
1. 检测原始根路径前缀
2. 遍历所有搜索结果
3. 替换路径前缀
4. 返回处理后的结果

## 相关文件

- `src/web.rs` - 后端路径替换逻辑
- `static/index.html` - 前端输入界面
- `src/searcher.rs` - 搜索和树构建逻辑

---

**版本:** v0.2.0  
**更新日期:** 2025-12-01
