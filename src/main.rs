/// jdpub
///
/// ## Links
/// - https://help.apple.com/itc/booksassetguide/en.lproj/itccf8ecf5c8.html
/// - https://software.grok.lsu.edu/Article.aspx?articleid=6900
use std::collections::HashMap;

use anyhow::{Result, anyhow, bail};
use build_html::{self as html, Html};

const JPDB_FILE_TEMPLATE: &'static str = "{{JPDB_FILE_TEMPLATE}}";

#[tokio::main]
async fn main() -> Result<()> {
    println!("jdpub");

    let db = DictDb::new()?;

    // let url = "https://ncode.syosetu.com/n9669bk/3/";

    // let client = reqwest::ClientBuilder::new().user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/144.0.0.0 Safari/537.36").build()?;
    // let res = client.get(url).send().await?;
    // let body = res.text().await?;

    let body = std::fs::read_to_string("./mushoku.html")?;

    let mut flags = durf_parser::ParseFlags::default();
    flags.parsing = false;
    flags
        .allow
        .push(durf_parser::ParseRule::from_element("article"));
    flags
        .allow
        .push(durf_parser::ParseRule::from_element("main"));
    flags
        .skip
        .push(durf_parser::ParseRule::from_class("c-pager"));
    flags
        .skip
        .push(durf_parser::ParseRule::from_class("c-modal"));
    flags
        .skip
        .push(durf_parser::ParseRule::from_class("js-siori"));
    flags
        .skip
        .push(durf_parser::ParseRule::from_class("c-announce-box"));
    flags
        .skip
        .push(durf_parser::ParseRule::from_class("c-toast"));
    flags.skip.push(durf_parser::ParseRule::from_class("c-ad"));
    flags
        .skip
        .push(durf_parser::ParseRule::from_class("p-reaction"));
    flags.skip.push(durf_parser::ParseRule::from_class(
        "p-novelpoint-form__body",
    ));
    flags
        .skip
        .push(durf_parser::ParseRule::from_class("l-foot-contents"));
    flags
        .skip
        .push(durf_parser::ParseRule::from_class("remodal-wrapper"));

    let mut ast = durf_parser::Ast::from_html(&body, flags)?;
    ast.minimize();

    println!("AST\n{}\n", ast);

    transform(&mut ast.root, &db)?;

    let doc = HtmlDoc::new(ast);
    let as_html = doc
        .to_html()
        .to_html_string()
        .replace(JPDB_FILE_TEMPLATE, "test.xhtml")
        .replace(
            "xml:lang=\"en\"",
            "xml:lang=\"en\" xmlns:epub=\"http://www.idpf.org/2007/ops\"",
        );

    println!("HTML\n{}\n", as_html);

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

    let mut builder = EpubBuilder::new(ZipLibrary::new()?)?;
    let cover: Vec<u8> = std::fs::read("/home/hachha/downloads/mushoku.png")?;
    // let dummy_image = format!("DUMMY");
    // Set some metadata
    builder
        .epub_version(epub_builder::EpubVersion::V30)
        .metadata("author", "Joan Doe")?
        .metadata("title", "Dummy Book <T>")?
        // Set the stylesheet (create a "stylesheet.css" file in EPUB that is used by some generated files)
        // .stylesheet(dummy_css.as_bytes())?
        // Add a image cover file
        .add_cover_image(
            "mushoku.png",
            // std::path::Path::new("/hom/harrison/downloads/mushoku.png"),
            // std::fs::read("/home/harrison/downloads/mushoku.png")?,
            cover.as_slice(),
            "image/png",
        )?
        // .add_cover_image("cover.png", dummy_image.as_bytes(), "image/png")?
        // Add a resource that is not part of the linear document structure
        // .add_resource("some_image.png", dummy_image.as_bytes(), "image/png")?
        // Add a cover page
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
        // Add a chapter, mark it as beginning of the "real content"
        .add_content(
            EpubContent::new("test.xhtml", as_html.as_bytes())
                .title("Chapter 1 <T>")
                .reftype(ReferenceType::Text),
        )?
        // Add a second chapter; this one has more toc information about its internal structure
        // .add_content(
        //     EpubContent::new("chapter_2.xhtml", dummy_content.as_bytes())
        //         .title("Chapter 2 <T>")
        //         .child(TocElement::new("chapter_2.xhtml#1", "Chapter 2, section 1")),
        // )?
        // Add a section. Since its level is set to 2, it will be attached to the previous chapter.
        // .add_content(
        //     EpubContent::new("section.xhtml", dummy_content.as_bytes())
        //         .title("Chapter 2 <T>, section 2")
        //         .level(2),
        // )?
        // Add a chapter without a title, which will thus not appear in the TOC.
        // .add_content(EpubContent::new("notes.xhtml", dummy_content.as_bytes()))?
        // Generate a toc inside of the document, that will be part of the linear structure.
        .inline_toc();
    let mut out = std::fs::File::create("mushoku.epub")?;
    // Finally, write the EPUB file to stdout
    // builder.generate(&mut io::stdout())?; // generate into stout
    builder.generate(&mut out)?; // generate into stout

    Ok(())
}

fn transform(node: &mut durf_parser::RawNode, db: &DictDb) -> Result<()> {
    use charabia::Tokenize;

    match node {
        durf_parser::RawNode::Empty => {}
        durf_parser::RawNode::Section(section) => {
            for node in section.nodes.iter_mut() {
                transform(node, db)?;
            }
        }
        durf_parser::RawNode::Text(text) => {
            let mut new_text = durf_parser::Text::new();

            let total_text = text
                .fragments
                .iter()
                .fold(String::new(), |acc, el| acc + &el.text)
                .replace("\n", " ")
                .replace("　", "");
            let total_text = total_text.as_str();

            let tokens = total_text.tokenize();
            'token_loop: for token in tokens {
                if token.is_word() {
                    let word = token.lemma();
                    if let Some(lookup) = db.lookup(word) {
                        if lookup.is_kana && word.chars().count() == 1 {
                            new_text
                                .fragments
                                .push(durf_parser::TextFragment::new(token.lemma(), None));
                            continue 'token_loop;
                        }
                        if lookup.jlpt.unwrap_or(0) > 2 {
                            new_text
                                .fragments
                                .push(durf_parser::TextFragment::new(token.lemma(), None));
                            continue 'token_loop;
                        }

                        let mut attributes = durf_parser::TextAttributes::default();
                        attributes.tooltip = Some(format!(
                            "{}[{}::{}::N{}]",
                            word,
                            lookup.kana,
                            lookup.meaning,
                            lookup.jlpt.unwrap_or(0),
                        ));
                        new_text.fragments.push(durf_parser::TextFragment::new(
                            token.lemma(),
                            Some(attributes),
                        ));
                    } else {
                        new_text
                            .fragments
                            .push(durf_parser::TextFragment::new(token.lemma(), None));
                    }
                } else {
                    new_text
                        .fragments
                        .push(durf_parser::TextFragment::new(token.lemma(), None));
                }
            }

            text.fragments = new_text.fragments;
        }
    }

    Ok(())
}

struct DictDb {
    jlpt_entries: HashMap<String, JlptEntry>,
    dict: jmdict_fast::Dict<'static>,
}

impl DictDb {
    fn new() -> Result<Self> {
        let dict = jmdict_fast::Dict::load_default()?;

        let mut db = Self {
            jlpt_entries: HashMap::new(),
            dict,
        };

        db.read_jlpt("metadata/jp/jlpt/n1.csv", 1)?;
        db.read_jlpt("metadata/jp/jlpt/n2.csv", 2)?;
        db.read_jlpt("metadata/jp/jlpt/n3.csv", 3)?;
        db.read_jlpt("metadata/jp/jlpt/n4.csv", 4)?;
        db.read_jlpt("metadata/jp/jlpt/n5.csv", 5)?;

        println!("Entries: {}", db.jlpt_entries.len());

        Ok(db)
    }

    fn read_jlpt(&mut self, file: &str, level: u8) -> Result<()> {
        let file = std::fs::File::open(file)?;
        let mut reader = csv::ReaderBuilder::new().from_reader(std::io::BufReader::new(file));
        for result in reader.deserialize() {
            // Notice that we need to provide a type hint for automatic
            // deserialization.
            let mut record: JlptEntry = match result {
                Ok(record) => record,
                Err(e) => continue,
            };
            record.level = level;
            self.jlpt_entries.insert(record.expression.clone(), record);
        }

        Ok(())
    }

    fn lookup(&self, word: &str) -> Option<DictLookup> {
        let jlpt_level = match self.jlpt_entries.get(word) {
            Some(j) => Some(j.level),
            None => None,
        };

        let results = self.dict.lookup_exact(word);
        if results.len() == 0 {
            return None;
        }

        if results[0].kana.len() == 0
            || results[0].sense.len() == 0
            || results[0].sense[0].gloss.len() == 0
        {
            return None;
        }

        use wana_kana::IsJapaneseStr;
        Some(DictLookup {
            is_kana: word.is_kana(),
            kana: results[0].kana[0].text.clone(),
            meaning: results[0].sense[0].gloss[0].text.clone(),
            jlpt: jlpt_level,
        })
    }
}

#[derive(Debug, serde::Deserialize)]
#[allow(unused)]
struct JlptEntry {
    expression: String,
    reading: String,
    meaning: String,
    tags: String,
    guid: String,
    #[serde(default)]
    level: u8,
}

struct DictLookup {
    is_kana: bool,
    kana: String,
    meaning: String,
    jlpt: Option<u8>,
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
