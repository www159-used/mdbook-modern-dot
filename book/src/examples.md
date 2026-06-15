# Examples

## [Graphviz](https://graphviz.org/)

The sample book enables `themed-output`, so diagrams render light and dark SVG variants
that follow mdBook theme selection. Write a single fenced block — no manual HTML tags.

### Themed diagram with explicit tokens

```modern-dot Themed Flow
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
    label="Input",
    fillcolor="{{ green_fill }}",
    color="{{ green_stroke }}"
  ];
  process [
    label="Process",
    fillcolor="{{ blue_fill }}",
    color="{{ blue_stroke }}"
  ];
  output [
    label="Output",
    fillcolor="{{ amber_fill }}",
    color="{{ amber_stroke }}"
  ];

  input -> process -> output;
}
```

### Simple diagram with automatic preamble injection

Diagrams without `{{ token }}` placeholders receive default graph/node/edge attributes
from `themes/default.toml`.

```modern-dot
digraph G {
  start -> middle -> end;
  start [shape=Mdiamond];
  end [shape=Msquare];
}
```

### Hard-coded colors (not theme-aware)

These colors stay fixed unless you replace them with theme tokens.

```modern-dot Legacy Colors
digraph G {
  subgraph cluster_0 {
    style=filled;
    color=lightgrey;
    node [style=filled,color=white];
    a0 -> a1 -> a2 -> a3;
    label = "process #1";
  }

  subgraph cluster_1 {
    node [style=filled];
    b0 -> b1 -> b2 -> b3;
    label = "process #2";
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
