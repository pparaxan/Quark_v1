use error_chain::error_chain;

mod category;
mod common;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
mod settings;
#[cfg(target_os = "windows")]
mod windows;

pub use self::common::{print_error, print_finished};
pub use self::settings::{BuildArtifact, PackageType, Settings};
use std::path::PathBuf;
#[cfg(target_os = "linux")]
use super::bundle::linux::deb_bundle;

// #[allow(!(unexpected_cfgs))] // fix this
error_chain! { // remove this dep? it's using an outdated version of `bitflags`; could fork but eh.
    foreign_links {
        Glob(glob::GlobError);
        GlobPattern(glob::PatternError);
        Io(std::io::Error);
        Image(image::ImageError);
        Json(serde_json::Error);
        Metadata(cargo_metadata::Error);
        // Target(target_build_utils::Error);
        Toml(toml::de::Error);
        Walkdir(walkdir::Error);
    }
}

pub fn bundle_project(settings: Settings) -> self::Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    for package_type in settings.package_types()? {
        paths.append(&mut match package_type {
            #[cfg(target_os = "macos")]
            PackageType::OsxBundle => macos::bundle_project(&settings)?,
            #[cfg(target_os = "windows")]
            PackageType::WindowsMsi => windows::bundle_project(&settings)?,
            #[cfg(target_os = "linux")]
            PackageType::Deb => deb_bundle::bundle_project(&settings)?,
            _ => Vec::new(),
        });
    }
    Ok(paths)
}
