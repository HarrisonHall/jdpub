mod epub;
mod html;

use super::*;

use epub::*;
use html::*;

const JPDB_FILE_TEMPLATE: &'static str = "{{JPDB_FILE_TEMPLATE}}";

/// Export chapters according to the config.
pub fn export(chapters: &mut Vec<durf::Ast>, config: &Config) -> Result<()> {
    let lossy_output = config.output().to_string_lossy();

    // Output to the correct format.
    if lossy_output.ends_with(".epub") {
        return epub::export(chapters, config);
    } else if lossy_output.ends_with(".html") {
        return html::export(chapters, config);
    }

    bail!(
        "Inalid output format: '{}'.",
        config.output().to_string_lossy()
    );
}
