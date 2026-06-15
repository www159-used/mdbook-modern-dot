use std::io;
use std::process;

use clap::{Parser, Subcommand};
use mdbook_preprocessor::Preprocessor;
use mdbook_preprocessor::errors::Error;

mod block;
mod config;
mod dot;
mod engine;
mod html;
mod preprocessor;
mod render;
mod theme;

use crate::preprocessor::ModernDot;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Supports { renderer: String },
}

fn main() {
    let cli = Cli::parse();
    let preprocessor = ModernDot;

    match cli.command {
        None => {
            if let Err(error) = run_preprocessor(&preprocessor) {
                eprintln!("{error}");
                process::exit(1);
            }
        }
        Some(Commands::Supports { renderer }) => match preprocessor.supports_renderer(&renderer) {
            Ok(true) => process::exit(0),
            Ok(false) => process::exit(1),
            Err(error) => {
                eprintln!("{error}");
                process::exit(1);
            }
        },
    }
}

fn run_preprocessor(pre: &dyn Preprocessor) -> Result<(), Error> {
    let (ctx, book) = mdbook_preprocessor::parse_input(io::stdin())?;

    if ctx.mdbook_version != mdbook_preprocessor::MDBOOK_VERSION {
        eprintln!(
            "Warning: The {} plugin was built against mdbook {}, but mdbook {} is running it",
            pre.name(),
            mdbook_preprocessor::MDBOOK_VERSION,
            ctx.mdbook_version
        );
    }

    let processed_book = pre.run(&ctx, book)?;
    serde_json::to_writer(io::stdout(), &processed_book)?;

    Ok(())
}
