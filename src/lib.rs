pub mod prelude;

use hyaline::{SizeHint, Webview, WebviewBuilder};
use std::path::PathBuf;

#[derive(Debug)]
pub enum CrowsaError {
    InitializationFailed,
    PathError,
    WebviewError,
}

/// Defines the primary configuration for a Crowsa application.
///
/// # Examples
///
/// ```rust, ignore
/// # fn main() -> Result<(), CrowsaError> {
/// let config = CrowsaConfig::new()
///     .content_path("./examples/global_html")
///     .window_title("Hello, World!")
///     .resizable(SizeHint::MIN);
///
/// // CrowsaConfig is meant to be used with `Crowsa::new`
/// let crowsa = Crowsa::new(config)?;
/// crowsa.run();
/// # Ok(())
/// # }
/// ```
///
/// Also see [`Crowsa`]
pub struct CrowsaConfig {
    content_path: String,
    window_title: String,
    width: u32,
    height: u32,
    debug: bool,
    resizable: SizeHint,
}

impl CrowsaConfig {
    /// Creates a new default `CrowsaConfig`.
    ///
    /// This method is identical to `CrowsaConfig::default()`, but is included as a matter of
    /// convenience.
    pub fn new() -> Self {
        CrowsaConfig::default()
    }

    /// Sets the `CrowsaConfig.content_path` value.
    ///
    /// The `content_path` value determines the relative path to your file in which Crowsa will
    /// include your web frontend.
    pub fn content_path(mut self, content_path: &str) -> Self {
        self.content_path = content_path.to_owned(); // TODO: possibility of Box<str>? saves memory
        // by not having metadata overhead
        self
    }

    /// Sets the `CrowsaConfig.window_title` value.
    ///
    /// The `window_title` value determines the value to give to the underlying operating system
    /// what title to give your application.
    pub fn window_title(mut self, window_title: &str) -> Self {
        self.window_title = window_title.to_owned();
        self
    }

    /// Sets the `CrowsaConfig.width` value.
    ///
    /// The `width` value determines the width of the webview should render at.
    ///
    /// # Platform specific behaviour
    ///
    /// Some users may use a tiling window manager, which may not respect these values, and instead
    /// tile their application, which does not have a size known at compile-time.
    // FIXME: ignore tiling?
    pub fn width(mut self, width: u32) -> Self {
        self.width = width;
        self
    }

    /// Sets the `CrowsaConfig.height` value.
    ///
    /// The `height` value determines the height of the webview should render at.
    ///
    /// # Platform specific behaviour
    ///
    /// Some users may use a tiling window manager, which may not respect these values, and instead
    /// tile their application, which does not have a size known at compile-time.
    pub fn height(mut self, height: u32) -> Self {
        self.height = height;
        self
    }

    // hmm. i want to make it so if this is ever called, it will toggle debug, but the standard
    // library doesn't do that: see std::fs::OpenOptions
    /// Sets the `CrowsaConfig.debug` value.
    ///
    /// The `debug` value makes debugging your frontend application much easier in two ways:
    /// - Allows a feature of the webview called "inspect element" to be opened;
    /// - JavaScript logs will be printed to the terminal
    ///
    /// It may be undesirable to leave this on in release mode.
    pub fn debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    /// Sets the `CrowsaConfig.resizable` value.
    ///
    /// The `resizable` value determines the resizing conditions for the frontend application.
    ///
    /// Also see [`SizeHint`]
    ///
    // TODO: this docstring may not work. link to html resource later
    /// [`SizeHint`]: hyaline::SizeHint
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

        let mut crowsa = Crowsa { webview, config };

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
