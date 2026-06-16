use std::sync::Mutex;

const THEME_CSS: &str = include_str!("../assets/modern-dot-theme.css");
const STYLE_ID: &str = "mdbook-modern-dot-theme";

pub struct ThemeCssInjector {
    emitted: Mutex<bool>,
}

impl Default for ThemeCssInjector {
    fn default() -> Self {
        Self {
            emitted: Mutex::new(false),
        }
    }
}

impl ThemeCssInjector {
    pub fn take_style_tag(&self) -> Option<String> {
        let mut emitted = self
            .emitted
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        if *emitted {
            return None;
        }
        *emitted = true;
        Some(format!("<style id=\"{STYLE_ID}\">\n{THEME_CSS}\n</style>"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emits_style_tag_only_once_per_chapter() {
        let injector = ThemeCssInjector::default();
        let first = injector.take_style_tag().unwrap();
        assert!(first.contains(STYLE_ID));
        assert!(first.contains(".theme-diagram"));
        assert!(injector.take_style_tag().is_none());

        let other_chapter = ThemeCssInjector::default();
        assert!(other_chapter.take_style_tag().is_some());
    }
}
