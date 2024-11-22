use crowsa::prelude::*;

fn main() -> Result<(), CrowsaError> {
    let config = CrowsaConfig::new()
        .frontend("./examples/frontend/cazic")
        .title("Cazic Music Player")
        .width(800)
        .height(600)
        .resizable(SizeHint::MIN);

    let crowsa = Crowsa::new(config)?;
    crowsa.run();
    Ok(())
}
