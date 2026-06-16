# 配置项

所有选项均写在 `book.toml` 的 `[preprocessor.modern-dot]` 下。

| 选项 | 默认值 | 说明 |
| --- | --- | --- |
| `command` | *（必填）* | `mdbook-modern-dot` 可执行文件路径 |
| `themed-output` | `false` | 渲染明、暗两套 SVG |
| `inject-theme-css` | `true` | 在每个含主题图的章节首张图前自动注入内置主题切换 CSS（`themed-output = true` 时生效） |
| `theme-file` | `themes/default.toml` | 主题 TOML 路径（相对书籍根目录） |
| `dark-suffix` | `-dark` | 文件模式下暗色 SVG 文件名后缀 |
| `theme-wrapper-class` | `theme-diagram` | 主题输出外层 CSS 类名 |
| `info-string` | `modern-dot` | 要处理的围栏代码块标记 |
| `output-to-file` | `false` | 输出 SVG 文件而非内联 HTML |
| `link-to-file` | `false` | 文件模式下用链接包裹图片（仅单主题） |
| `arguments` | `["-Tsvg"]` | 传给 `dot` 的额外参数 |
| `after` | — | 在其他预处理器之后运行（如 `["links"]`） |

## 环境变量

mdBook 支持用环境变量覆盖配置：

```shell
MDBOOK_preprocessor__modern_dot__themed_output="true" mdbook build
MDBOOK_preprocessor__modern_dot__output_to_file="true" mdbook build
```

选项名中的连字符 `-` 变为下划线 `_`；嵌套键使用双下划线 `__`。

## 自定义 info string

仅处理与 `info-string` 匹配的代码块，其他标记（如普通 ` ```dot `）保持不变。

```toml
[preprocessor.modern-dot]
info-string = "graphviz"
```

## 嵌入外部 dot 文件

配合 mdBook 的 `links` 预处理器引入 dot 源码：

````markdown
```dot
&#123;&#123;#include path/to/diagram.dot&#125;&#125;
```
````

确保 modern-dot 在 links **之后**运行：

```toml
[preprocessor.modern-dot]
after = ["links"]
```

## 文件模式的 `.gitignore`

使用 `output-to-file = true` 时，建议添加：

```
*.modern-dot.svg
*.modern-dot-dark.svg
```
