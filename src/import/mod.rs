use std::str::FromStr;

use super::*;

/// Parse the input to durf asts.
pub async fn import(config: &mut Config) -> Result<Book> {
    let mut book = config.export.book()?;

    for chapter in config.import.chapters.iter() {
        let mut uri: String = chapter.uri.clone();
        let title = match chapter.title.as_str() {
            "" => None,
            _ => Some(chapter.title.to_string()),
        };
        tracing::debug!("Parsing uri: {uri}.");

        // If website, fetch content.
        if uri.starts_with("http://") || uri.starts_with("https://") {
            let client = reqwest::ClientBuilder::new()
                .user_agent(&config.parse.html.user_agent)
                .build()?;
            let res = client.get(&uri).send().await?;
            let body = res.text().await?;

            let flags = config.parse.html.parse_flags()?;
            let mut ast = match durf::Ast::from_html(&body, flags) {
                Ok(ast) => ast,
                Err(e) => {
                    tracing::error!("Failed to parse website: {e}");
                    continue;
                }
            };
            ast.minimize();
            book.chapters.push(Chapter { ast, title });
            continue;
        }

        // If still not parsed, try to treat as file.
        {
            // Remove file uri.
            if uri.starts_with("file://") {
                uri = uri.replace("file://", "");
            }

            let normalized_path = match PathBuf::from_str(uri.as_str()) {
                Ok(fname) => fname,
                Err(e) => bail!("Unable to parse filename: {e}"),
            };

            // // If file doesn't exist, try to get it relative to the config file.
            // if !normalized_path.exists()

            let body = match std::fs::read_to_string(&normalized_path) {
                Ok(body) => body,
                Err(e) => {
                    tracing::error!(
                        "Unable to read file: '{}': {e}",
                        normalized_path.to_string_lossy()
                    );
                    continue;
                }
            };

            // We parsed the actual document.
            let flags = config.parse.html.parse_flags()?;
            let mut ast = match durf::Ast::from_html(&body, flags) {
                Ok(ast) => ast,
                Err(e) => {
                    tracing::error!("Failed to parse website: {e}");
                    continue;
                }
            };
            ast.minimize();
            book.chapters.push(Chapter { ast, title });
            continue;
        }
    }

    // If we still weren't able to parse the file, fail.
    if book.chapters.is_empty() {
        bail!("No chapters were parsed.");
    }

    Ok(book)
}
