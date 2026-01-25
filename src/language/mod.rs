use super::*;

pub struct DictDb {
    jlpt_entries: HashMap<String, JlptEntry>,
    dict: jmdict_fast::Dict<'static>,
}

impl DictDb {
    pub fn new() -> Result<Self> {
        let dict = jmdict_fast::Dict::load_default()?;

        let mut db = Self {
            jlpt_entries: HashMap::new(),
            dict,
        };

        db.read_jlpt_csv("jp/jlpt/n1.csv", 1)?;
        db.read_jlpt_csv("jp/jlpt/n2.csv", 2)?;
        db.read_jlpt_csv("jp/jlpt/n3.csv", 3)?;
        db.read_jlpt_csv("jp/jlpt/n4.csv", 4)?;
        db.read_jlpt_csv("jp/jlpt/n5.csv", 5)?;

        tracing::debug!("Parsed {} JLPT entries.", db.jlpt_entries.len());

        Ok(db)
    }

    fn read_jlpt_csv(&mut self, file: &str, level: u8) -> Result<()> {
        let content = read_embedded_text::<Metadata>(file)?;
        // let file = std::fs::File::open(file)?;
        // let mut reader = csv::ReaderBuilder::new().from_reader(std::io::BufReader::new(file));
        let mut reader =
            csv::ReaderBuilder::new().from_reader(std::io::BufReader::new(content.as_bytes()));
        // let mut reader = csv::ReaderBuilder::new().from_reader(&content);
        for result in reader.deserialize() {
            // Notice that we need to provide a type hint for automatic
            // deserialization.
            let mut record: JlptEntry = match result {
                Ok(record) => record,
                Err(e) => {
                    tracing::warn!("Unable to parse JlptEntry: {e}");
                    continue;
                }
            };
            record.level = level;
            self.jlpt_entries.insert(record.expression.clone(), record);
        }

        Ok(())
    }

    pub fn lookup(&self, word: &str) -> Option<DictLookup> {
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

    /// Transform a durf AST to one annotated with a DictDb.
    pub fn transform(&self, node: &mut durf::RawNode, config: &Config) -> Result<()> {
        match node {
            durf_parser::RawNode::Empty => {}
            durf_parser::RawNode::Section(section) => {
                for node in section.nodes.iter_mut() {
                    self.transform(node, config)?;
                }
            }
            durf_parser::RawNode::Text(text) => {
                let mut new_text = durf_parser::Text::new();

                // TODO: Configurable skip characters.
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
                        if let Some(lookup) = self.lookup(word) {
                            // TODO: Support keeping the previous text attributes.

                            // If this is a single character kana, skip.
                            // TODO: This should be smarter. We should check for
                            // things like counters.
                            if lookup.is_kana && word.chars().count() == 1 {
                                new_text
                                    .fragments
                                    .push(durf_parser::TextFragment::new(token.lemma(), None));
                                continue 'token_loop;
                            }

                            // If the jlpt level of this word is higher than our
                            // jlpt level, skip.
                            if lookup.jlpt.unwrap_or(0) as u32 > config.jlpt_level {
                                new_text
                                    .fragments
                                    .push(durf_parser::TextFragment::new(token.lemma(), None));
                                continue 'token_loop;
                            }

                            // Add appropriate attributes.
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
}

#[derive(RustEmbed)]
#[folder = "metadata/"]
struct Metadata;

/// Serde derive class for the CSV format of JLPT vocabulary.
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

pub struct DictLookup {
    pub is_kana: bool,
    pub kana: String,
    pub meaning: String,
    pub jlpt: Option<u8>,
}
