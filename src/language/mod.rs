use super::*;

/// A dictionary database.
/// Currently, this only supports Japanese.
pub struct DictDb {
    common: HashMap<String, CommonVocab>,
    dict: jmdict_fast::Dict<'static>,
    config: LanguageConfig,
}

impl DictDb {
    /// Create a new DictDb.
    pub fn new(config: &Config) -> Result<Self> {
        let dict = jmdict_fast::Dict::load_default()?;

        let mut db = Self {
            common: HashMap::new(),
            dict,
            config: config.language.clone(),
        };

        db.read_kore_csv("kore_6k/kore.csv")?;

        db.read_jlpt_csv("jlpt/n1.csv", 1)?;
        db.read_jlpt_csv("jlpt/n2.csv", 2)?;
        db.read_jlpt_csv("jlpt/n3.csv", 3)?;
        db.read_jlpt_csv("jlpt/n4.csv", 4)?;
        db.read_jlpt_csv("jlpt/n5.csv", 5)?;

        tracing::debug!("Parsed {} JLPT entries.", db.common.len());

        Ok(db)
    }

    /// Read a jlpt csv.
    fn read_jlpt_csv(&mut self, file: &str, level: u8) -> Result<()> {
        let content = read_embedded_text::<JapaneseMetadata>(file)?;
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
            self.add_common(record.into());
        }

        Ok(())
    }

    /// Read the kore 6k csv.
    fn read_kore_csv(&mut self, file: &str) -> Result<()> {
        let content = read_embedded_text::<JapaneseMetadata>(file)?;
        let mut reader =
            csv::ReaderBuilder::new().from_reader(std::io::BufReader::new(content.as_bytes()));
        for result in reader.deserialize() {
            let record: KoreEntry = match result {
                Ok(record) => record,
                Err(e) => {
                    tracing::warn!("Unable to parse KoreEntry: {e}");
                    continue;
                }
            };
            self.add_common(record.into());
        }

        Ok(())
    }

    /// Add common word.
    fn add_common(&mut self, common: CommonVocab) -> bool {
        // Don't add single-character hiragana.
        if common.word.is_kana() && common.word.character_count() == 1 {
            return false;
        }

        // Otherwise insert both the word and its reading.
        if common.word != common.reading {
            self.common.insert(common.word.clone(), common.clone());
        }
        self.common.insert(common.reading.clone(), common);
        true
    }

    /// Lookup word.
    pub fn lookup(&self, word: &str) -> Option<DictLookup> {
        let word = word.trim();

        // Skip english words, numerals, etc.
        if word.is_english() {
            return None;
        }

        // If a common word, just use that.
        if let Some(common) = self.common.get(word) {
            return Some(common.into());
        }

        // Otherwise we look the word up.
        let results = match self.config.approximate {
            false => self.dict.lookup_exact(word),
            true => {
                let deinflected = self.dict.lookup_exact_with_deinflection(word);
                if deinflected.len() > 0 {
                    // If a deinflected word is common, we'll use that.
                    for entry in deinflected.iter() {
                        for kanji in entry.kanji.iter() {
                            if let Some(common) = self.common.get(&kanji.text) {
                                return Some(common.into());
                            }
                        }
                        for kana in entry.kana.iter() {
                            if let Some(common) = self.common.get(&kana.text) {
                                return Some(common.into());
                            }
                        }
                    }

                    // Otherwise, we use teh deinflected word.
                    deinflected
                } else {
                    self.dict.lookup_partial(word)
                }
            }
        };
        if results.len() == 0 {
            return None;
        }

        if results[0].kana.len() == 0
            || results[0].sense.len() == 0
            || results[0].sense[0].gloss.len() == 0
        {
            return None;
        }

        Some(DictLookup {
            is_kana: word.trim().is_kana(),
            kana: results[0].kana[0].text.clone(),
            meaning: results[0].sense[0].gloss[0].text.clone(),
            jlpt: JlptLevel::None,
        })
    }

    /// Transform a durf AST to one annotated with DictDb lookups.
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
                            // If an existing annotation exists, prefer that (e.g.,
                            // name readings).

                            // If this is a single character kana, skip.
                            // TODO: This should be smarter. We should check for
                            // things like counters.
                            if lookup.is_kana
                                && (word.character_count() == 1
                                    || lookup.kana.character_count() == 1)
                            {
                                new_text
                                    .fragments
                                    .push(durf_parser::TextFragment::new(token.lemma(), None));
                                continue 'token_loop;
                            }

                            // If the jlpt level of this word is higher than our
                            // jlpt level, skip.
                            if lookup.jlpt > config.language.japanese.lowest_level() {
                                new_text
                                    .fragments
                                    .push(durf_parser::TextFragment::new(token.lemma(), None));
                                continue 'token_loop;
                            }

                            // Add appropriate attributes.
                            let mut attributes = durf_parser::TextAttributes::default();
                            if lookup.jlpt <= config.language.japanese.definitions() {
                                attributes.tooltip = Some(format!(
                                    "{}[{}::{}::{}]",
                                    word, lookup.kana, lookup.meaning, lookup.jlpt,
                                ));
                            }
                            if lookup.jlpt <= config.language.japanese.furigana() {
                                attributes.annotation = Some(lookup.kana);
                            }
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

/// Dictionary lookup result.
pub struct DictLookup {
    pub is_kana: bool,
    pub kana: String,
    pub meaning: String,
    pub jlpt: JlptLevel,
}

/// Embeded Japanese language data.
#[derive(RustEmbed)]
#[folder = "metadata/language/jp"]
struct JapaneseMetadata;

#[derive(Debug, Clone)]
struct CommonVocab {
    word: String,
    reading: String,
    meaning: String,
    level: u8,
}

impl From<&CommonVocab> for DictLookup {
    fn from(value: &CommonVocab) -> Self {
        Self {
            is_kana: !value.word.is_kana(),
            kana: value.reading.clone(),
            meaning: value.meaning.clone(),
            jlpt: value.level.into(),
        }
    }
}

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

impl From<JlptEntry> for CommonVocab {
    fn from(value: JlptEntry) -> Self {
        Self {
            word: value.expression,
            reading: value.reading,
            meaning: value.meaning,
            level: value.level,
        }
    }
}

/// Serde derive class for the CSV format of kore 6k.
#[derive(Debug, serde::Deserialize)]
#[allow(unused)]
struct KoreEntry {
    #[serde(alias = "Vocab-expression")]
    expression: String,
    #[serde(alias = "Core-index")]
    core_index: usize,
    #[serde(alias = "Vocab-meaning")]
    meaning: String,
    #[serde(alias = "Vocab-kana")]
    reading: String,
    #[serde(alias = "jlpt ", alias = "jlpt")]
    _jlpt: String,
}

impl KoreEntry {
    fn level(&self) -> u8 {
        match self._jlpt.chars().last().unwrap_or('0') {
            '0' => 1,
            '1' => 2,
            '2' => 3,
            '3' => 4,
            '4' => 5,
            _ => 1,
        }
    }
}

impl From<KoreEntry> for CommonVocab {
    fn from(value: KoreEntry) -> Self {
        let level = value.level();
        Self {
            word: value.expression,
            reading: value.reading,
            meaning: value.meaning,
            level,
        }
    }
}

trait JapaneseText<T: AsRef<str> = Self>: AsRef<str> {
    #[allow(unused)]
    fn is_english(&self) -> bool {
        !self.is_kana() && !self.is_kanji()
    }

    fn is_kana(&self) -> bool {
        use wana_kana::IsJapaneseStr;

        self.as_ref().is_japanese() && !self.is_kanji()
    }

    fn is_kanji(&self) -> bool {
        use wana_kana::IsJapaneseStr;

        self.as_ref().contains_kanji()
    }

    fn character_count(&self) -> usize {
        self.as_ref().trim().chars().count()
    }
}

impl JapaneseText for &str {}

impl JapaneseText for String {}
