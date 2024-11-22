use hyaline::SizeHint;


/// Defines the primary configuration for a Quark application.
///
/// # Examples
///
/// ```rust, ignore
/// # fn main() -> Result<(), QuarkError> {
/// let config = QuarkConfig::new()
///     .frontend("./path_to_html_folder")
///     .title("Quark!")
///     .resizable(SizeHint::MIN);
///
/// // QuarkConfig is meant to be used with `Quark::new`
/// let quark = Quark::new(config)?;
/// quark.run();
/// # Ok(())
/// # }
/// ```
///
/// Also see [`Quark`]
pub struct QuarkConfig {
    pub(crate) frontend: String,
    pub(crate) title: String,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) debug: bool,
    pub(crate) resizable: SizeHint,
}

impl QuarkConfig {
    /// Creates a new default `QuarkConfig`.
    ///
    /// This method is identical to `QuarkConfig::default()`, but is included as a matter of
    /// convenience.
    #[must_use]
    pub fn new() -> Self {
        QuarkConfig::default()
    }

    /// Sets the `QuarkConfig.frontend` value.
    ///
    /// The `frontend` value determines the relative path to your file in which Quark will
    /// include your web frontend.
    #[must_use]
    pub fn frontend(mut self, frontend: &str) -> Self {
        self.frontend = frontend.to_owned(); // TODO: possibility of Box<str>? saves memory by not having metadata overhead
        self
    }

    /// Sets the `QuarkConfig.title` value.
    ///
    /// The `title` value determines the value to give to the underlying operating system
    /// what title to give your application.
    #[must_use]
    pub fn title(mut self, title: &str) -> Self {
        self.title = title.to_owned();
        self
    }

    /// Sets the `QuarkConfig.width` value.
    ///
    /// The `width` value determines the width of the webview should render at.
    #[must_use]
    pub fn width(mut self, width: u32) -> Self {
        self.width = width;
        self
    }

    /// Sets the `QuarkConfig.height` value.
    ///
    /// The `height` value determines the height of the webview should render at.
    #[must_use]
    pub fn height(mut self, height: u32) -> Self {
        self.height = height;
        self
    }

    // hmm. i want to make it so if this is ever called, it will toggle debug, but the standard
    // library doesn't do that: see std::fs::OpenOptions
    /// Sets the `QuarkConfig.debug` value.
    ///
    /// The `debug` value makes debugging your frontend application much easier in two ways:
    /// - Allows a feature of the webview called "inspect element" to be opened;
    /// - JavaScript logs will be printed to the terminal
    ///
    /// It may be undesirable to leave this on in release mode.
    #[must_use]
    pub fn debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    /// Sets the `QuarkConfig.resizable` value.
    ///
    /// The `resizable` value determines the resizing conditions for the frontend application.
    ///
    /// Also see [`SizeHint`]
    ///
    // TODO: this docstring may not work. link to html resource later
    /// [`SizeHint`]: hyaline::SizeHint
    #[must_use]
    pub fn resizable(mut self, resizable: SizeHint) -> Self {
        self.resizable = resizable;
        self
    }
}

impl Default for QuarkConfig {
    fn default() -> Self {
        Self {
            frontend: String::from("src_quark"),
            title: String::from("Quark Application"),
            width: 800,
            height: 600,
            debug: true,
            resizable: SizeHint::MAX,
        }
    }
}
