use super::*;

use epub_parser::Epub;

pub fn import(
    path: &Path,
    #[allow(unused)] config: &Config,
    book: &mut Book,
    #[allow(unused)] chapter_config: &ChapterConfig,
) -> Result<()> {
    let epub = match Epub::parse(path) {
        Ok(epub) => epub,
        Err(e) => bail!("{e:?}"),
    };

    // Add metadata.
    if let Some(title) = &epub.metadata.title {
        if book.title.is_empty() {
            book.title = title.clone();
        }
    }
    if let Some(author) = &epub.metadata.author {
        if book.author.is_empty() {
            book.author = author.clone();
        }
    }

    // Add chapters.
    // TODO: Map pages into chapters.
    // for chapter in epub.toc.iter() {

    // }

    let mut total_content = String::new();
    for page in epub.pages.iter() {
        total_content += page.content.as_str();
        total_content += "\n";
    }
    let flags = durf::ParseFlags::default();
    let mut ast = match durf::Ast::from_text(&total_content, flags) {
        Ok(ast) => ast,
        Err(e) => {
            bail!("Failed to parse epub text: {e}");
        }
    };
    ast.minimize();

    let chapter = Chapter {
        title: Some(book.title.clone()),
        ast,
    };
    book.chapters.push(chapter);

    Ok(())
}
