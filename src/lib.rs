pub mod config;
pub mod error;
pub mod prelude;

use config::CrowsaConfig;
use error::CrowsaError;
use hyaline::{Webview, WebviewBuilder};
use std::path::PathBuf;

pub struct Crowsa {
    webview: Webview,
    config: CrowsaConfig,
}

impl Crowsa {
    pub fn new(config: CrowsaConfig) -> Result<Self, CrowsaError> {
        let webview = WebviewBuilder::new()
            .title(&config.title)
            .resize(config.resizable)
            .debug(config.debug)
            .build();

        let mut crowsa = Crowsa { webview, config };

        crowsa.setup()?;
        Ok(crowsa)
    }

    fn setup(&mut self) -> Result<(), CrowsaError> {
        let current_dir = std::env::current_dir().map_err(|_| CrowsaError::PathError)?;
        let content_path = PathBuf::from(&self.config.frontend).join("index.html");
        let full_path = current_dir.join(content_path);

        if !full_path.exists() {
            return Err(CrowsaError::PathError);
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
