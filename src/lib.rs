pub mod cli;
pub mod config;
pub mod error;
pub mod prelude;
pub mod package;

use crate::cli::build_http::*;
use crate::cli::build_static::*;
use config::QuarkConfig;
use error::QuarkError;
use hyaline::{Webview, WebviewBuilder};

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
            // quark.build_http()?;
            build_http(&mut quark)?;
        } else {
            // quark.build_static()?;
            build_static(&mut quark)?;
        }
        Ok(quark)
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
