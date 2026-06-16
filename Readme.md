# mdbook-modern-dot

mdBook 预处理器：将 `modern-dot` 代码块渲染为 [Graphviz](https://graphviz.org/) 图表，支持明暗双主题。

## 安装

```shell
cargo install mdbook-modern-dot
brew install graphviz   # 构建时需要 dot
```

## 快速开始

```toml
# book.toml
[preprocessor.modern-dot]
command = "mdbook-modern-dot"
```

````markdown
```modern-dot
digraph { a -> b; }
```
````

## 文档

- 在线文档：<https://www159-used.github.io/mdbook-modern-dot/>
- 源码目录：[`book/`](book/)

本地预览：

```shell
mdbook serve book
```

## 开发

```shell
cargo test && cargo lint
```

## 许可证

AGPL-3.0-or-later — [LICENSE](LICENSE)
