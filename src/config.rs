use super::*;

#[derive(RustEmbed)]
#[folder = "metadata/config"]
struct BuiltInConfigMetadata;

/// Configuration file.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Config {
    /// Parse config.
    #[serde(default)]
    pub parse: ParseConfig,
    /// Language configuration.
    #[serde(default, alias = "lang")]
    pub language: LanguageConfig,
    /// Import configuration.
    #[serde(default)]
    pub import: ImportConfig,
    // Export configuration.
    #[serde(default)]
    pub export: ExportConfig,
}

impl Config {
    /// Create a config using the built-in configurations.
    pub fn from_builtin() -> Result<Self> {
        let mut config = Config::default();
        for path in BuiltInConfigMetadata::iter() {
            let builtin = read_embedded_toml::<Config, BuiltInConfigMetadata>(&path)?;
            config.merge(builtin)?;
            tracing::debug!("Read built-in configuration {path}.");
        }

        Ok(config)
    }

    /// Merge configuration files.
    /// This needs to be improved to support only overriding sections that are
    /// specified.
    pub fn merge(&mut self, other: Config) -> Result<()> {
        // Merge parse section.
        self.parse.merge(other.parse)?;

        // Merge import section.
        self.import.chapters.extend(other.import.chapters);

        // Merge language support. TODO
        self.language = other.language;

        // Merge export.
        self.export.merge(other.export)?;

        Ok(())
    }
}

/// Parsing configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ParseConfig {
    #[serde(default)]
    pub html: HtmlParseConfig,
}

impl ParseConfig {
    fn merge(&mut self, other: ParseConfig) -> Result<()> {
        self.html.merge(other.html)?;

        Ok(())
    }
}

/// HTML parsing configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HtmlParseConfig {
    /// User-agent for web requests.
    #[serde(default, alias = "user-agent")]
    pub user_agent: String,
    /// The allow parse rules.
    #[serde(default, alias = "pass")]
    pub allow: Vec<durf::ParseRule>,
    /// The skip parse rules.
    #[serde(default, alias = "block", alias = "deny")]
    pub skip: Vec<durf::ParseRule>,
    /// Maximum parse depth.
    #[serde(default)]
    depth: Option<usize>,
}

impl HtmlParseConfig {
    fn merge(&mut self, other: HtmlParseConfig) -> Result<()> {
        if !other.user_agent.is_empty() {
            self.user_agent = other.user_agent;
        }
        self.allow.extend(other.allow);
        self.skip.extend(other.skip);
        if let Some(depth) = &other.depth {
            self.depth = Some(*depth);
        }

        Ok(())
    }
}

impl HtmlParseConfig {
    pub fn depth(&self) -> usize {
        self.depth.unwrap_or(10)
    }

    /// Generate parse flags from teh import configuration.
    pub fn parse_flags(&self) -> Result<durf::ParseFlags> {
        let mut pf = durf::ParseFlags::default();

        if self.allow.len() > 0 || self.skip.len() > 0 {
            pf.parsing = false;
            pf.allow = self.allow.clone();
            pf.skip = self.skip.clone();
        }

        Ok(pf)
    }
}

impl Default for HtmlParseConfig {
    fn default() -> Self {
        Self {
            user_agent: format!(
                "{}/{}",
                std::env!("CARGO_PKG_NAME"),
                std::env!("CARGO_PKG_VERSION")
            ),
            allow: Vec::new(),
            skip: Vec::new(),
            depth: None,
        }
    }
}

/// Language configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageConfig {
    /// Use approximate lookups and definitions.
    pub approximate: bool,
    /// Japanese language configuration.
    #[serde(default)]
    pub japanese: JapaneseLanguageConfig,
}

impl Default for LanguageConfig {
    fn default() -> Self {
        Self {
            approximate: true,
            japanese: JapaneseLanguageConfig::default(),
        }
    }
}

/// Japanese language configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JapaneseLanguageConfig {
    /// JLPT level.
    #[serde(default, alias = "jlpt-level")]
    pub jlpt_level: u32,
}

impl Default for JapaneseLanguageConfig {
    fn default() -> Self {
        Self { jlpt_level: 3 }
    }
}

/// Import configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ImportConfig {
    /// Chapter config.
    #[serde(alias = "chapter")]
    pub chapters: Vec<ChapterConfig>,
}

/// Chapter configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChapterConfig {
    /// Chapter title. This defaults to 'Chapter <n>'.
    #[serde(default)]
    pub title: String,
    /// Chapter uri.
    #[serde(alias = "path", alias = "url", alias = "file")]
    pub uri: String,
}

/// Export configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExportConfig {
    /// Book title.
    #[serde(default)]
    pub title: String,
    /// Book author.
    #[serde(default)]
    pub author: String,
    /// A cover file.
    #[serde(default, alias = "cover", alias = "cover-file")]
    pub cover: Option<PathBuf>,
    /// The output file.
    #[serde(default, alias = "file", alias = "output", alias = "path")]
    pub output_file: PathBuf,
}

impl ExportConfig {
    fn merge(&mut self, other: Self) -> Result<()> {
        if !other.title.is_empty() {
            self.title = other.title;
        }
        if !other.author.is_empty() {
            self.author = other.author;
        }
        if let Some(cover) = other.cover {
            self.cover = Some(cover);
        }
        if !other.output_file.to_string_lossy().is_empty() {
            self.output_file = other.output_file;
        }

        Ok(())
    }

    pub fn book(&self) -> Result<Book> {
        Ok(Book {
            title: self.title.clone(),
            author: self.author.clone(),
            chapters: Vec::new(),
        })
    }

    pub fn export_type(&self) -> ExportType {
        let output_lossy = self.output_file.to_string_lossy();
        if output_lossy.ends_with(".epub") {
            return ExportType::Epub;
        } else if output_lossy.ends_with(".html") {
            return ExportType::Html;
        }

        tracing::debug!("Failed to derive export type, using epub.");
        ExportType::Epub
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportType {
    #[serde(alias = "epub")]
    Epub,
    Html,
    // Markdown,
}

impl Default for ExportType {
    fn default() -> Self {
        Self::Epub
    }
}
