pub mod config;
pub mod error;
pub mod prelude;

use config::QuarkConfig;
use error::QuarkError;
use hyaline::{Webview, WebviewBuilder};
use std::{str, path::{PathBuf, Path}};
use rust_embed::RustEmbed;

#[cfg_attr(debug_assertions, allow(dead_code))]
const BUILDTYPE: bool = cfg!(debug_assertions);


#[derive(RustEmbed)]
#[folder = "$CARGO_MANIFEST_DIR"]
struct Asset;

pub struct Quark {
    webview: Webview,
    config: QuarkConfig,
}

impl Quark {
    pub fn new(config: QuarkConfig) -> Result<Self, QuarkError> {
        let webview = WebviewBuilder::new()
            .title(&config.title)
            .width(config.width)
            .height(config.height)
            .resize(config.resizable)
            .debug(BUILDTYPE)
            .build();

        let mut quark = Quark { webview, config };

        quark.setup()?;
        Ok(quark)
    }

    fn setup(&mut self) -> Result<(), QuarkError> {
        let index_html_path = format!("{}/index.html", self.config.frontend);
        let index_html = Asset::get(&index_html_path)
            .ok_or(QuarkError::RustEmbedAssetNotFoundError)?;

        let html_content = String::from_utf8(index_html.data.to_vec())
            .map_err(|_| QuarkError::RustEmbedAssetError)?;

        self.webview.set_html(&html_content); // just uses `self.webview.dispatch` as the backend
        Ok(())
    }

    pub fn bind<F>(&mut self, name: &str, handler: F)
    where
        F: FnMut(&str, &str) + 'static,
    {
        self.webview.bind(name, handler);
    }

    pub fn eval(&mut self, js: &str) {
        self.webview.eval(js);
    }

    pub fn run(mut self) {
        self.webview.run();
    }
}
