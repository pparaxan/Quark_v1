//pub mod darwin;
pub mod linux;
pub mod nt;

pub mod category;
pub mod common;
pub mod settings;

use crate::cli::package::linux::deb_bundle;
use crate::package::{BuildArtifact, PackageType, Settings};
use std::{env, path::PathBuf, process, ffi::OsString, result::Result};

pub fn package(settings: Settings) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    for package_type in settings.package_types()? {
        paths.append(&mut match package_type {
            // PackageType::OsxBundle => osx_bundle::bundle_project(&settings)?,
            // PackageType::WindowsMsi => msi_bundle::bundle_project(&settings)?,
            PackageType::Deb => deb_bundle::bundle_project(&settings)?,
            // PackageType::Rpm => rpm_bundle::bundle_project(&settings)?, # No code is here for this, yet.
        });
    }
    Ok(paths)
}
