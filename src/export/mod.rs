use std::str::FromStr;

use super::*;

const JPDB_FILE_TEMPLATE: &'static str = "{{JPDB_FILE_TEMPLATE}}";

pub fn export(ast: durf::Ast, config: &Config) -> Result<()> {
    // Create an html document.
    let doc = HtmlDoc::new(ast);
    let as_html = doc
        .to_html()
        .to_html_string()
        .replace(JPDB_FILE_TEMPLATE, "test.xhtml")
        .replace(
            "xml:lang=\"en\"",
            "xml:lang=\"en\" xmlns:epub=\"http://www.idpf.org/2007/ops\"",
        );

    tracing::trace!("Generated HTML:\n{}\n", as_html);

    std::fs::write("test.xhtml", &as_html)?;

    use epub_builder::EpubBuilder;
    use epub_builder::EpubContent;
    use epub_builder::ReferenceType;
    use epub_builder::Result;
    use epub_builder::TocElement;
    use epub_builder::ZipLibrary;
    use std::io;
    use std::io::Read;
    use std::io::Write;

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

    // Add the xhtml, mark it as beginning of the "real content"
    builder.add_content(
        EpubContent::new("content.xhtml", as_html.as_bytes())
            .title("content")
            .reftype(ReferenceType::Text),
    )?;

    builder.inline_toc();

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

struct HtmlDoc {
    ast: durf_parser::Ast,
    page: html::HtmlPage,
    num_footnotes: u32,
    // footnotes: Vec<FootNote>,
}

impl HtmlDoc {
    fn new(ast: durf_parser::Ast) -> Self {
        Self {
            ast,
            page: html::HtmlPage::new().with_title("JPDB"),
            num_footnotes: 0,
        }
    }
}

trait ToHtml {
    fn to_html(&self) -> html::HtmlPage {
        use html::HtmlContainer;

        let page =
            html::HtmlPage::new()
                .with_title("JPDB")
                .with_raw(match self.to_html_element() {
                    Some(e) => e.to_string(),
                    None => String::new(),
                });
        page
    }

    fn to_html_element(&self) -> Option<html::HtmlElement>;
}

impl ToHtml for HtmlDoc {
    fn to_html(&self) -> build_html::HtmlPage {
        use html::HtmlContainer;
        let page = html::HtmlPage::with_version(html::HtmlVersion::XHTML1_1)
            .with_title("JPDB")
            .with_raw(match self.to_html_element() {
                Some(e) => e.to_string(),
                None => String::new(),
            });
        page
    }

    fn to_html_element(&self) -> Option<build_html::HtmlElement> {
        self.ast.root.to_html_element()
    }
}

impl ToHtml for durf_parser::Node {
    fn to_html_element(&self) -> Option<build_html::HtmlElement> {
        match &**self {
            durf_parser::RawNode::Empty => None,
            durf_parser::RawNode::Section(section) => {
                let mut elem = html::HtmlElement::new(match section.ordering() {
                    durf_parser::SectionOrdering::Set => html::HtmlTag::ParagraphText,
                    _ => html::HtmlTag::Div,
                });
                for node in section.nodes.iter() {
                    match node.to_html_element() {
                        Some(n) => {
                            elem = elem.with_child(n.into());
                        }
                        None => {}
                    }
                }
                Some(elem)
            }
            durf_parser::RawNode::Text(text) => {
                let mut elem = html::HtmlElement::new(html::HtmlTag::Div);
                for fragment in text.fragments.iter() {
                    // If plain text, just add.
                    if fragment.attributes.is_plain() {
                        elem.add_child(fragment.text.as_str().into());
                        continue;
                    }

                    // Heading is special.
                    if let Some(heading) = &fragment.attributes.heading {
                        elem = elem.with_child(
                            html::HtmlElement::new(match heading {
                                0..2 => html::HtmlTag::Heading1,
                                2 => html::HtmlTag::Heading2,
                                3 => html::HtmlTag::Heading3,
                                4 => html::HtmlTag::Heading4,
                                5 => html::HtmlTag::Heading5,
                                _ => html::HtmlTag::Heading6,
                            })
                            .with_child(fragment.text.as_str().into())
                            .into(),
                        );
                        continue;
                    }

                    let mut text_elem = html::HtmlElement::new(html::HtmlTag::Span);
                    if let Some(tooltip) = &fragment.attributes.tooltip {
                        let id = uuid::Uuid::new_v4();
                        text_elem = text_elem
                            .with_child(
                                html::HtmlElement::new(html::HtmlTag::Link)
                                    .with_child(fragment.text.as_str().into())
                                    .with_attribute("class", "noteref")
                                    .with_attribute(
                                        "href",
                                        // format!("{JPDB_FILE_TEMPLATE}#tooltip-{id}"),
                                        format!("#tooltip-{id}"),
                                    )
                                    .with_attribute("epub:type", "noteref")
                                    .into(),
                            )
                            // .with_child(html::HtmlChild::new(html::HtmlTag::))
                            .with_child(
                                html::HtmlElement::new(html::HtmlTag::Aside)
                                    .with_child(tooltip.as_str().into())
                                    .with_attribute("class", "footnote")
                                    .with_attribute("id", format!("tooltip-{id}"))
                                    .with_attribute("epub:type", "footnote")
                                    // .with_attribute("epub:-cr-hint", "non-linear")
                                    .with_attribute("epub:linear", "no")
                                    .into(),
                            );
                        elem = elem.with_child(text_elem.into());
                        continue;
                    }
                    // if fragment.attributes.preformatted {
                    //     text_elem = text_elem.
                    // }
                    elem =
                        elem.with_child(text_elem.with_child(fragment.text.as_str().into()).into());
                }

                Some(elem)
            }
        }
    }
}
