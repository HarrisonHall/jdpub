use super::*;

use epub_builder::EpubBuilder;
use epub_builder::EpubContent;
use epub_builder::ReferenceType;
// use epub_builder::Result;
// use epub_builder::TocElement;
use epub_builder::ZipLibrary;

pub fn export(chapters: &mut Vec<durf::Ast>, config: &Config) -> Result<()> {
    // Create the builder.
    let mut builder = match EpubBuilder::new(match ZipLibrary::new() {
        Ok(z) => z,
        Err(e) => bail!("Failed to create epub ZipLibrary: {e}"),
    }) {
        Ok(builder) => builder,
        Err(e) => bail!("Failed to create epub builder: {e}"),
    };
    builder.epub_version(epub_builder::EpubVersion::V30);

    if let Some(cover) = &config.cover_file {
        let cover_image = match std::fs::read(cover) {
            Ok(d) => d,
            Err(e) => bail!("Unable to read cover image: {e}"),
        };
        let mimetype = get_mimetype(cover.to_string_lossy());
        match builder.add_cover_image("cover.png", cover_image.as_slice(), mimetype) {
            Ok(_) => {}
            Err(e) => bail!("Failed to add cover image: {e}"),
        };
    }
    builder.metadata(
        "author",
        match config.author.as_str() {
            "" => "jdpub",
            _ => &config.author,
        },
    )?;
    builder.metadata(
        "title",
        match config.title.as_str() {
            "" => format!("jdpub-{}", uuid::Uuid::new_v4()),
            _ => config.title.clone(),
        },
    )?;

    builder.stylesheet(".footnotes { display: hidden; }".as_bytes())?;
    // .stylesheet(css_file.as_bytes())?
    // .add_content(
    //     EpubContent::new("cover.xhtml", dummy_content.as_bytes())
    //         .title("Cover")
    //         .reftype(ReferenceType::Cover),
    // )?
    // Add a title page
    // .add_content(
    //     EpubContent::new("title.xhtml", dummy_content.as_bytes())
    //         .title("Title <T>")
    //         .reftype(ReferenceType::TitlePage),
    // )?

    // Add cover.
    builder.add_content(
        EpubContent::new(
            "cover.xhtml",
            html::html::HtmlPage::new()
                .with_raw(
                    html::html::HtmlElement::new(html::html::HtmlTag::Heading1)
                        .with_child(config.title.clone().into()),
                )
                .to_html_string()
                .as_bytes(),
        )
        .title("Cover")
        .reftype(ReferenceType::Cover),
    )?;

    // Add the table of contents.
    builder.inline_toc();

    // Add the xhtml, mark it as beginning of the "real content"
    for (i, chapter) in chapters.iter_mut().enumerate() {
        // Convert chapter to html.
        let chapter_name = format!("Chapter {}", i + 1);
        let mut doc = html::HtmlDoc::new(chapter.clone());

        // Build to html string.
        let as_html = doc
            .build()?
            .to_html_string()
            // TODO: This can be removed.
            .replace(JPDB_FILE_TEMPLATE, "test.xhtml")
            // We need to add the epub namespace to use epub attributes.
            .replace(
                "xml:lang=\"en\"",
                "xml:lang=\"en\" xmlns:epub=\"http://www.idpf.org/2007/ops\"",
            );

        // Add content to epub.
        builder.add_content(
            EpubContent::new(
                format!("{}.xhtml", chapter_name.to_lowercase().replace(" ", "_")),
                as_html.as_bytes(),
            )
            .title(chapter_name)
            .reftype(ReferenceType::Text),
        )?;
    }

    let mut out = std::fs::File::create(config.output())?;
    match builder.generate(&mut out) {
        Ok(()) => {
            tracing::info!(
                "Successfully generated {}.",
                config.output().to_string_lossy()
            );
        }
        Err(e) => {
            bail!(
                "Failed to write {}: {}",
                config.output().to_string_lossy(),
                e
            );
        }
    };

    Ok(())
}
