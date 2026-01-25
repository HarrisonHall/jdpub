use std::io::Write;

use super::*;

pub use build_html::HtmlContainer;
pub use build_html::{self as html, Html};

pub fn export(chapters: &mut Vec<durf::Ast>, config: &Config) -> Result<()> {
    // Create new HTML page.
    let mut page = html::HtmlPage::new().with_title(match config.title.as_str() {
        "" => "JDPUB",
        _ => &config.title,
    });

    // Add the html for each chapter.
    let mut docs = Vec::new();
    for (i, chapter) in chapters.iter_mut().enumerate() {
        // Convert chapter to html.
        // let chapter_name = format!("Chapter {}", i + 1);
        let mut doc = HtmlDoc::new(chapter.clone());
        doc.export_as = ExportOption::Html;

        // Add ast as element.
        let root = doc.ast.root.clone();
        page.add_raw(match doc.to_html_element(&root) {
            Some(e) => e.to_string(),
            None => String::new(),
        });

        docs.push(doc);
    }

    // Add the tooltips (footnotes) for each chapter.
    let mut footnotes_elem = html::HtmlElement::new(html::HtmlTag::ParagraphText)
        .with_attribute("class", "footnotes NoShow");
    for (i, doc) in docs.iter_mut().enumerate() {
        for footnote in doc.footnotes.iter() {
            let footnote = footnote.clone();
            footnotes_elem.add_child(footnote.into());
        }
    }
    page.add_raw(footnotes_elem.to_string());

    // Build to html string.
    let as_html = page
        .with_style(
            "
            [role=\"tooltip\"] {
              visibility: hidden;
              position: absolute;
              top: 2rem;
              left: 2rem;
              background: black;
              color: white;
              padding: 0.5rem;
              border-radius: 0.25rem;
              /* Give some time before hiding so mouse can exit the input
              and enter the tooltip */
              transition: visibility 0.5s;
            }
            [aria-describedby]:hover,
            [aria-describedby]:focus {
              position: relative;
            }
            [aria-describedby]:hover + [role=\"tooltip\"],
            [aria-describedby]:focus + [role=\"tooltip\"],
            [role=\"tooltip\"]:hover,
            [role=\"tooltip\"]:focus {
              visibility: visible;
            }
            ",
        )
        .to_html_string()
        // TODO: This can be removed.
        .replace(JPDB_FILE_TEMPLATE, "test.xhtml")
        // We need to add the epub namespace to use epub attributes.
        .replace(
            "xml:lang=\"en\"",
            "xml:lang=\"en\" xmlns:epub=\"http://www.idpf.org/2007/ops\"",
        );

    let mut out = std::fs::File::create(config.output())?;
    match out.write_all(as_html.as_bytes()) {
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

pub enum ExportOption {
    Html,
    Epub,
}

pub struct HtmlDoc {
    pub ast: durf::Ast,
    footnotes: Vec<html::HtmlElement>,
    export_as: ExportOption,
}

impl HtmlDoc {
    pub fn new(ast: durf_parser::Ast) -> Self {
        Self {
            ast,
            footnotes: Vec::new(),
            export_as: ExportOption::Epub,
            // page: html::HtmlPage::new().with_title("JPDB"),
            // num_footnotes: 0,
        }
    }

    pub fn build(&mut self) -> Result<html::HtmlPage> {
        // Create new page.
        let mut page = html::HtmlPage::new().with_title("JPDB");

        // Add ast as element.
        let root = self.ast.root.clone();
        page.add_raw(match self.to_html_element(&root) {
            Some(e) => e.to_string(),
            None => String::new(),
        });

        // Add footnotes.
        let mut footnotes_elem = html::HtmlElement::new(html::HtmlTag::ParagraphText)
            .with_attribute("class", "footnotes NoShow");
        let footnotes: Vec<html::HtmlElement> = self.footnotes.drain(..).collect();
        for footnote in footnotes {
            footnotes_elem = footnotes_elem.with_child(footnote.into());
        }
        page.add_raw(footnotes_elem.to_string());

        Ok(page)
    }

    fn to_html_element(&mut self, node: &durf::Node) -> Option<html::HtmlElement> {
        match &**node {
            durf_parser::RawNode::Empty => None,
            durf_parser::RawNode::Section(section) => {
                let mut elem = html::HtmlElement::new(match section.ordering() {
                    durf_parser::SectionOrdering::Set => html::HtmlTag::ParagraphText,
                    _ => html::HtmlTag::Div,
                });
                for node in section.nodes.iter() {
                    match self.to_html_element(node) {
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
                        text_elem = text_elem.with_child(
                            html::HtmlElement::new(html::HtmlTag::Link)
                                .with_child(fragment.text.as_str().into())
                                .with_attribute("class", "noteref")
                                .with_attribute(
                                    "href",
                                    // format!("{JPDB_FILE_TEMPLATE}#tooltip-{id}"),
                                    format!("#tooltip-{id}"),
                                )
                                .with_attribute("epub:type", "noteref")
                                .with_attribute("role", "doc-noteref")
                                .with_attribute("aria-describedby", format!("#tooltip-{id}"))
                                .into(),
                        );
                        // .with_child(html::HtmlChild::new(html::HtmlTag::))
                        self.footnotes.push(
                            html::HtmlElement::new(html::HtmlTag::Aside)
                                .with_child(tooltip.as_str().into())
                                .with_attribute("class", "footnote")
                                .with_attribute("id", format!("tooltip-{id}"))
                                .with_attribute("epub:type", "footnote")
                                // .with_attribute("epub:-cr-hint", "non-linear")
                                .with_attribute("epub:linear", "no")
                                .with_attribute(
                                    "role",
                                    match self.export_as {
                                        ExportOption::Html => "tooltip",
                                        ExportOption::Epub => "doc-footnote",
                                    },
                                )
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
