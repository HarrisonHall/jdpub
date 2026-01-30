use std::str::FromStr;

use super::*;

/// Annotate documents with readings and definitions.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Input chapters or book files.
    pub input: Vec<String>,
    /// Output file.
    #[arg(short, long)]
    pub output: Option<PathBuf>,
    /// Additional configuration file, parsed first.
    #[arg(short, long)]
    config: Option<PathBuf>,
    /// Debug flag.
    #[arg(short, long, default_value_t = false)]
    pub debug: bool,
    /// Verbose flag.
    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,
    /// Skip built-in configurations.
    #[arg(long, default_value_t = false)]
    pub do_not_use_builtin: bool,
}

impl Cli {
    pub fn new() -> Result<Self> {
        Ok(Self::parse())
    }

    pub fn config(&self) -> Result<Config> {
        let mut config = match self.do_not_use_builtin {
            true => Config::default(),
            false => Config::from_builtin()?,
        };

        // If another config was specified, merge it.
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

            config.merge(parsed_config)?;
        } else {
            tracing::debug!("No config file specified.");
        }

        // Add inputs.
        for input_file in self.input.iter() {
            if input_file.ends_with(".toml") {
                // Toml inputs are essentially additional configuration files.
                let config_text = match std::fs::read_to_string(&input_file) {
                    Ok(data) => data,
                    Err(e) => bail!("Failed to read config file: {e}"),
                };
                let parsed_config: Config = match toml::from_str(&config_text) {
                    Ok(c) => c,
                    Err(e) => bail!("Failed to parse config file: {e}"),
                };

                tracing::debug!("Successfully parsed config file '{}'.", input_file);

                config.merge(parsed_config)?;
            } else {
                // Otherwise we use the filse as a new chapter.
                config.import.chapters.push(ChapterConfig {
                    uri: input_file.clone(),
                    ..Default::default()
                });
            }
        }

        // Add output file info.
        if let Some(output) = &self.output {
            config.export.output_file = output.clone();
        }

        if config.export.output_file.to_string_lossy().is_empty() {
            config.export.output_file = PathBuf::from_str("./output.epub")?;
        }

        Ok(config)
    }
}
