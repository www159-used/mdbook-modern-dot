use mdbook_preprocessor::book::Book;
use mdbook_preprocessor::errors::{Error, Result};
use mdbook_preprocessor::{Preprocessor, PreprocessorContext};

use crate::config::{Config, PREPROCESSOR_NAME};
use crate::dot;
use crate::engine::Engine;

pub struct ModernDot;

impl Preprocessor for ModernDot {
    fn name(&self) -> &str {
        PREPROCESSOR_NAME
    }

    fn run(&self, ctx: &PreprocessorContext, mut book: Book) -> Result<Book> {
        dot::ensure_available()?;

        let config = Config::load(ctx)?;
        let src_dir = ctx.root.join(&ctx.config.book.src);

        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .build()
            .map_err(|error| {
                Error::from(std::io::Error::other(format!(
                    "create tokio runtime: {error}"
                )))
            })?;

        runtime.block_on(async {
            Engine::new(src_dir, config).process_book(&mut book).await
        })?;

        Ok(book)
    }
}
