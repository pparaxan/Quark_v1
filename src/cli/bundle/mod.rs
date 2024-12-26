use error_chain::*;
use clap::{App, AppSettings, Arg, SubCommand};
use std::env;
use std::ffi::OsString;
use std::process;

// mod bundle;
mod category;
mod common;
mod linux;
mod windows;
mod macos;
mod settings;

pub use self::common::{print_error, print_finished};
pub use self::settings::{BuildArtifact, PackageType, Settings};
use self::linux::deb_bundle;
use std::path::PathBuf;
use crate::cli::parse_args;

error_chain! {
    foreign_links {
        Glob(glob::GlobError);
        GlobPattern(glob::PatternError);
        Io(std::io::Error);
        Image(image::ImageError);
        Json(serde_json::Error);
        Metadata(cargo_metadata::Error);
        Target(target_build_utils::Error);
        Term(term::Error);
        Toml(toml::de::Error);
        Walkdir(walkdir::Error);
    }
}

/// Ensures the binary file is up-to-date by running `cargo build`.
fn build_project_if_unbuilt(settings: &Settings) -> self::Result<()> {
    if env::var("CARGO_BUNDLE_SKIP_BUILD").is_ok() {
        return Ok(());
    }

    let mut cargo = process::Command::new(env::var_os("CARGO").unwrap_or_else(|| OsString::from("cargo")));
    cargo.arg("build");

    if let Some(triple) = settings.target_triple() {
        cargo.arg(format!("--target={}", triple));
    }
    if let Some(features) = settings.features() {
        cargo.arg(format!("--features={}", features));
    }
    match settings.build_artifact() {
        BuildArtifact::Main => {}
        BuildArtifact::Bin(name) => {
            cargo.arg(format!("--bin={}", name));
        }
        BuildArtifact::Example(name) => {
            cargo.arg(format!("--example={}", name));
        }
    }
    match settings.build_profile() {
        "dev" => {}
        "release" => {
            cargo.arg("--release");
        }
        custom => {
            cargo.arg("--profile");
            cargo.arg(custom);
        }
    }
    if settings.all_features() {
        cargo.arg("--all-features");
    }
    if settings.no_default_features() {
        cargo.arg("--no-default-features");
    }

    let status = cargo.status()?;
    if !status.success() {
        bail!("`cargo build` failed with status: {}", status);
    }

    Ok(())
}

/// Main execution logic for the `cargo-bundle` CLI tool.
fn run() -> self::Result<()> {
    let args = parse_args();

    if args.bundle {
        let all_formats: Vec<&str> = PackageType::all()
            .iter()
            .map(PackageType::short_name)
            .collect();

        let matches = App::new("cargo-bundle")
            .version(format!("v0.0.1").as_str())
            .bin_name("cargo")
            .setting(AppSettings::GlobalVersion)
            .setting(AppSettings::SubcommandRequired)
            .subcommand(
                SubCommand::with_name("bundle")
                    .author("George Burton <burtonageo@gmail.com>")
                    .about("Bundle Rust executables into OS bundles")
                    .setting(AppSettings::DisableVersion)
                    .setting(AppSettings::UnifiedHelpMessage)
                    .arg(
                        Arg::with_name("bin")
                            .long("bin")
                            .value_name("NAME")
                            .help("Bundle the specified binary"),
                    )
                    .arg(
                        Arg::with_name("example")
                            .long("example")
                            .value_name("NAME")
                            .conflicts_with("bin")
                            .help("Bundle the specified example"),
                    )
                    .arg(
                        Arg::with_name("format")
                            .long("format")
                            .value_name("FORMAT")
                            .possible_values(&all_formats)
                            .help("Which bundle format to produce"),
                    )
                    .arg(
                        Arg::with_name("release")
                            .long("release")
                            .help("Build a bundle from a target built in release mode"),
                    )
                    .arg(
                        Arg::with_name("profile")
                            .long("profile")
                            .value_name("NAME")
                            .conflicts_with("release")
                            .help("Build a bundle from a target build using the given profile"),
                    )
                    .arg(
                        Arg::with_name("target")
                            .long("target")
                            .value_name("TRIPLE")
                            .help("Build a bundle for the target triple"),
                    )
                    .arg(
                        Arg::with_name("features")
                            .long("features")
                            .value_name("FEATURES")
                            .help("Set crate features for the bundle. Eg: `--features \"f1 f2\"`"),
                    )
                    .arg(
                        Arg::with_name("all-features")
                            .long("all-features")
                            .help("Build a bundle with all crate features."),
                    )
                    .arg(
                        Arg::with_name("no-default-features")
                            .long("no-default-features")
                            .help("Build a bundle without the default crate features."),
                    ),
            )
            .get_matches();

        if let Some(bundle_matches) = matches.subcommand_matches("bundle") {
            let settings = env::current_dir()
                .map_err(From::from)
                .and_then(|dir| Settings::new(dir, bundle_matches))?;

            build_project_if_unbuilt(&settings)?;
            let output_paths = bundle_project(settings)?;

            self::print_finished(&output_paths)?;
        }
    }
    Ok(())
}

/// Entry point of the application.
fn main() {
    if let Err(error) = run() {
        self::print_error(&error).unwrap();
        process::exit(1);
    }
}

pub fn bundle_project(settings: Settings) -> self::Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    for package_type in settings.package_types()? {
        paths.append(&mut match package_type {
            PackageType::OsxBundle => macos::bundle_project(&settings)?,
            PackageType::WindowsMsi => windows::bundle_project(&settings)?,
            PackageType::Deb => deb_bundle::bundle_project(&settings)?,
        });
    }
    Ok(paths)
}
