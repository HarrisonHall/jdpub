use super::*;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Input uri/file.
    pub input: String,
    /// Output file.
    #[arg(short, long)]
    pub output: PathBuf,
    /// Configuration file.
    #[arg(short, long)]
    config: Option<PathBuf>,
    /// debug flag.
    #[arg(short, long, default_value_t = false)]
    pub debug: bool,
    /// Verbose flag.
    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,
    /// Cover image uri/file.
    #[arg(long)]
    pub cover: Option<PathBuf>,
    /// Title of export.
    #[arg(long)]
    pub title: Option<String>,
    /// Author of export.
    #[arg(long)]
    pub author: Option<String>,
}

impl Cli {
    pub fn new() -> Result<Self> {
        Ok(Self::parse())
    }

    pub fn config(&self) -> Result<Config> {
        let mut config = Config::default();
        if let Some(config_path) = &self.config {
            let config_text = match std::fs::read_to_string(config_path) {
                Ok(data) => data,
                Err(e) => bail!("Failed to read config file: {e}"),
            };
            let parsed_config: Config = match toml::from_str(&config_text) {
                Ok(c) => c,
                Err(e) => bail!("Failed to parse config file: {e}"),
            };

            tracing::debug!(
                "Successfully parsed config file '{}'.",
                config_path.to_string_lossy()
            );
            config = parsed_config;
        } else {
            tracing::debug!("No config file specified.");
        }

        if let Some(cover_file) = &self.cover {
            config.cover_file = Some(cover_file.clone());
        }

        if let Some(title) = &self.title {
            config.title = title.clone();
        }

        if let Some(author) = &self.author {
            config.author = author.clone();
        }

        config.set_input_output(&self.input, &self.output);

        Ok(config)
    }
}
