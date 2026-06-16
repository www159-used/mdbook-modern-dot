# 安装

## 安装预处理器

从 [crates.io](https://crates.io/crates/mdbook-modern-dot) 安装：

```shell
cargo install mdbook-modern-dot
```

从源码安装：

```shell
cargo install --path .
```

## 安装 Graphviz

预处理器在构建时会调用 `dot` 命令，需单独安装 [Graphviz](https://graphviz.org/download/)：

```shell
# macOS
brew install graphviz

# Debian/Ubuntu
sudo apt install graphviz
```

验证安装：

```shell
dot -V
```

若未找到 `dot`，`mdbook build` 会失败并提示安装方法。

## 安装 mdBook

构建书籍还需要 mdBook：

```shell
cargo install mdbook
```
