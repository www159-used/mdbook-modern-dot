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
```

启用 `themed-output` 后，预处理器会在**每个含主题图的章节**首张图前自动注入内置 CSS，**无需**复制 `modern-dot-theme.css` 或配置 `additional-css`。

若需自定义样式，可设置 `inject-theme-css = false` 并自行引入 CSS 文件。

## 本仓库开发

开发本项目时，可指向本地 crate：

```toml
[preprocessor.modern-dot]
command = "cargo run -q --manifest-path ../Cargo.toml --bin mdbook-modern-dot"
themed-output = true
theme-file = "../themes/default.toml"
```
