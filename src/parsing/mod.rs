use std::str::FromStr;

use super::*;

/// Parse the input to durf asts.
pub async fn parse(config: &mut Config) -> Result<Vec<durf::Ast>> {
    let mut uri: String = config.input().into();

    tracing::info!("URI: {uri}");

    // let mut as_html: Option<String> = None;
    let mut raw_chapters: Vec<String> = Vec::new();

    // Fetch html.
    if uri.starts_with("http://") || uri.starts_with("https://") {
        let client = reqwest::ClientBuilder::new()
            .user_agent(&config.user_agent)
            .build()?;
        let res = client.get(&uri).send().await?;
        let body = res.text().await?;
        raw_chapters.push(body);
    }

    // Remove file uri.
    if uri.starts_with("file://") {
        uri = uri.replace("file://", "");
    }

    // If still not parsed, try to treat as file.
    if raw_chapters.is_empty() {
        let normalized_path = match shellexpand::full(&uri) {
            Ok(fname) => fname,
            Err(e) => bail!("Unable to normalize filename: {e}"),
        }
        .into_owned();

        let body = match std::fs::read_to_string(&normalized_path) {
            Ok(body) => body,
            Err(e) => bail!("Unable to read file: {e}"),
        };

        if normalized_path.ends_with(".toml") {
            // Parse a book document into multiple chapters.
            let book: BookDoc = match toml::from_str(&body) {
                Ok(book) => book,
                Err(e) => bail!("Failed to parse book document: {e}"),
            };
            if book.cover.is_some() {
                config.cover_file = book.cover.clone();
            }
            if let Some(title) = &book.title {
                config.title = title.clone();
            }

            // Read each chapter from the book file.
            let book_path = PathBuf::from_str(normalized_path.as_str())?;
            let book_dir = match book_path.parent() {
                Some(p) => p.to_path_buf(),
                None => std::env::current_dir()?,
            };
            for chapter in book.chapters.iter() {
                let mut chapter_file = book_dir.clone();
                chapter_file.push(chapter);
                let body = match std::fs::read_to_string(&chapter_file) {
                    Ok(body) => body,
                    Err(e) => bail!("Unable to read file: {e}"),
                };
                raw_chapters.push(body);
            }
        } else {
            // We parsed the actual document.
            raw_chapters.push(body);
        }
    }

    // If we still weren't able to parse the file, fail.
    if raw_chapters.is_empty() {
        bail!("Unable to get contents of uri '{uri}'.");
    }

    // Parse to durf.
    let mut chapters = Vec::new();
    for chapter in raw_chapters.iter() {
        let flags = config.parse_flags()?;
        let mut ast = durf_parser::Ast::from_html(chapter, flags)?;
        ast.minimize();

        tracing::trace!("AST:\n{}", ast);
        chapters.push(ast);
    }

    Ok(chapters)
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct BookDoc {
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    cover: Option<PathBuf>,
    chapters: Vec<PathBuf>,
}

// TODO
// Instead of ASTs, pass around chapters
#[derive(Serialize, Deserialize, Clone, Debug)]
struct Chapter {
    title: String,
    path: PathBuf,
}
