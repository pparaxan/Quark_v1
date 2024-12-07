use libquark::prelude::*;

fn main() -> Result<(), QuarkError> {
    let config = QuarkConfig::new()
        .title("Hello World")
        .resizable(SizeHint::FIXED);

    let quark = Quark::new(config)?;
    quark.run();
    Ok(())
}
