use crate::Quark;
use crate::error::QuarkError;
use crate::cli::QUARKFOLDER;
use std::path::Path;

pub fn build_static(quark: &mut Quark) -> Result<(), QuarkError> {
    let path = QUARKFOLDER
        .get_file(Path::new("index.html"))
        .ok_or(QuarkError::FrontendPathMissing)?
        .contents_utf8()
        .ok_or(QuarkError::IncludeDirCouldntConvertToUTF8)?;

    quark.webview.set_html(path);
    Ok(())
}
