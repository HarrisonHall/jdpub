//! jdpub

mod cli;
mod config;
mod export;
mod language;
mod logging;
mod parsing;
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
    let mut chapters = parsing::parse(&mut config).await?;

    // Add annotations.
    for chapter in chapters.iter_mut() {
        db.transform(&mut chapter.root, &config)?;
    }

    // Export accordingly.
    export::export(&mut chapters, &config)?;

    Ok(())
}
