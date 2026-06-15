use std::path::PathBuf;

pub const OUTPUT_SUFFIX: &str = "modern-dot.svg";

#[derive(Debug, Clone)]
pub struct DiagramBlock {
    pub graph_name: String,
    pub code: String,
    pub chapter_name: String,
    pub chapter_path: PathBuf,
    pub index: usize,
}

impl DiagramBlock {
    pub fn light_file_name(&self) -> String {
        format!("{}.{}", self.stem(), OUTPUT_SUFFIX)
    }

    pub fn dark_file_name(&self, dark_suffix: &str) -> String {
        self.light_file_name()
            .replace(OUTPUT_SUFFIX, &format!("modern-dot{dark_suffix}.svg"))
    }

    pub fn light_path(&self) -> PathBuf {
        self.chapter_path.join(self.light_file_name())
    }

    pub fn dark_path(&self, dark_suffix: &str) -> PathBuf {
        self.chapter_path.join(self.dark_file_name(dark_suffix))
    }

    fn stem(&self) -> String {
        if self.graph_name.is_empty() {
            format!("{}_{}", slug(&self.chapter_name), self.index)
        } else {
            format!(
                "{}_{}_{}",
                slug(&self.chapter_name),
                slug(&self.graph_name),
                self.index
            )
        }
    }
}

fn slug(content: &str) -> String {
    content
        .chars()
        .filter_map(|ch| {
            if ch.is_alphanumeric() {
                Some(ch.to_ascii_lowercase())
            } else if ch.is_whitespace() || matches!(ch, '_' | '-') {
                Some('_')
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn sample_block() -> DiagramBlock {
        DiagramBlock {
            graph_name: "Graph Name".into(),
            code: String::new(),
            chapter_name: "Test Chapter".into(),
            chapter_path: PathBuf::from("./book"),
            index: 0,
        }
    }

    #[test]
    fn light_file_name_uses_modern_dot_suffix() {
        assert_eq!(
            sample_block().light_file_name(),
            "test_chapter_graph_name_0.modern-dot.svg"
        );
    }

    #[test]
    fn dark_file_name_appends_configured_suffix() {
        assert_eq!(
            sample_block().dark_file_name("-dark"),
            "test_chapter_graph_name_0.modern-dot-dark.svg"
        );
    }

    #[test]
    fn paths_join_chapter_directory() {
        let block = sample_block();
        assert_eq!(
            block.light_path(),
            Path::new("./book/test_chapter_graph_name_0.modern-dot.svg")
        );
    }
}
