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
    let config = cli.config()?;

    // Build the database.
    let db = DictDb::new()?;

    // Parse input.
    let mut ast = parsing::parse(&config).await?;

    // Add annotations.
    db.transform(&mut ast.root, &config)?;

    // Export accordingly.
    export::export(ast, &config)?;

    Ok(())
}
