use super::{SizeHint, Webview, Window};

#[derive(Default)]
pub struct WebviewBuilder<'a> {
    title: Option<&'a str>,
    url: Option<&'a str>,
    init: Option<&'a str>,
    eval: Option<&'a str>,
    width: usize,
    height: usize,
    resize: SizeHint,
    debug: bool,
    dispatch: Option<Box<dyn FnOnce(&mut Webview) + Send + 'static>>,
    window: Option<&'a mut Window>,
}

impl<'a> WebviewBuilder<'a> {
    pub fn new() -> Self {
        WebviewBuilder::default()
    }

    pub fn debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    pub fn window(mut self, window: &'a mut Window) -> Self {
        self.window = Some(window);
        self
    }

    pub fn title(mut self, title: &'a str) -> Self {
        self.title = Some(title);
        self
    }

    pub fn url(mut self, url: &'a str) -> Self {
        self.url = Some(url);
        self
    }

    pub fn init(mut self, init: &'a str) -> Self {
        self.init = Some(init);
        self
    }

    pub fn eval(mut self, eval: &'a str) -> Self {
        self.eval = Some(eval);
        self
    }

    pub fn width(mut self, width: usize) -> Self {
        self.width = width;
        self
    }

    pub fn height(mut self, height: usize) -> Self {
        self.height = height;
        self
    }

    pub fn resize(mut self, hint: SizeHint) -> Self {
        self.resize = hint;
        self
    }

    pub fn dispatch<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&mut Webview) + Send + 'static,
    {
        self.dispatch = Some(Box::new(f));
        self
    }

    pub fn build(self) -> Webview {
        let mut w = Webview::create(self.debug, self.window);
        if let Some(title) = self.title {
            w.set_title(title);
        }

        if let Some(init) = self.init {
            w.init(init);
        }

        if let Some(url) = self.url {
            w.navigate(url);
        }

        if let Some(eval) = self.eval {
            w.eval(eval);
        }

        w.set_size(self.width as u16, self.height as u16, self.resize);

        if let Some(f) = self.dispatch {
            w.dispatch(f);
        }

        w
    }
}
