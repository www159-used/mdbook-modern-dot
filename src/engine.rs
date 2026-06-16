use async_recursion::async_recursion;
use core::mem;
use futures::future;
use mdbook_markdown::pulldown_cmark::CodeBlockKind::Fenced;
use mdbook_markdown::pulldown_cmark::{Event, Tag, TagEnd};
use mdbook_markdown::{MarkdownOptions, new_cmark_parser};
use mdbook_preprocessor::book::{Book, BookItem, Chapter};
use mdbook_preprocessor::errors::Result;
use pulldown_cmark_to_cmark::cmark;
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;

use crate::block::DiagramBlock;
use crate::config::Config;
use crate::render::render_diagram;
use crate::theme_css::ThemeCssInjector;

pub struct Engine {
    src_dir: PathBuf,
    config: Config,
    theme_css: Arc<ThemeCssInjector>,
}

impl Engine {
    pub fn new(src_dir: PathBuf, config: Config) -> Self {
        Self {
            src_dir,
            config,
            theme_css: Arc::new(ThemeCssInjector::default()),
        }
    }

    pub async fn process_book(&self, book: &mut Book) -> Result<()> {
        self.process_sub_items(&mut book.items).await
    }

    #[async_recursion(?Send)]
    async fn process_sub_items(&self, items: &mut Vec<BookItem>) -> Result<()> {
        let mut item_futures = Vec::with_capacity(items.len());
        for item in mem::take(items) {
            item_futures.push(async {
                match item {
                    BookItem::Chapter(chapter) => {
                        self.process_chapter(chapter).await.map(BookItem::Chapter)
                    }
                    item => Ok(item),
                }
            });
        }

        *items = future::join_all(item_futures)
            .await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;

        Ok(())
    }

    #[async_recursion(?Send)]
    pub async fn process_chapter(&self, mut chapter: Chapter) -> Result<Chapter> {
        self.process_sub_items(&mut chapter.sub_items).await?;

        let Some(chapter_rel_path) = chapter.path.as_ref() else {
            return Ok(chapter);
        };

        let mut chapter_path = self.src_dir.join(chapter_rel_path);
        chapter_path.pop();

        let mut buf = String::with_capacity(chapter.content.len());
        let mut block_builder: Option<BlockBuilder> = None;
        let mut image_index = 0;

        let events = new_cmark_parser(&chapter.content, &MarkdownOptions::default());
        enum ChapterPiece {
            PassThrough(Event<'static>),
            Diagram(usize),
        }

        let mut pieces = Vec::<ChapterPiece>::new();
        let mut diagram_futures =
            Vec::<Pin<Box<dyn Future<Output = Result<Vec<Event<'static>>>>>>>::new();

        for event in events {
            if let Some(mut builder) = block_builder.take() {
                match event {
                    Event::Text(ref text) => {
                        builder.append_code(text);
                        block_builder = Some(builder);
                    }
                    Event::End(TagEnd::CodeBlock) => {
                        let block = builder.build(image_index);
                        image_index += 1;

                        pieces.push(ChapterPiece::Diagram(diagram_futures.len()));
                        let config = self.config.clone();
                        let theme_css = Arc::clone(&self.theme_css);
                        diagram_futures.push(Box::pin(async move {
                            render_diagram(block, &config, &theme_css).await
                        }));
                    }
                    _ => {
                        block_builder = Some(builder);
                    }
                }
            } else if let Event::Start(Tag::CodeBlock(Fenced(info_string))) = &event {
                let prefix_len = self.config.info_string.len();
                let (prefix, graph_name) = info_string.split_at(info_string.len().min(prefix_len));
                if prefix == self.config.info_string {
                    block_builder = Some(BlockBuilder::new(
                        chapter_path.clone(),
                        chapter.name.trim().to_string(),
                        graph_name.trim().to_string(),
                    ));
                    continue;
                }
                pieces.push(ChapterPiece::PassThrough(event.into_static()));
            } else {
                pieces.push(ChapterPiece::PassThrough(event.into_static()));
            }
        }

        let diagram_results = future::join_all(diagram_futures)
            .await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;

        let mut output_events = Vec::new();
        for piece in pieces {
            match piece {
                ChapterPiece::PassThrough(event) => output_events.push(event),
                ChapterPiece::Diagram(index) => {
                    output_events.extend(diagram_results[index].iter().cloned())
                }
            }
        }

        cmark(output_events.into_iter(), &mut buf)?;
        chapter.content = buf;

        Ok(chapter)
    }
}

struct BlockBuilder {
    path: PathBuf,
    chapter_name: String,
    graph_name: String,
    code: String,
}

impl BlockBuilder {
    fn new(path: PathBuf, chapter_name: String, graph_name: String) -> Self {
        Self {
            path,
            chapter_name,
            graph_name,
            code: String::new(),
        }
    }

    fn append_code(&mut self, code: &str) {
        self.code.push_str(code);
    }

    fn build(self, index: usize) -> DiagramBlock {
        DiagramBlock {
            graph_name: self.graph_name,
            code: self.code.trim().into(),
            chapter_name: self.chapter_name,
            chapter_path: self.path,
            index,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    const CHAPTER_NAME: &str = "Test Chapter";
    const NORMALIZED_CHAPTER_NAME: &str = "test_chapter";

    struct RecordingEngine {
        src_dir: PathBuf,
        config: Config,
    }

    impl RecordingEngine {
        async fn process_chapter(&self, mut chapter: Chapter) -> Result<Chapter> {
            self.process_sub_items(&mut chapter.sub_items).await?;

            let Some(chapter_rel_path) = chapter.path.as_ref() else {
                return Ok(chapter);
            };

            let mut chapter_path = self.src_dir.join(chapter_rel_path);
            chapter_path.pop();

            let mut buf = String::with_capacity(chapter.content.len());
            let mut block_builder: Option<BlockBuilder> = None;
            let mut image_index = 0;

            let events = new_cmark_parser(&chapter.content, &MarkdownOptions::default());
            enum ChapterPiece {
                PassThrough(Event<'static>),
                Diagram(DiagramBlock),
            }

            let mut pieces = Vec::<ChapterPiece>::new();

            for event in events {
                if let Some(mut builder) = block_builder.take() {
                    match event {
                        Event::Text(ref text) => {
                            builder.append_code(text);
                            block_builder = Some(builder);
                        }
                        Event::End(TagEnd::CodeBlock) => {
                            pieces.push(ChapterPiece::Diagram(builder.build(image_index)));
                            image_index += 1;
                        }
                        _ => {
                            block_builder = Some(builder);
                        }
                    }
                } else if let Event::Start(Tag::CodeBlock(Fenced(info_string))) = &event {
                    let prefix_len = self.config.info_string.len();
                    let (prefix, graph_name) =
                        info_string.split_at(info_string.len().min(prefix_len));
                    if prefix == self.config.info_string {
                        block_builder = Some(BlockBuilder::new(
                            chapter_path.clone(),
                            chapter.name.trim().to_string(),
                            graph_name.trim().to_string(),
                        ));
                        continue;
                    }
                    pieces.push(ChapterPiece::PassThrough(event.into_static()));
                } else {
                    pieces.push(ChapterPiece::PassThrough(event.into_static()));
                }
            }

            let mut output_events = Vec::new();
            for piece in pieces {
                match piece {
                    ChapterPiece::PassThrough(event) => output_events.push(event),
                    ChapterPiece::Diagram(block) => {
                        output_events.push(Event::Text(
                            format!(
                                "{}|{:?}|{}|{}",
                                block.light_file_name(),
                                block.light_path(),
                                block.graph_name,
                                block.index
                            )
                            .into(),
                        ));
                    }
                }
            }

            cmark(output_events.into_iter(), &mut buf)?;
            chapter.content = buf;
            Ok(chapter)
        }

        #[async_recursion(?Send)]
        async fn process_sub_items(&self, items: &mut Vec<BookItem>) -> Result<()> {
            let mut item_futures = Vec::with_capacity(items.len());
            for item in mem::take(items) {
                item_futures.push(async {
                    match item {
                        BookItem::Chapter(chapter) => {
                            self.process_chapter(chapter).await.map(BookItem::Chapter)
                        }
                        item => Ok(item),
                    }
                });
            }

            *items = future::join_all(item_futures)
                .await
                .into_iter()
                .collect::<Result<Vec<_>, _>>()?;

            Ok(())
        }
    }

    #[tokio::test]
    async fn ignores_unmarked_dot_blocks() {
        let expected = r#"# Chapter

````dot
digraph Test {
    a -> b
}
````"#;
        let chapter = process_chapter(new_chapter(expected)).await.unwrap();
        assert_eq!(chapter.content, expected);
    }

    #[tokio::test]
    async fn processes_custom_info_string() {
        let chapter = new_chapter(
            r#"# Chapter
```graphviz
digraph Test {
    a -> b
}
```
"#,
        );
        let expected = format!(
            r#"# Chapter

{NORMALIZED_CHAPTER_NAME}_0.modern-dot.svg|"/./book/{NORMALIZED_CHAPTER_NAME}_0.modern-dot.svg"||0"#
        );

        let config = Config {
            info_string: "graphviz".into(),
            ..Config::default()
        };
        let chapter = process_chapter_with_config(chapter, config).await.unwrap();
        assert_eq!(chapter.content, expected);
    }

    #[tokio::test]
    async fn ignores_mismatched_custom_info_string() {
        let expected = r#"# Chapter

````dot
digraph Test {
    a -> b
}
````"#;

        let config = Config {
            info_string: "graphviz".into(),
            ..Config::default()
        };
        let chapter = process_chapter_with_config(new_chapter(expected), config)
            .await
            .unwrap();
        assert_eq!(chapter.content, expected);
    }

    #[tokio::test]
    async fn unnamed_diagram() {
        let chapter = new_chapter(
            r#"# Chapter
```modern-dot
digraph Test {
    a -> b
}
```
"#,
        );
        let expected = format!(
            r#"# Chapter

{NORMALIZED_CHAPTER_NAME}_0.modern-dot.svg|"/./book/{NORMALIZED_CHAPTER_NAME}_0.modern-dot.svg"||0"#
        );
        let chapter = process_chapter(chapter).await.unwrap();
        assert_eq!(chapter.content, expected);
    }

    #[tokio::test]
    async fn named_diagram() {
        let chapter = new_chapter(
            r#"# Chapter
```modern-dot Graph Name
digraph Test {
    a -> b
}
```
"#,
        );
        let expected = format!(
            r#"# Chapter

{NORMALIZED_CHAPTER_NAME}_graph_name_0.modern-dot.svg|"/./book/{NORMALIZED_CHAPTER_NAME}_graph_name_0.modern-dot.svg"|Graph Name|0"#
        );
        let chapter = process_chapter(chapter).await.unwrap();
        assert_eq!(chapter.content, expected);
    }

    #[tokio::test]
    async fn preserve_escaping() {
        let chapter = new_chapter(
            r"# Chapter

*asteriks*
/*asteriks/*
( \int x dx = \frac{x^2}{2} + C)

```modern-dot Graph Name
digraph Test {
    a -> b
}
```
",
        );
        let expected = format!(
            r#"# Chapter

*asteriks*
/*asteriks/*
( \int x dx = \frac{{x^2}}{{2}} + C)

{NORMALIZED_CHAPTER_NAME}_graph_name_0.modern-dot.svg|"/./book/{NORMALIZED_CHAPTER_NAME}_graph_name_0.modern-dot.svg"|Graph Name|0"#
        );
        let chapter = process_chapter(chapter).await.unwrap();
        assert_eq!(chapter.content, expected);
    }

    #[tokio::test]
    async fn preserve_tables() {
        let chapter = new_chapter(
            r#"# Chapter

|Tables|Are|Cool|
|------|:-:|---:|
|col 1 is|left-aligned|$1600|
|col 2 is|centered|$12|
|col 3 is|right-aligned|$1|

```modern-dot Graph Name
digraph Test {
    a -> b
}
```
"#,
        );
        let expected = format!(
            r#"# Chapter

|Tables|Are|Cool|
|------|:-:|---:|
|col 1 is|left-aligned|$1600|
|col 2 is|centered|$12|
|col 3 is|right-aligned|$1|

{NORMALIZED_CHAPTER_NAME}_graph_name_0.modern-dot.svg|"/./book/{NORMALIZED_CHAPTER_NAME}_graph_name_0.modern-dot.svg"|Graph Name|0"#
        );
        let chapter = process_chapter(chapter).await.unwrap();
        assert_eq!(chapter.content, expected);
    }

    const SLEEP_DURATION: Duration = Duration::from_millis(100);

    #[tokio::test]
    async fn concurrent_chapter_processing() {
        struct SlowEngine;

        impl SlowEngine {
            #[async_recursion(?Send)]
            async fn process_sub_items(&self, items: &mut Vec<BookItem>) -> Result<()> {
                let mut item_futures = Vec::with_capacity(items.len());
                for item in mem::take(items) {
                    item_futures.push(async {
                        match item {
                            BookItem::Chapter(chapter) => {
                                self.process_chapter(chapter).await.map(BookItem::Chapter)
                            }
                            item => Ok(item),
                        }
                    });
                }

                *items = future::join_all(item_futures)
                    .await
                    .into_iter()
                    .collect::<Result<Vec<_>, _>>()?;

                Ok(())
            }

            async fn process_chapter(&self, mut chapter: Chapter) -> Result<Chapter> {
                self.process_sub_items(&mut chapter.sub_items).await?;
                tokio::time::sleep(SLEEP_DURATION).await;
                chapter.content = "# Chapter\n\n".into();
                Ok(chapter)
            }
        }

        const TOTAL_CHAPTERS: usize = 10;
        let mut chapters = Vec::with_capacity(TOTAL_CHAPTERS);
        for _ in 0..TOTAL_CHAPTERS {
            chapters.push(BookItem::Chapter(new_chapter(
                r#"# Chapter
```modern-dot Graph Name
digraph Test {
    a -> b
}
```
"#,
            )));
        }

        let start = Instant::now();
        SlowEngine.process_sub_items(&mut chapters).await.unwrap();
        let duration = start.elapsed();

        for item in chapters {
            if let BookItem::Chapter(chapter) = item {
                assert_eq!(chapter.content, "# Chapter\n\n");
            } else {
                panic!("expected chapter items only");
            }
        }

        assert!(
            duration < SLEEP_DURATION * 2,
            "{duration:?} should be less than 2 * {SLEEP_DURATION:?}"
        );
    }

    #[tokio::test]
    async fn chapter_sub_items() {
        let content = r#"# Chapter

```modern-dot Graph Name
digraph Test {
    a -> b
}
```
"#;
        let mut chapter = new_chapter(content);
        chapter
            .sub_items
            .push(BookItem::Chapter(new_chapter(content)));

        let expected = format!(
            r#"# Chapter

{NORMALIZED_CHAPTER_NAME}_graph_name_0.modern-dot.svg|"/./book/{NORMALIZED_CHAPTER_NAME}_graph_name_0.modern-dot.svg"|Graph Name|0"#
        );

        let mut chapter = process_chapter(chapter).await.unwrap();
        assert_eq!(chapter.content, expected);
        if let BookItem::Chapter(child_chapter) = chapter.sub_items.remove(0) {
            assert_eq!(child_chapter.content, expected);
        }
    }

    #[tokio::test]
    async fn skip_draft_chapters() {
        let draft_chapter = Chapter::new_draft(CHAPTER_NAME, vec![]);
        let mut book_items = vec![
            BookItem::Chapter(draft_chapter.clone()),
            BookItem::Chapter(new_chapter(
                r#"# Chapter
```modern-dot Graph Name
digraph Test {
    a -> b
}
```
"#,
            )),
        ];

        RecordingEngine {
            src_dir: PathBuf::from("/"),
            config: Config::default(),
        }
        .process_sub_items(&mut book_items)
        .await
        .unwrap();

        assert_eq!(
            book_items,
            vec![
                BookItem::Chapter(draft_chapter),
                BookItem::Chapter(new_chapter(format!(
                    r#"# Chapter

{NORMALIZED_CHAPTER_NAME}_graph_name_0.modern-dot.svg|"/./book/{NORMALIZED_CHAPTER_NAME}_graph_name_0.modern-dot.svg"|Graph Name|0"#
                )))
            ]
        );
    }

    async fn process_chapter(chapter: Chapter) -> Result<Chapter> {
        process_chapter_with_config(chapter, Config::default()).await
    }

    async fn process_chapter_with_config(chapter: Chapter, config: Config) -> Result<Chapter> {
        RecordingEngine {
            src_dir: PathBuf::from("/"),
            config,
        }
        .process_chapter(chapter)
        .await
    }

    fn new_chapter(content: impl Into<String>) -> Chapter {
        Chapter::new(
            CHAPTER_NAME,
            content.into(),
            PathBuf::from("./book/chapter.md"),
            vec![],
        )
    }
}
