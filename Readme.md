# mdbook-modern-dot

mdBook preprocessor that renders [Graphviz](https://graphviz.org/) diagrams from fenced code blocks, with optional **light/dark theme** support aligned with mdBook's HTML themes.

## Install

```shell
cargo install --path .
```

Install Graphviz (required at build time):

```shell
brew install graphviz
```

## Quick start

`book.toml`:

```toml
[preprocessor.modern-dot]
command = "mdbook-modern-dot"
```

In your markdown:

~~~markdown
```modern-dot
digraph {
    "processed" -> "graph"
}
```
~~~

Optional diagram title after the info string:

~~~markdown
```modern-dot Data Flow
digraph { a -> b; }
```
~~~

Blocks with other info strings (e.g. plain ` ```dot `) are left unchanged.

## Light/dark themes

Enable dual-theme rendering so diagrams follow mdBook theme selection (Light/Rust vs Coal/Navy/Ayu):

```toml
[preprocessor.modern-dot]
command = "mdbook-modern-dot"
themed-output = true
theme-file = "themes/default.toml"

[output.html]
additional-css = ["modern-dot-theme.css"]
```

Copy [`assets/modern-dot-theme.css`](assets/modern-dot-theme.css) into your book directory (or reference it via `additional-css`).

You do not write light/dark HTML yourself — when `themed-output = true`, the preprocessor renders both SVG variants and wraps them automatically.

| Option | Default | Description |
| --- | --- | --- |
| `themed-output` | `false` | Render light and dark SVG variants |
| `theme-file` | `themes/default.toml` | TOML theme tokens (relative to book root) |
| `dark-suffix` | `-dark` | Suffix for dark SVG files in file mode |
| `theme-wrapper-class` | `theme-diagram` | CSS class on themed output wrapper |
| `info-string` | `modern-dot` | Fenced-block marker to process |

Environment variables follow mdBook conventions:

```shell
MDBOOK_preprocessor__modern_dot__themed_output="true" mdbook build
```

### Theme file format

See [`themes/default.toml`](themes/default.toml). Use `{{ token }}` placeholders in dot code:

```modern-dot
digraph G {
  graph [ bgcolor="transparent", fontcolor="{{ text }}" ];
  node  [ style=filled, fillcolor="{{ node_fill }}", fontcolor="{{ text }}" ];
  A -> B;
}
```

When a diagram has **no** placeholders, default `[preamble]` attributes from the theme file are injected automatically.

### Generated HTML

**Inline mode** (default):

```html
<div class="mdbook-modern-dot-output theme-diagram">
  <div class="diagram-light"><svg>...</svg></div>
  <div class="diagram-dark"><svg>...</svg></div>
</div>
```

**File mode** (`output-to-file = true`):

- `chapter_0.modern-dot.svg` (light)
- `chapter_0.modern-dot-dark.svg` (dark)

```html
<p class="theme-diagram">
  <img class="diagram-light" src="chapter_0.modern-dot.svg" alt="">
  <img class="diagram-dark" src="chapter_0.modern-dot-dark.svg" alt="">
</p>
```

## Output to file

```toml
[preprocessor.modern-dot]
output-to-file = true
```

```shell
MDBOOK_preprocessor__modern_dot__output_to_file="true" mdbook build
```

Add to `.gitignore`:

```
*.modern-dot.svg
*.modern-dot-dark.svg
```

## Link to output file

When using `output-to-file`, wrap images in links via `link-to-file` (single-theme mode only):

```toml
[preprocessor.modern-dot]
output-to-file = true
link-to-file = true
```

## Embedding dot files

~~~markdown
```dot
{{#include path/to/file.dot}}
```
~~~

Ensure the preprocessor runs after `links`:

```toml
[preprocessor.modern-dot]
after = ["links"]
```

## Custom info string

Default marker is `modern-dot`. To use a different one:

```toml
[preprocessor.modern-dot]
info-string = "graphviz"
```

More about preprocessors: [mdBook preprocessors](https://rust-lang.github.io/mdBook/format/configuration/preprocessors.html).

## Development

Requires Rust **1.92+** (see [`rust-toolchain.toml`](rust-toolchain.toml)).

```shell
cargo test
cargo lint
cargo fmt-check
```

## License

GPL-3.0-or-later — see [LICENSE](LICENSE).
