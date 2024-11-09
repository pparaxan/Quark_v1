use crowsa::{Crowsa, CrowsaError, CrowsaConfig, SizeHint};

fn main() -> Result<(), CrowsaError> {
    // let config = CrowsaConfig {
    //     content_path: String::from("./examples/global_html"),
    //     window_title: String::from("Hello World"),
    //     width: 1024,
    //     height: 768,
    //     debug: true,
    //     resizable: 1,
    // };
    let config = CrowsaConfig::new()
        .content_path("./examples/global_html")
        .window_title("Hello World")
        .resizable(SizeHint::MIN);

    let crowsa = Crowsa::new(config)?;
    crowsa.run();
    Ok(())
}
