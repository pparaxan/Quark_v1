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
    pub resizable: u8,
}

impl Default for CrowsaConfig {
    fn default() -> Self {
        Self {
            content_path: String::from("src_crowsa"),
            window_title: String::from("A Crowsa Application"),
            width: 800,
            height: 600,
            debug: true,
            resizable: 3,
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
            .resize(if config.resizable > 0 && config.resizable < 4 {
                match config.resizable {
                    0 => SizeHint::NONE,
                    1 => SizeHint::MIN,
                    2 => SizeHint::MAX,
                    3 => SizeHint::FIXED,
                    _ => SizeHint::FIXED,
                }
            } else {
                SizeHint::FIXED
            })
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
