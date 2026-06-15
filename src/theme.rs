use mdbook_preprocessor::errors::{Error, Result};
use regex::Regex;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;
use std::sync::OnceLock;

#[derive(Debug, Clone, Deserialize, Default)]
pub struct PreambleConfig {
    pub graph: Option<String>,
    pub node: Option<String>,
    pub edge: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ThemeFile {
    #[serde(default)]
    pub light: HashMap<String, String>,
    #[serde(default)]
    pub dark: HashMap<String, String>,
    #[serde(default)]
    pub preamble: Option<PreambleConfig>,
}

pub fn load_themes(path: &Path) -> Result<ThemeFile> {
    let raw = std::fs::read_to_string(path).map_err(|error| {
        Error::from(std::io::Error::other(format!(
            "read theme file {}: {error}",
            path.display()
        )))
    })?;
    toml::from_str(&raw).map_err(|error| {
        Error::from(std::io::Error::other(format!(
            "parse theme file {}: {error}",
            path.display()
        )))
    })
}

pub fn apply_theme(template: &str, theme: &HashMap<String, String>) -> Result<String> {
    static PLACEHOLDER: OnceLock<Regex> = OnceLock::new();
    let placeholder =
        PLACEHOLDER.get_or_init(|| Regex::new(r"\{\{\s*([a-zA-Z0-9_]+)\s*\}\}").unwrap());

    let mut missing = Vec::new();
    let rendered = placeholder
        .replace_all(template, |caps: &regex::Captures| {
            let key = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            match theme.get(key) {
                Some(value) => value.clone(),
                None => {
                    missing.push(key.to_string());
                    caps.get(0).map(|m| m.as_str()).unwrap_or("").to_string()
                }
            }
        })
        .into_owned();

    if !missing.is_empty() {
        missing.sort();
        missing.dedup();
        return Err(Error::from(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("missing theme keys: {}", missing.join(", ")),
        )));
    }

    Ok(rendered)
}

pub fn inject_preamble(
    code: &str,
    preamble: &PreambleConfig,
    theme: &HashMap<String, String>,
) -> Result<String> {
    let mut parts = Vec::new();
    if let Some(graph) = &preamble.graph {
        parts.push(apply_theme(graph, theme)?);
    }
    if let Some(node) = &preamble.node {
        parts.push(apply_theme(node, theme)?);
    }
    if let Some(edge) = &preamble.edge {
        parts.push(apply_theme(edge, theme)?);
    }

    if parts.is_empty() {
        return Ok(code.to_string());
    }

    let insert = format!("{}\n", parts.join("\n"));

    if let Some(pos) = code.find('{') {
        let mut out = String::with_capacity(code.len() + insert.len());
        out.push_str(&code[..=pos]);
        out.push('\n');
        out.push_str(&insert);
        out.push_str(&code[pos + 1..]);
        Ok(out)
    } else {
        Ok(format!("{insert}{code}"))
    }
}

pub fn prepare_dot(
    code: &str,
    theme: &HashMap<String, String>,
    preamble: Option<&PreambleConfig>,
) -> Result<String> {
    let code = if code.contains("{{") {
        code.to_string()
    } else if let Some(preamble) = preamble {
        inject_preamble(code, preamble, theme)?
    } else {
        code.to_string()
    };

    apply_theme(&code, theme)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_theme_replaces_placeholders() {
        let mut theme = HashMap::new();
        theme.insert("edge".to_string(), "#111111".to_string());
        let out = apply_theme("edge [color=\"{{ edge }}\"];", &theme).unwrap();
        assert_eq!(out, "edge [color=\"#111111\"];");
    }

    #[test]
    fn apply_theme_reports_missing_keys() {
        let theme = HashMap::new();
        let err = apply_theme("color=\"{{ missing }}\";", &theme).unwrap_err();
        assert!(err.to_string().contains("missing theme keys: missing"));
    }

    #[test]
    fn inject_preamble_inserts_after_opening_brace() {
        let mut theme = HashMap::new();
        theme.insert("text".to_string(), "#000".to_string());
        theme.insert("node_fill".to_string(), "#fff".to_string());
        theme.insert("node_stroke".to_string(), "#ccc".to_string());
        theme.insert("edge".to_string(), "#999".to_string());

        let preamble = PreambleConfig {
            graph: Some("graph [fontcolor=\"{{ text }}\"];".to_string()),
            node: Some("node [fillcolor=\"{{ node_fill }}\"];".to_string()),
            edge: None,
        };

        let out = inject_preamble("digraph G { A -> B; }", &preamble, &theme).unwrap();
        assert!(out.contains("digraph G {\ngraph [fontcolor=\"#000\"];"));
        assert!(out.contains("node [fillcolor=\"#fff\"];"));
        assert!(out.contains("A -> B;"));
    }

    #[test]
    fn prepare_dot_skips_preamble_when_placeholders_present() {
        let mut theme = HashMap::new();
        theme.insert("text".to_string(), "#111".to_string());

        let preamble = PreambleConfig {
            graph: Some("graph [fontcolor=\"{{ text }}\"];".to_string()),
            node: None,
            edge: None,
        };

        let out = prepare_dot(
            "digraph G { node [fontcolor=\"{{ text }}\"]; }",
            &theme,
            Some(&preamble),
        )
        .unwrap();
        assert_eq!(out, "digraph G { node [fontcolor=\"#111\"]; }");
        assert!(!out.contains("graph [fontcolor"));
    }
}
