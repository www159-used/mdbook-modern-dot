# 开发指南

## 环境要求

- Rust **1.92+**（见 [`rust-toolchain.toml`](https://github.com/www159-used/mdbook-modern-dot/blob/master/rust-toolchain.toml)）
- Graphviz（`dot` 在 PATH 中）
- mdBook

## 构建本文档

在仓库根目录执行：

```shell
mdbook build book
mdbook serve book
```

`book/` 目录既是项目手册，也是启用 `themed-output` 的集成示例。

## 常用命令

```shell
cargo test
cargo lint      # clippy + fmt 检查
cargo fmt-check
```

## 发布

使用 [cargo-release](https://github.com/crate-ci/cargo-release)：

```shell
cargo release patch -x --no-confirm
```

## 许可证

AGPL-3.0-or-later — 见 [LICENSE](https://github.com/www159-used/mdbook-modern-dot/blob/master/LICENSE)。
