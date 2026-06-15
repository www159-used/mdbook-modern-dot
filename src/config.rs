use crate::theme::{load_themes, ThemeFile};
use mdbook_preprocessor::errors::{Error, Result};
use mdbook_preprocessor::PreprocessorContext;
use serde::Deserialize;
use toml::Table;

pub const PREPROCESSOR_NAME: &str = "modern-dot";
pub const DEFAULT_INFO_STRING: &str = "modern-dot";

#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct Config {
    pub output_to_file: bool,
    pub link_to_file: bool,
    pub info_string: String,
    pub arguments: Vec<String>,
    pub themed_output: bool,
    pub theme_file: Option<String>,
    pub dark_suffix: String,
    pub theme_wrapper_class: String,
    #[serde(skip)]
    pub theme: Option<ThemeFile>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            output_to_file: false,
            link_to_file: false,
            info_string: DEFAULT_INFO_STRING.to_string(),
            arguments: vec!["-Tsvg".into()],
            themed_output: false,
            theme_file: None,
            dark_suffix: "-dark".into(),
            theme_wrapper_class: "theme-diagram".into(),
            theme: None,
        }
    }
}

impl Config {
    pub fn load(ctx: &PreprocessorContext) -> Result<Self> {
        let mut config = Self::default();

        if let Some(table) = ctx
            .config
            .preprocessors::<Table>()?
            .get(PREPROCESSOR_NAME)
        {
            config = deserialize_table(table)?;
        }

        if config.themed_output {
            let theme_path = config
                .theme_file
                .take()
                .map(|path| ctx.root.join(path))
                .unwrap_or_else(|| ctx.root.join("themes/default.toml"));
            config.theme = Some(load_themes(&theme_path)?);
        }

        Ok(config)
    }
}

fn deserialize_table<T: for<'de> Deserialize<'de>>(table: &Table) -> Result<T> {
    let raw = toml::to_string(table).map_err(|error| {
        Error::from(std::io::Error::other(format!(
            "serialize preprocessor config: {error}"
        )))
    })?;
    toml::from_str(&raw).map_err(|error| {
        Error::from(std::io::Error::other(format!(
            "parse preprocessor config: {error}"
        )))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_expected_marker() {
        let config = Config::default();
        assert_eq!(config.info_string, "modern-dot");
        assert!(!config.themed_output);
    }

    #[test]
    fn deserializes_kebab_case_keys() {
        let table: Table = toml::from_str(
            r#"
            themed-output = true
            info-string = "custom"
            dark-suffix = "-night"
        "#,
        )
        .unwrap();
        let config: Config = deserialize_table(&table).unwrap();
        assert!(config.themed_output);
        assert_eq!(config.info_string, "custom");
        assert_eq!(config.dark_suffix, "-night");
    }
}
