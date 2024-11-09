use crowsa::prelude::*;

fn main() -> Result<(), CrowsaError> {
    let config = CrowsaConfig::new()
        .content_path("./examples/global_html")
        .window_title("Hello World")
        .resizable(SizeHint::MIN);

    let crowsa = Crowsa::new(config)?;
    crowsa.run();
    Ok(())
}
