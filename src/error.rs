#[derive(Debug)]
pub enum QuarkError {
    InitializationFailed,
    PathError,
    WebviewError,
    RustEmbedAssetError,
    RustEmbedAssetNotFoundError,
    ServerError,
}
