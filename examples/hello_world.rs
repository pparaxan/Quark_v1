use crowsa::{Crowsa, CrowsaError, CrowsaConfig};

fn main() -> Result<(), CrowsaError> {
    let config = CrowsaConfig {
        content_path: String::from("./examples/global_html"),
        window_title: String::from("Hello World"),
        width: 1024,
        height: 768,
        debug: true,
        resizable: true,
    };

    let crowsa = Crowsa::new(config)?;
    crowsa.run();
    Ok(())
}
