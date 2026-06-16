# 示例

以下图表由 **mdbook-modern-dot** 在 `themed-output = true` 下实时渲染。
切换 mdBook 主题（Light ↔ Coal/Navy/Ayu）即可看到配色变化。

## [Graphviz](https://graphviz.org/)

示例书籍启用了 `themed-output`，图表会生成明、暗两套 SVG，随 mdBook 主题切换。只需写一个围栏代码块，无需手写 HTML。

### 显式 token 的主题图

```modern-dot 主题流程
digraph G {
  graph [
    rankdir=LR,
    bgcolor="transparent",
    fontcolor="{{ text }}"
  ];
  node [
    shape=box,
    style="rounded,filled",
    fillcolor="{{ node_fill }}",
    color="{{ node_stroke }}",
    fontcolor="{{ text }}"
  ];
  edge [ color="{{ edge }}" ];

  input [
    label="输入",
    fillcolor="{{ green_fill }}",
    color="{{ green_stroke }}"
  ];
  process [
    label="处理",
    fillcolor="{{ blue_fill }}",
    color="{{ blue_stroke }}"
  ];
  output [
    label="输出",
    fillcolor="{{ amber_fill }}",
    color="{{ amber_stroke }}"
  ];

  input -> process -> output;
}
```

### 自动 preamble 注入的简单图

不含 `{{ token }}` 占位符的图表，会从 `themes/default.toml` 自动注入默认 graph/node/edge 属性。

```modern-dot
digraph G {
  start -> middle -> end;
  start [shape=Mdiamond];
  end [shape=Msquare];
}
```

### 硬编码颜色（不随主题变化）

以下颜色固定不变，除非改为主题 token。

```modern-dot 传统配色
digraph G {
  subgraph cluster_0 {
    style=filled;
    color=lightgrey;
    node [style=filled,color=white];
    a0 -> a1 -> a2 -> a3;
    label = "流程 #1";
  }

  subgraph cluster_1 {
    node [style=filled];
    b0 -> b1 -> b2 -> b3;
    label = "流程 #2";
    color=blue
  }
  start -> a0;
  start -> b0;
  a1 -> b3;
  b2 -> a3;
  a3 -> a0;
  a3 -> end;
  b3 -> end;

  start [shape=Mdiamond];
  end [shape=Msquare];
}
```
