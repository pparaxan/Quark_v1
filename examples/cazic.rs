use quark::prelude::*;

fn main() -> Result<(), QuarkError> {
    let config = QuarkConfig::new()
        .frontend("./examples/frontend/cazic")
        .title("Cazic Music Player")
        .width(800)
        .height(600)
        .resizable(SizeHint::MIN);

    let quark = Quark::new(config)?;
    quark.run();
    Ok(())
}
