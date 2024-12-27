use error_chain::*;

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

pub fn bundle_executable() -> Result<Vec<PathBuf>> {
    // Get current working directory
    let current_dir = std::env::current_dir()?;

    // Create bundle settings with release profile
    let settings = self::Settings::new(current_dir)?;

    // Bundle the project
    let bundle_paths = self::bundle_project(settings)?;

    Ok(bundle_paths)
}
