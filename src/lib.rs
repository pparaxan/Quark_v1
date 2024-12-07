pub mod cli;
pub mod config;
pub mod error;
pub mod prelude;
pub mod setup;
pub mod webview;

use config::QuarkConfig;
use error::QuarkError;
use webview::{Webview, WebviewBuilder};

#[allow(dead_code)]
pub struct Quark {
    webview: Webview,
    config: QuarkConfig,
}

impl Quark {
    pub fn new(config: QuarkConfig) -> Result<Self, QuarkError> {
        let args = cli::parse_args();

        let webview = WebviewBuilder::new()
            .title(&config.title)
            .width(config.width)
            .height(config.height)
            .resize(config.resizable)
            .debug(cfg!(debug_assertions))
            .build();

        let mut quark = Quark { webview, config };

        if args.live {
            quark.setup_http()?;
        } else {
            quark.setup_static()?;
        }
        Ok(quark)
    }

    fn setup_static(&mut self) -> Result<(), QuarkError> {
        crate::setup::setup_static(self)
    }

    fn setup_http(&mut self) -> Result<(), QuarkError> {
        crate::setup::setup_http(self)
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
