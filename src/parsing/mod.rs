use super::*;

/// Parse the input to a durf ast.
pub async fn parse(config: &Config) -> Result<durf::Ast> {
    let mut uri: String = config.input().into();

    tracing::info!("URI: {uri}");

    let mut as_html: Option<String> = None;

    // Fetch html.
    if uri.starts_with("http://") || uri.starts_with("https://") {
        let client = reqwest::ClientBuilder::new()
            .user_agent(&config.user_agent)
            .build()?;
        let res = client.get(&uri).send().await?;
        let body = res.text().await?;
        as_html = Some(body);
    }

    // Remove file uri.
    if uri.starts_with("file://") {
        uri = uri.replace("file://", "");
    }

    // If still not parsed, try to treat as file.
    if as_html.is_none() {
        let normalized_path = match shellexpand::full(&uri) {
            Ok(fname) => fname,
            Err(e) => bail!("Unable to normalize filename: {e}"),
        }
        .into_owned();
        // let normalized_path = uri.clone();
        tracing::info!("p: {}", normalized_path);
        let body = match std::fs::read_to_string(&normalized_path) {
            Ok(body) => body,
            Err(e) => bail!("Unable to read file: {e}"),
        };
        as_html = Some(body);
    }

    let as_html = match as_html {
        Some(as_html) => as_html,
        None => bail!("Unable to get contents of uri '{uri}'."),
    };

    // Parse to durf.
    let flags = config.parse_flags()?;
    let mut ast = durf_parser::Ast::from_html(&as_html, flags)?;
    ast.minimize();

    tracing::trace!("AST:\n{}", ast);

    Ok(ast)
}
