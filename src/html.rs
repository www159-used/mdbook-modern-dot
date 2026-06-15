use regex::Regex;
use std::sync::OnceLock;

pub const OUTPUT_CLASS: &str = "mdbook-modern-dot-output";
/// Breaks out of an open mdBook heading before block diagram HTML.
pub const BLOCK_SEPARATOR: &str = "<!-- mdbook-modern-dot -->";

fn svg_sanitize_regexes() -> &'static [Regex; 4] {
    static RE: OnceLock<[Regex; 4]> = OnceLock::new();
    RE.get_or_init(|| {
        [
            Regex::new(r"<!DOCTYPE [^>]+>").unwrap(),
            Regex::new(r"<\?xml [^>]+\?>").unwrap(),
            Regex::new(r">\s+<").unwrap(),
            Regex::new(r"\n").unwrap(),
        ]
    })
}

pub fn sanitize_svg(output: String) -> String {
    let re = svg_sanitize_regexes();
    let output = re[0].replace_all(&output, "");
    let output = re[1].replace_all(&output, "");
    let output = re[2].replace_all(&output, "><");
    let output = re[3].replace_all(&output, "");
    output.trim().to_string()
}

pub fn inline_diagram(svg: String) -> String {
    format!(
        "<div class=\"{OUTPUT_CLASS}\">{svg}</div>",
        svg = sanitize_svg(svg)
    )
}

pub fn themed_inline_diagram(light_svg: String, dark_svg: String, wrapper_class: &str) -> String {
    format!(
        "<div class=\"{OUTPUT_CLASS} {wrapper_class}\"><div class=\"diagram-light\">{light}</div><div class=\"diagram-dark\">{dark}</div></div>",
        light = sanitize_svg(light_svg),
        dark = sanitize_svg(dark_svg),
    )
}

pub fn themed_file_diagram(
    wrapper_class: &str,
    light_src: &str,
    dark_src: &str,
    alt: &str,
) -> String {
    format!(
        "<p class=\"{wrapper_class}\"><img class=\"diagram-light\" src=\"{light_src}\" alt=\"{alt}\"><img class=\"diagram-dark\" src=\"{dark_src}\" alt=\"{alt}\"></p>",
        light_src = escape_html(light_src),
        dark_src = escape_html(dark_src),
        alt = escape_html(alt),
    )
}

pub fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_separator_is_html_comment() {
        assert!(BLOCK_SEPARATOR.starts_with("<!--"));
    }

    #[test]
    fn themed_inline_includes_both_variants() {
        let html = themed_inline_diagram(
            "<svg>light</svg>".into(),
            "<svg>dark</svg>".into(),
            "theme-diagram",
        );
        assert!(html.contains(OUTPUT_CLASS));
        assert!(html.contains("diagram-light"));
        assert!(html.contains("diagram-dark"));
    }
}
