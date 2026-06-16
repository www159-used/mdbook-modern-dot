# 简介

**mdbook-modern-dot** 是一个 [mdBook](https://rust-lang.github.io/mdBook/) 预处理器，可将 Markdown 中的围栏代码块渲染为 [Graphviz](https://graphviz.org/) 图表。

## 功能特性

- 构建时通过 `dot` 命令处理 `modern-dot` 代码块
- **内联 SVG**（默认）或 **输出到文件** 并以图片引用
- 可选 **明暗双主题渲染**，与 mdBook HTML 主题（Light/Rust 与 Coal/Navy/Ayu）联动
- 支持 TOML 主题文件、`{{ token }}` 占位符与自动 preamble 注入

## 工作原理

1. 在 Markdown 中编写 Graphviz dot 代码：

   ````markdown
   ```modern-dot
   digraph { a -> b; }
   ```
   ````

2. 执行 `mdbook build` 时，预处理器调用 `dot`，按需应用主题 token，并将代码块替换为 HTML（内联 SVG）或生成的图片文件。

3. 启用 `themed-output = true` 时，会同时生成明、暗两套 SVG，由 CSS 根据当前 mdBook 主题切换显示。

```modern-dot 流程概览
digraph {
  rankdir=LR;
  markdown [label="Markdown\n```modern-dot```", shape=box];
  preprocessor [label="mdbook-modern-dot", shape=box];
  dot [label="Graphviz dot", shape=box];
  html [label="HTML / SVG", shape=box];
  markdown -> preprocessor -> dot -> html;
}
```

本书本身即由 mdbook-modern-dot 构建——切换 mdBook 主题即可看到图表配色随之变化。
