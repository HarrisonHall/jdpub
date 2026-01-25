use super::*;

/// Configuration file.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    /// User-agent for web requests.
    #[serde(default, alias = "user-agent")]
    pub user_agent: String,
    /// The allow parse rules.
    #[serde(default)]
    pub allow: Vec<durf::ParseRule>,
    /// The skip parse rules.
    #[serde(default)]
    pub skip: Vec<durf::ParseRule>,
    /// Maximum parse depth.
    #[serde(default)]
    pub depth: usize,
    /// JLPT level.
    #[serde(default, alias = "jlpt-level")]
    pub jlpt_level: u32,
    /// A cover file.
    #[serde(default, alias = "cover", alias = "cover-file")]
    pub cover_file: Option<PathBuf>,
    /// Book title.
    #[serde(default)]
    pub title: String,
    /// Book author.
    #[serde(default)]
    pub author: String,
    // The following fields are not imported or exported:
    /// The input file/uri.
    #[serde(skip)]
    input: String,
    // The output file.
    #[serde(skip)]
    output: PathBuf,
}

impl Config {
    pub fn parse_flags(&self) -> Result<durf::ParseFlags> {
        let mut pf = durf::ParseFlags::default();

        if self.allow.len() > 0 || self.skip.len() > 0 {
            pf.parsing = false;
            pf.allow = self.allow.clone();
            pf.skip = self.skip.clone();
            tracing::trace!("Using ParseFlags from configuration file.");
        }

        Ok(pf)
    }

    pub fn set_input_output(&mut self, input: impl Into<String>, output: impl Into<PathBuf>) {
        self.input = input.into();
        self.output = output.into();
    }

    pub fn input(&self) -> &str {
        self.input.as_str()
    }

    pub fn output(&self) -> &Path {
        self.output.as_path()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            user_agent: format!(
                "{}/{}",
                std::env!("CARGO_PKG_NAME"),
                std::env!("CARGO_PKG_VERSION")
            ),
            allow: Vec::new(),
            skip: Vec::new(),
            depth: 10,
            jlpt_level: 2,
            title: "jdpub".into(),
            cover_file: None,
            author: "".into(),
            input: "".into(),
            output: PathBuf::new(),
        }
    }
}
