use crate::Quark;
use crate::error::QuarkError;
use crate::cli::QUARKFOLDER;
use std::path::Path;

pub fn build_static(quark: &mut Quark) -> Result<(), QuarkError> {
    let index_path = QUARKFOLDER
        .get_file(Path::new("index.html"))
        .ok_or(QuarkError::RustEmbedAssetNotFoundError)?
        .contents_utf8()
        .ok_or(QuarkError::RustEmbedAssetError)?;

    quark.webview.set_html(index_path);
    Ok(())
}
