# Reminex 新功能说明

## 版本 0.2.1 新功能

### 1. 多数据库选择功能

#### 功能描述
- 用户可以同时选择多个数据库进行搜索
- 使用 Material Design 风格的 Chip 组件进行可视化展示
- 支持单选、多选和全选模式
- 搜索结果会自动合并来自多个数据库的结果

#### 使用方法
1. 在 Web 界面上方会显示所有可用的数据库
2. 点击数据库 Chip 进行选择/取消选择（深蓝色表示已选中）
3. 可以点击"全部数据库"选择所有数据库
4. 默认自动选中第一个数据库
5. 至少需要选择一个数据库才能进行搜索

#### 技术实现
- **前端**：使用 JavaScript 管理 `selectedDatabases` 数组
- **后端**：修改 `SearchRequest` 结构支持逗号分隔的数据库列表
- **API 变更**：`selected_db` 参数现在接受逗号分隔的数据库名称

### 2. 自定义搜索分隔符

#### 功能描述
- 用户可以自定义用于分割搜索关键词的分隔符
- 默认分隔符：`;` `；` `,` `，` `\t`
- 可添加任意字符作为分隔符，包括特殊字符
- 支持删除不需要的分隔符（至少保留一个）
- 一键恢复默认设置
- 设置自动保存到浏览器本地存储

#### 使用方法
1. 在搜索框下方找到"自定义搜索分隔符"区域
2. 在输入框中输入要添加的分隔符：
   - 普通字符：直接输入（如 `|` `&` `#`）
   - 制表符：输入 `\t`
   - 空格：输入 `\s`
   - 换行符：输入 `\n`
3. 点击"添加"按钮或按 Enter 键添加
4. 点击分隔符 Chip 右侧的 × 按钮删除该分隔符
5. 点击"恢复默认"按钮重置为默认分隔符

#### 技术实现
- **前端**：
  - `customDelimiters` 数组存储自定义分隔符
  - `parseKeywordsWithCustomDelimiters()` 函数使用正则表达式解析关键词
  - localStorage 持久化保存设置
  
- **后端**：
  - 新增 `delimiters` 字段到 `SearchRequest` 结构（JSON 字符串）
  - 新增 `parse_search_keywords_with_delimiters()` 函数
  - 搜索处理器自动解析 JSON 格式的分隔符列表

### 3. 关键词高亮功能（已有功能增强）

#### 功能描述
- 搜索结果中的匹配关键词会自动高亮显示
- 可自定义高亮颜色
- 支持开关高亮功能

#### 使用方法
1. 勾选"高亮路径中的关键词"复选框启用高亮
2. 使用颜色选择器选择喜欢的颜色
3. 或者在文本框中输入颜色值：
   - 十六进制：`#ff7043`
   - RGB：`rgb(255, 112, 67)`
   - RGBA：`rgba(255, 112, 67, 0.8)`
   - HSL：`hsl(14, 100%, 63%)`
   - 颜色名称：`red`, `blue`, `orange` 等

## API 变更

### SearchRequest 结构更新

```rust
pub struct SearchRequest {
    pub query: String,
    pub selected_db: String,  // 现在接受逗号分隔的数据库列表，如 "db1,db2,db3"
    pub limit: Option<usize>,
    pub name_only: bool,
    pub case_sensitive: bool,
    pub root_path: Option<String>,
    pub include_filters: Option<String>,
    pub exclude_filters: Option<String>,
    pub delimiters: Option<String>,  // 新增：JSON 格式的分隔符数组
}
```

### 新增函数

```rust
// src/searcher.rs
pub fn parse_search_keywords_with_delimiters(
    input: &str, 
    delimiters: &[char]
) -> Vec<String>
```

## 测试

所有新功能都已通过测试：

```bash
$ cargo test
running 39 tests
test result: ok. 39 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

新增测试：
- `test_parse_search_keywords_with_custom_delimiters` - 测试自定义分隔符解析

## 兼容性说明

### 向后兼容
- 旧的搜索请求仍然可用（单数据库选择）
- 如果不提供 `delimiters` 参数，会使用默认分隔符
- 现有的搜索历史记录可以正常加载

### 前端要求
- 需要支持 localStorage 的现代浏览器
- 建议使用 Chrome 90+、Firefox 88+、Safari 14+ 或 Edge 90+

## 使用示例

### 多数据库搜索示例
```
选择数据库：[photos ✓] [documents ✓] [videos]

搜索关键词：vacation; 2023

结果：
- 来自 photos 数据库：150 个文件
- 来自 documents 数据库：25 个文件
总计：175 个文件
```

### 自定义分隔符示例
```
默认分隔符：; ； , ，  \t

添加自定义分隔符：| 和 &

搜索示例：
- "photo|video" → ["photo", "video"]
- "cat&dog;bird" → ["cat", "dog", "bird"]
- "my file,doc" → ["my file", "doc"]
```

## 注意事项

1. **分隔符冲突**：如果文件名中包含自定义分隔符，可能会导致意外的关键词分割
2. **性能考虑**：选择多个数据库会增加搜索时间，建议根据需要选择必要的数据库
3. **浏览器存储**：清除浏览器缓存会丢失自定义设置（分隔符和颜色偏好）

## 未来计划

- [ ] 添加分隔符模板（预设常用分隔符组合）
- [ ] 支持正则表达式分隔符
- [ ] 数据库标签和分组功能
- [ ] 搜索结果去重选项
- [ ] 导出时包含多数据库来源信息

---

**更新日期**: 2024-01-XX  
**版本**: 0.2.1
