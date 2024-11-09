pub mod prelude;

use std::path::PathBuf;
use hyaline::{Webview, WebviewBuilder, SizeHint};

#[derive(Debug)]
pub enum CrowsaError {
    InitializationFailed,
    PathError,
    WebviewError,
}

pub struct CrowsaConfig {
    pub content_path: String,
    pub window_title: String,
    pub width: u32,
    pub height: u32,
    pub debug: bool,
    pub resizable: SizeHint,
}

impl CrowsaConfig {
    pub fn new() -> Self {
        CrowsaConfig::default()
    }

    pub fn content_path(mut self, content_path: &str) -> Self {
        self.content_path = content_path.to_owned(); // TODO: possibility of Box<str>? saves memory
                                                     // by not having metadata overhead
        self
    }

    pub fn window_title(mut self, window_title: &str) -> Self {
        self.window_title = window_title.to_owned();
        self
    }

    pub fn width(mut self, width: u32) -> Self {
        self.width = width;
        self
    }

    pub fn height(mut self, height: u32) -> Self {
        self.height = height;
        self
    }

    // hmm. i want to make it so if this is ever called, it will toggle debug, but the standard
    // library doesn't do that: see std::fs::OpenOptions
    pub fn debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    // TODO: use enums cause xandr doesnt know what hes doing
    pub fn resizable(mut self, resizable: SizeHint) -> Self {
        self.resizable = resizable;
        self
    }
}

impl Default for CrowsaConfig {
    fn default() -> Self {
        Self {
            content_path: String::from("src_crowsa"),
            window_title: String::from("A Crowsa Application"),
            width: 800,
            height: 600,
            debug: true,
            resizable: SizeHint::FIXED,
        }
    }
}

pub struct Crowsa {
    webview: Webview,
    config: CrowsaConfig,
}

impl Crowsa {
    pub fn new(config: CrowsaConfig) -> Result<Self, CrowsaError> {
        let webview = WebviewBuilder::new()
            .title(&config.window_title)
            .resize(config.resizable)
            .debug(config.debug)
            .build();

        let mut crowsa = Crowsa {
            webview,
            config,
        };

        crowsa.setup()?;
        Ok(crowsa)
    }

    fn setup(&mut self) -> Result<(), CrowsaError> {
        let current_dir = std::env::current_dir().map_err(|_| CrowsaError::PathError)?;
        let content_path = PathBuf::from(&self.config.content_path).join("index.html");
        let full_path = current_dir.join(content_path);

        if !full_path.exists() {
            return Err(CrowsaError::PathError);
        }

        let uri = format!("file://{}", full_path.display());
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
