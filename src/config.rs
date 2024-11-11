use hyaline::SizeHint;

pub struct CrowsaConfig {
    pub(crate) content_path: String,
    pub(crate) window_title: String,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) debug: bool,
    pub(crate) resizable: SizeHint,
}

impl CrowsaConfig {
    #[must_use]
    pub fn new() -> Self {
        CrowsaConfig::default()
    }

    #[must_use]
    pub fn content_path(mut self, content_path: &str) -> Self {
        self.content_path = content_path.to_owned();
        self
    }

    #[must_use]
    pub fn window_title(mut self, window_title: &str) -> Self {
        self.window_title = window_title.to_owned();
        self
    }

    #[must_use]
    pub fn width(mut self, width: u32) -> Self {
        self.width = width;
        self
    }

    #[must_use]
    pub fn height(mut self, height: u32) -> Self {
        self.height = height;
        self
    }

    #[must_use]
    pub fn debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    #[must_use]
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
