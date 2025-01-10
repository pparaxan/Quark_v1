pub mod build_http;
pub mod build_static;
#[cfg(feature = "bundle")]
pub mod bundle;

use include_dir::{include_dir, Dir};

static QUARKFOLDER: Dir = include_dir!("$CARGO_MANIFEST_DIR/src_quark");

#[derive(Debug, Default)]
pub struct Args {
    pub live: bool,
    pub bundle: bool,
}

pub fn parse_args() -> Args {
    // https://github.com/WilliamAnimate/catgirls_anytime/blob/849c973e8e355cb6ae0695e287764299c6c2543d/src/lib.rs#L18-L76
    let args: Vec<String> = std::env::args().collect();

    let mut parsed_args = Args::default();

    for args in &args[1..] {
        match args.as_str() {
            "--help" => {
                println!("Usage: cargo run -- [OPTION]");
                println!("--live          Start a live server with hot reload support.");
                println!("--bundle        Package your Quark application for your target.\n                You need the `bundle` feature enable.");
                println!("--help          Display this help message and exit.");
                std::process::exit(0);
            }
            "--live" => {
                parsed_args.live = true;
            }
            #[cfg(feature = "bundle")]
            "--bundle" => {
                parsed_args.bundle = true;
                use std::path::PathBuf;
                fn bundle_executable() -> self::bundle::Result<Vec<PathBuf>> {
                    let current_dir = std::env::current_dir()?;
                    let settings = self::bundle::Settings::new(current_dir)?;
                    let bundle_paths = self::bundle::bundle_project(settings)?;

                    Ok(bundle_paths)
                }
                match bundle_executable() {
                    Ok(paths) => {
                        println!("Successfully bundled application to:");
                        for path in paths {
                            println!("  {}", path.display());
                        }
                    }
                    Err(e) => {
                        let _ = bundle::print_error(&e);
                        std::process::exit(1);
                    }
                }
                std::process::exit(0);
            }
            other => {
                eprintln!(
                    "'{other}' is an unknown argument silly. Use '--help' to list the commands."
                );
                std::process::exit(1);
            }
        }
    }
    parsed_args
}
