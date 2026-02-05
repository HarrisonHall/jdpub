use super::*;

use ::epub::doc::EpubDoc;

pub fn import(
    path: &Path,
    #[allow(unused)] config: &Config,
    book: &mut Book,
    #[allow(unused)] chapter_config: &ChapterConfig,
) -> Result<()> {
    let mut doc = match EpubDoc::new(path) {
        Ok(epub) => epub,
        Err(e) => bail!("Failed to load epub: {e:?}"),
    };

    // Add metadata.
    if let Some(title) = doc.get_title() {
        if book.title.is_empty() {
            book.title = title.clone();
        }
    }
    if let Some(author) = doc.mdata("author") {
        if book.author.is_empty() {
            book.author = author.value.clone();
        }
    }

    // Add chapters.
    let flags = durf::ParseFlags::default();
    while let Some((chapter_data, mime_type)) = doc.get_current() {
        let path = match doc.get_current_path() {
            Some(p) => p.clone(),
            None => PathBuf::from_str("n/a")?,
        };
        if !mime_type.contains("application/xhtml+xml") {
            continue;
        }
        tracing::trace!("Parsing epub chapter from {path:?}.");

        if let Ok(content) = std::str::from_utf8(chapter_data.as_slice()) {
            let mut ast = match durf::Ast::from_html(&content, flags.clone()) {
                Ok(ast) => ast,
                Err(e) => {
                    bail!("Failed to parse epub text: {e}");
                }
            };
            ast.minimize();
            let chapter = Chapter { title: None, ast };
            book.chapters.push(chapter);
        } else {
            tracing::warn!("Unable to parse ")
        }

        if !doc.go_next() {
            break;
        }
    }

    Ok(())
}
