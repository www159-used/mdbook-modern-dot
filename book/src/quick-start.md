# 快速开始

在 `book.toml` 中添加预处理器：

```toml
[preprocessor.modern-dot]
command = "mdbook-modern-dot"
```

在 Markdown 中编写图表：

````markdown
```modern-dot
digraph {
    "processed" -> "graph"
}
```
````

可在 info string 后添加可选标题：

````markdown
```modern-dot 数据流
digraph { input -> output; }
```
````

构建：

```shell
mdbook build
```

## 启用明暗主题

```toml
[preprocessor.modern-dot]
command = "mdbook-modern-dot"
themed-output = true
theme-file = "themes/default.toml"

[output.html]
additional-css = ["modern-dot-theme.css"]
```

将 [`modern-dot-theme.css`](https://github.com/www159-used/mdbook-modern-dot/blob/master/assets/modern-dot-theme.css) 复制到书籍目录（与 `book.toml` 同级）。

**无需**手写明暗 HTML——预处理器会自动渲染两套 SVG 并完成包裹。

## 本仓库开发

开发本项目时，可指向本地 crate：

```toml
[preprocessor.modern-dot]
command = "cargo run -q -p mdbook-modern-dot"
themed-output = true
theme-file = "../themes/default.toml"
```
