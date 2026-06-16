# 输出模式

## 内联模式（默认）

`output-to-file = false` — SVG 直接嵌入章节 HTML。

适合大多数书籍：无额外文件，图表随页面一起发布。

## 文件模式

```toml
[preprocessor.modern-dot]
output-to-file = true
```

每张图在源章节旁生成 SVG 文件：

- `chapter_name_0.modern-dot.svg`（浅色）
- `chapter_name_0.modern-dot-dark.svg`（深色，需 `themed-output = true`）

Markdown 中的代码块会被替换为 `<img>` 标签（主题模式下为双图 HTML）。

### 链接到文件

单主题文件模式下，可将图片包裹在指向 SVG 的链接中：

```toml
[preprocessor.modern-dot]
output-to-file = true
link-to-file = true
```

启用 `themed-output = true` 时不使用此选项（主题文件输出使用固定 HTML 结构）。

## 文件命名规则

生成文件名由以下部分组成：

1. 规范化后的章节名
2. info string 中的可选图标题
3. 章节内代码块序号

示例：章节「Getting Started」、标题「Data Flow」、第一个块 → `getting_started_data_flow_0.modern-dot.svg`。

章节名与图标题中的非字母数字字符会规范化为下划线（仅保留 ASCII 字母数字）。
