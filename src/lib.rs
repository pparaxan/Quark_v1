pub mod config;
pub mod error;
pub mod prelude;

use config::QuarkConfig;
use error::QuarkError;
use hyaline::{Webview, WebviewBuilder};
use std::path::PathBuf;

#[cfg_attr(debug_assertions, allow(dead_code))]
const BUILDTYPE: bool = cfg!(debug_assertions);

pub struct Quark {
    webview: Webview,
    config: QuarkConfig,
}

impl Quark {
    pub fn new(config: QuarkConfig) -> Result<Self, QuarkError> {
        let webview = WebviewBuilder::new()
            .title(&config.title)
            .resize(config.resizable)
            .debug(if BUILDTYPE { true } else { false })
            .build();

        let mut quark = Quark { webview, config };

        quark.setup()?;
        Ok(quark)
    }

    fn setup(&mut self) -> Result<(), QuarkError> {
        let current_dir = std::env::current_dir().map_err(|_| QuarkError::PathError)?;
        let content_path = PathBuf::from(&self.config.frontend).join("index.html");
        let full_path = current_dir.join(content_path);

        if !full_path.exists() {
            return Err(QuarkError::PathError);
        }

        let uri = format!("file://{}", full_path.display()); // TODO: make it built in the exec?
        // Like make it doesn't have to depend on a local file
        self.webview.navigate(&uri);
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
