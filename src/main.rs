//! jdpub

mod book;
mod cli;
mod config;
mod export;
mod import;
mod language;
mod logging;
mod prelude;
mod util;

pub use prelude::internal::*;
pub use prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Parse the CLI.
    let cli = Cli::new()?;
    crate::logging::init(&cli)?;

    // Parse configuration.
    let mut config = cli.config()?;

    // Build the database.
    let db = DictDb::new()?;

    // Parse input.
    let mut book = import::import(&mut config).await?;

    // Add annotations.
    for chapter in book.chapters.iter_mut() {
        db.transform(&mut chapter.ast.root, &config)?;
    }

    // Export accordingly.
    export::export(&mut book, &config)?;

    Ok(())
}
