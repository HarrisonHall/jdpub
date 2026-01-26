mod epub;
mod html;

use super::*;

use html::*;

const JPDB_FILE_TEMPLATE: &'static str = "{{JPDB_FILE_TEMPLATE}}";

/// Export chapters according to the config.
pub fn export(book: &mut Book, config: &Config) -> Result<()> {
    match config.export.export_type() {
        config::ExportType::Epub => epub::export(book, config),
        config::ExportType::Html => html::export(book, config),
    }
}
