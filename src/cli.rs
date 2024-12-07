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
                println!("--live          Start a live server with hot reload support");
                println!("--bundle        Package your Quark application for your target [WIP]");
                println!("--help          Display this help message and exit");
                std::process::exit(0);
            }
            "--live" => {
                parsed_args.live = true;
            }
            "--bundle" => {
                println!("Error: This argument isn't complete!");
                std::process::exit(1);
            }
            other => {
                eprintln!("'{other}' is an unknown argument silly. Use '--help' to list the commands.");
                std::process::exit(1);
            }
        }
    }
    parsed_args
}
