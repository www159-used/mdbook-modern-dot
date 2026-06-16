# 主题系统

启用 `themed-output = true` 后，每张图会渲染两次：分别使用主题文件中的 `[light]` 与 `[dark]` token。

## 主题文件格式

示例（`themes/default.toml`）：

```toml
[light]
text = "#24292f"
node_fill = "#f6f7f9"
edge = "#5c6b7a"

[dark]
text = "#adbac7"
node_fill = "#2d333b"
edge = "#768390"

[preamble]
graph = 'graph [ bgcolor="transparent", fontcolor="{{ text }}" ];'
node = 'node [ style="filled", fillcolor="{{ node_fill }}", color="{{ node_stroke }}", fontcolor="{{ text }}" ];'
edge = 'edge [ color="{{ edge }}", fontcolor="{{ text }}" ];'
```

## dot 代码中的占位符

在图表中使用 `{{ token }}`：

```modern-dot
digraph G {
  graph [ bgcolor="transparent", fontcolor="{{ text }}" ];
  node  [ style=filled, fillcolor="{{ node_fill }}", fontcolor="{{ text }}" ];
  A -> B;
}
```

缺少 token 会导致构建失败，并列出未找到的键名。

## 自动 preamble 注入

若代码块中 **没有** `{{ ... }}` 占位符，会在图定义开括号 `{` 之后自动注入主题文件 `[preamble]` 中的属性。

已使用占位符的代码块不会注入 preamble，样式由你完全控制。

## CSS 切换

将 `modern-dot-theme.css` 复制到书籍目录，并通过 `additional-css` 引入。该文件在浅色主题下隐藏 `.diagram-dark`，在深色主题（Coal、Navy、Ayu）下隐藏 `.diagram-light`。

生成的 HTML 结构：

```html
<div class="mdbook-modern-dot-output theme-diagram">
  <div class="diagram-light"><svg>...</svg></div>
  <div class="diagram-dark"><svg>...</svg></div>
</div>
```
