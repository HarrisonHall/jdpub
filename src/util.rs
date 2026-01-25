use super::*;

/// Embedded data type.
#[derive(Clone)]
pub struct EmbeddedData(Cow<'static, [u8]>);

impl EmbeddedData {
    /// Create empty embedded data placeholder.
    pub fn empty() -> Self {
        Self(Cow::from(&[]))
    }
}

impl std::ops::Deref for EmbeddedData {
    type Target = Cow<'static, [u8]>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Read embedded file as data.
pub fn read_embedded_data<Embed: RustEmbed>(path: impl AsRef<str>) -> Result<EmbeddedData> {
    match Embed::get(path.as_ref()) {
        Some(file) => Ok(EmbeddedData(file.data)),
        None => bail!("Unable to find file {}", path.as_ref()),
    }
}

/// Parse embedded file to text.
pub fn read_embedded_text<Embed: RustEmbed>(path: impl AsRef<str>) -> Result<String> {
    let data = read_embedded_data::<Embed>(path.as_ref())?;
    match std::str::from_utf8(&data) {
        Ok(file) => Ok(file.to_string()),
        _ => bail!("Unable to convert binary file {} to string", path.as_ref()),
    }
}

/// Read embedded toml.
pub fn read_embedded_toml<T: DeserializeOwned, Embed: RustEmbed>(
    path: impl AsRef<str>,
) -> Result<T> {
    let data = read_embedded_data::<Embed>(path.as_ref())?;
    match std::str::from_utf8(&data) {
        Ok(file) => toml::from_str::<T>(file)
            .map_err(|e| anyhow!("read_embedded_toml error for {}: {e}", path.as_ref())),
        _ => bail!("Unable to convert binary file {} to string", path.as_ref()),
    }
}

pub fn get_mimetype(resource: impl AsRef<str>) -> &'static str {
    let resource = resource.as_ref();

    if resource.ends_with(".css") {
        "text/css"
    } else if resource.ends_with(".epub") {
        "application/epub+zip"
    } else if resource.ends_with(".gif") {
        "image/gif"
    } else if resource.ends_with(".html") {
        "text/html"
    } else if resource.ends_with(".ico") {
        "image/vdn.microsoft.icon"
    } else if resource.ends_with(".jpg") {
        "image/jpeg"
    } else if resource.ends_with(".jpeg") {
        "image/jpeg"
    } else if resource.ends_with(".js") {
        "text/javascript"
    } else if resource.ends_with(".md") {
        "text/plain"
    } else if resource.ends_with(".otf") {
        "font/otf"
    } else if resource.ends_with(".png") {
        "image/png"
    } else if resource.ends_with(".toml") {
        "text/plain"
    } else if resource.ends_with(".ttf") {
        "font/ttf"
    } else if resource.ends_with(".txt") {
        "text/plain"
    } else if resource.ends_with(".woff") {
        "font/woff"
    } else if resource.ends_with(".woff2") {
        "font/woff2"
    } else if resource.ends_with(".xhtml") {
        "application/xhtml+xml"
    } else {
        tracing::warn!(
            "Failed to find mimetype for {resource}, defaulting to application/octet-stream."
        );
        "application/octet-stream"
    }
}
