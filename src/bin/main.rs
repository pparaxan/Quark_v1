// No need for examples, you can just `cargo run`
use libquark::prelude::*;

fn main() -> Result<(), QuarkError> {
    let config = QuarkConfig::new()
        .title("A Quark Application")
        .resizable(SizeHint::FIXED);

    let quark = Quark::new(config)?;
    quark.run();
    Ok(())
}
