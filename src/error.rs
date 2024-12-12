#[derive(Debug)]
pub enum QuarkError {
    FrontendPathMissing,
    IncludeDirCouldntConvertToUTF8,
    ServerPortIsntAvailable,
    ServerError,
}
