use std::io;

use futures::join;
use mdbook_markdown::pulldown_cmark::{Event, LinkType, Tag, TagEnd};
use mdbook_preprocessor::errors::Result;

use crate::block::DiagramBlock;
use crate::config::Config;
use crate::dot::{render_to_file, render_to_svg};
use crate::html::{BLOCK_SEPARATOR, inline_diagram, themed_file_diagram, themed_inline_diagram};
use crate::theme::prepare_dot;
use crate::theme_css::ThemeCssInjector;

pub async fn render_diagram(
    block: DiagramBlock,
    config: &Config,
    theme_css: &ThemeCssInjector,
) -> Result<Vec<Event<'static>>> {
    let mut events = if config.output_to_file {
        render_to_files(&block, config, theme_css).await?
    } else {
        render_inline(&block, config, theme_css).await?
    };
    events.push(Event::Text("\n\n".into()));
    Ok(events)
}

async fn render_inline(
    block: &DiagramBlock,
    config: &Config,
    theme_css: &ThemeCssInjector,
) -> Result<Vec<Event<'static>>> {
    if config.themed_output {
        let (light_svg, dark_svg) = render_themed_svgs(&block.code, config).await?;
        Ok(themed_html_diagram_events(
            themed_inline_diagram(light_svg, dark_svg, &config.theme_wrapper_class),
            config,
            theme_css,
        ))
    } else {
        let svg = render_to_svg(&config.arguments, &block.code).await?;
        Ok(html_diagram_events(inline_diagram(svg)))
    }
}

async fn render_to_files(
    block: &DiagramBlock,
    config: &Config,
    theme_css: &ThemeCssInjector,
) -> Result<Vec<Event<'static>>> {
    if config.themed_output {
        let (light_dot, dark_dot) = themed_dot_sources(&block.code, config)?;
        let light_path = block.light_path();
        let dark_path = block.dark_path(&config.dark_suffix);
        let light_name = block.light_file_name();
        let dark_name = block.dark_file_name(&config.dark_suffix);

        let (light, dark) = join!(
            render_to_file(&config.arguments, &light_dot, &light_path),
            render_to_file(&config.arguments, &dark_dot, &dark_path),
        );
        light?;
        dark?;

        Ok(themed_html_diagram_events(
            themed_file_diagram(
                &config.theme_wrapper_class,
                &light_name,
                &dark_name,
                &block.graph_name,
            ),
            config,
            theme_css,
        ))
    } else {
        render_to_file(&config.arguments, &block.code, &block.light_path()).await?;
        Ok(file_image_events(
            &block.light_file_name(),
            &block.graph_name,
            config.link_to_file,
        ))
    }
}

fn file_image_events(file_name: &str, graph_name: &str, link_to_file: bool) -> Vec<Event<'static>> {
    let file_name = file_name.to_string();
    let graph_name = graph_name.to_string();
    let mut nodes = Vec::with_capacity(if link_to_file { 5 } else { 3 });

    if link_to_file {
        nodes.push(Event::Start(Tag::Link {
            link_type: LinkType::Inline,
            dest_url: file_name.clone().into(),
            title: graph_name.clone().into(),
            id: "".into(),
        }));
    }

    nodes.push(Event::Start(Tag::Image {
        link_type: LinkType::Inline,
        dest_url: file_name.into(),
        title: graph_name.into(),
        id: "".into(),
    }));
    nodes.push(Event::End(TagEnd::Image));

    if link_to_file {
        nodes.push(Event::End(TagEnd::Link));
    }

    nodes
}

fn html_diagram_events(html: String) -> Vec<Event<'static>> {
    vec![
        Event::Html(format!("{BLOCK_SEPARATOR}\n\n").into()),
        Event::Html(html.into()),
    ]
}

fn themed_html_diagram_events(
    html: String,
    config: &Config,
    theme_css: &ThemeCssInjector,
) -> Vec<Event<'static>> {
    let mut events = Vec::new();
    events.push(Event::Html(format!("{BLOCK_SEPARATOR}\n\n").into()));
    if config.inject_theme_css
        && let Some(style) = theme_css.take_style_tag()
    {
        events.push(Event::Html(style.into()));
    }
    events.push(Event::Html(html.into()));
    events
}

async fn render_themed_svgs(code: &str, config: &Config) -> Result<(String, String)> {
    let (light_dot, dark_dot) = themed_dot_sources(code, config)?;
    let (light_svg, dark_svg) = join!(
        render_to_svg(&config.arguments, &light_dot),
        render_to_svg(&config.arguments, &dark_dot),
    );
    Ok((light_svg?, dark_svg?))
}

fn themed_dot_sources(code: &str, config: &Config) -> Result<(String, String)> {
    let theme = config.theme.as_ref().ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidInput, "missing theme configuration")
    })?;

    let preamble = theme.preamble.as_ref();
    let light_dot = prepare_dot(code, &theme.light, preamble)?;
    let dark_dot = prepare_dot(code, &theme.dark, preamble)?;
    Ok((light_dot, dark_dot))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{PreambleConfig, ThemeFile};
    use std::collections::HashMap;

    fn sample_theme() -> ThemeFile {
        let mut light = HashMap::new();
        light.insert("text".to_string(), "#111111".to_string());
        let mut dark = HashMap::new();
        dark.insert("text".to_string(), "#eeeeee".to_string());
        ThemeFile {
            light,
            dark,
            preamble: None,
        }
    }

    #[test]
    fn themed_dot_sources_applies_light_and_dark_tokens() {
        let config = Config {
            themed_output: true,
            theme: Some(sample_theme()),
            ..Config::default()
        };

        let (light_dot, dark_dot) =
            themed_dot_sources("digraph G { node [fontcolor=\"{{ text }}\"]; }", &config).unwrap();
        assert!(light_dot.contains("#111111"));
        assert!(dark_dot.contains("#eeeeee"));
    }

    #[test]
    fn themed_dot_sources_injects_preamble_when_needed() {
        let config = Config {
            themed_output: true,
            theme: Some(ThemeFile {
                light: HashMap::from([("text".to_string(), "#111111".to_string())]),
                dark: HashMap::from([("text".to_string(), "#eeeeee".to_string())]),
                preamble: Some(PreambleConfig {
                    graph: Some("graph [fontcolor=\"{{ text }}\"];".to_string()),
                    node: None,
                    edge: None,
                }),
            }),
            ..Config::default()
        };

        let (light_dot, _) = themed_dot_sources("digraph G { A -> B; }", &config).unwrap();
        assert!(light_dot.contains("graph [fontcolor=\"#111111\"];"));
    }

    #[test]
    fn file_image_events_without_link() {
        let mut events = file_image_events("_name_0.modern-dot.svg", "Name", false);
        events.push(Event::Text("\n\n".into()));
        let mut iter = events.into_iter();

        assert!(matches!(iter.next(), Some(Event::Start(Tag::Image { .. }))));
        assert!(matches!(iter.next(), Some(Event::End(TagEnd::Image))));
        assert_eq!(iter.next(), Some(Event::Text("\n\n".into())));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn file_image_events_with_link() {
        let mut events = file_image_events("_name_0.modern-dot.svg", "Name", true);
        events.push(Event::Text("\n\n".into()));
        let mut iter = events.into_iter();

        assert!(matches!(iter.next(), Some(Event::Start(Tag::Link { .. }))));
        assert!(matches!(iter.next(), Some(Event::Start(Tag::Image { .. }))));
        assert!(matches!(iter.next(), Some(Event::End(TagEnd::Image))));
        assert!(matches!(iter.next(), Some(Event::End(TagEnd::Link))));
        assert_eq!(iter.next(), Some(Event::Text("\n\n".into())));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn inline_diagram_event_shape() {
        let events = html_diagram_events(inline_diagram("<svg/>".into()));
        let mut events = events;
        events.push(Event::Text("\n\n".into()));
        let mut iter = events.into_iter();

        assert!(matches!(
            iter.next(),
            Some(Event::Html(sep)) if sep.contains(BLOCK_SEPARATOR)
        ));
        assert!(matches!(iter.next(), Some(Event::Html(_))));
        assert_eq!(iter.next(), Some(Event::Text("\n\n".into())));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn themed_events_inject_builtin_css_once_per_chapter() {
        let config = Config {
            themed_output: true,
            inject_theme_css: true,
            ..Config::default()
        };

        let overview = ThemeCssInjector::default();
        let first = themed_html_diagram_events("<p>diagram</p>".into(), &config, &overview);
        let second = themed_html_diagram_events("<p>other</p>".into(), &config, &overview);

        assert_eq!(first.len(), 3);
        assert!(matches!(&first[1], Event::Html(html) if html.contains("mdbook-modern-dot-theme")));
        assert_eq!(second.len(), 2);

        let dominated_path = ThemeCssInjector::default();
        let other_chapter =
            themed_html_diagram_events("<p>diagram</p>".into(), &config, &dominated_path);
        assert_eq!(other_chapter.len(), 3);
        assert!(
            matches!(&other_chapter[1], Event::Html(html) if html.contains("mdbook-modern-dot-theme"))
        );
    }

    #[test]
    fn themed_style_survives_cmark_roundtrip() {
        use pulldown_cmark_to_cmark::cmark;

        let config = Config {
            themed_output: true,
            inject_theme_css: true,
            ..Config::default()
        };
        let theme_css = ThemeCssInjector::default();
        let events = themed_html_diagram_events("<div>d</div>".into(), &config, &theme_css);
        let mut buf = String::new();
        cmark(events.into_iter(), &mut buf).unwrap();
        assert!(
            buf.contains("mdbook-modern-dot-theme"),
            "expected style in markdown output: {buf}"
        );
    }
}
