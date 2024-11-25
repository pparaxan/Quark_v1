pub mod prelude;
pub mod config;
pub mod error;
pub mod webview;

use config::QuarkConfig;
use error::QuarkError;
use webview::{Webview, WebviewBuilder};
use std::{str, path::Path};
use include_dir::{Dir, include_dir};

#[cfg_attr(debug_assertions, allow(dead_code))]
const BUILDTYPE: bool = cfg!(debug_assertions);
static QUARKFOLDER: Dir = include_dir!("$CARGO_MANIFEST_DIR/src_quark");

#[allow(dead_code)]
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
        let index_html = Path::new("index.html");
        let index_path = QUARKFOLDER.get_file(index_html)
            .ok_or(QuarkError::RustEmbedAssetNotFoundError)?
            .contents_utf8()
            .ok_or(QuarkError::RustEmbedAssetError)?;

        self.webview.set_html(index_path); // just uses `self.webview.dispatch` as the backend
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
