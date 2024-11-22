use quark::prelude::*;

fn main() -> Result<(), QuarkError> {
    let config = QuarkConfig::new()
        .frontend("./examples/frontend/basic")
        .title("Hello World")
        .resizable(SizeHint::FIXED);

    let quark = Quark::new(config)?;
    quark.run();
    Ok(())
}
