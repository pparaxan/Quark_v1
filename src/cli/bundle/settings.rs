// This codebase is a frking mess.
use super::{category::AppCategory, common::print_warning};
use cargo_metadata::{Metadata, MetadataCommand};
use error_chain::bail;
use serde::Deserialize;
use serde_json::Value;
use std::borrow::Cow;
//use std::collections::HashMap;
//use std::ffi::OsString;
use std::fmt::Display;
use std::path::{Path, PathBuf};
use target_build_utils::TargetInfo;
use std::ffi::OsString;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PackageType {
    OsxBundle,
    WindowsMsi,
    Deb,
} // We got OsxBundle, **WindowsMsi** & Deb... what?

impl PackageType {
    pub fn from_short_name(name: &str) -> Option<PackageType> {
        // Other types we may eventually want to support: apk
        match name {
            "deb" => Some(PackageType::Deb),
            "msi" => Some(PackageType::WindowsMsi),
            "osx" => Some(PackageType::OsxBundle),
            _ => None,
        }
    }

    pub fn short_name(&self) -> &'static str {
        match *self {
            PackageType::Deb => "deb",
            PackageType::WindowsMsi => "msi",
            PackageType::OsxBundle => "osx",
        }
    }

    pub fn all() -> &'static [PackageType] {
        ALL_PACKAGE_TYPES
    } // remove this maybe?
}

const ALL_PACKAGE_TYPES: &[PackageType] = &[
    PackageType::Deb,
    PackageType::WindowsMsi,
    PackageType::OsxBundle,
];

#[derive(Clone, Debug)]
pub enum BuildArtifact {
    Main,
    Bin(String),
}

#[derive(Clone, Debug, Default, Deserialize)]
struct BundleSettings {
    // General settings:
    name: Option<String>,
    identifier: Option<String>,
    icon: Option<Vec<String>>,
    version: Option<String>,
    resources: Option<Vec<String>>,
    copyright: Option<String>,
    category: Option<AppCategory>,
    short_description: Option<String>,
    long_description: Option<String>,
    // OS-specific settings:
    linux_mime_types: Option<Vec<String>>,
    linux_exec_args: Option<String>,
    linux_use_terminal: Option<bool>,
    deb_depends: Option<Vec<String>>,
    osx_frameworks: Option<Vec<String>>,
    osx_minimum_system_version: Option<String>,
    osx_url_schemes: Option<Vec<String>>,
    // Bundles for other binaries/examples:
    // bin: Option<HashMap<String, BundleSettings>>, // never used, gg- there's prob no need for this either way.
    // example: Option<HashMap<String, BundleSettings>>, // removed support for examples
}

#[derive(Clone, Debug)]
pub struct Settings {
    package: cargo_metadata::Package,
    package_type: Option<PackageType>, // If `None`, use the default package type for this os
    target: Option<(String, TargetInfo)>,
    features: Option<String>,
    project_out_directory: PathBuf,
    build_artifact: BuildArtifact,
    profile: String,
    binary_path: PathBuf,
    binary_name: String,
    bundle_settings: BundleSettings,

}

/// Try to load `Cargo.toml` file in the specified directory
fn load_metadata(dir: &Path) -> crate::cli::bundle::Result<Metadata> {
    let cargo_file_path = dir.join("Cargo.toml");
    Ok(MetadataCommand::new()
        .manifest_path(cargo_file_path)
        .exec()?)
}

impl Settings {
    pub fn new(current_dir: PathBuf) -> crate::cli::bundle::Result<Self> {
        // Build the project first
        let status = std::process::Command::new("cargo")
            .args(["build", "--profile", "release", "--quiet"])
            .status()?; // sometimes the stupid code works better than the good one

        if !status.success() {
            bail!("Failed to build project in the release profile");
        }

        let profile = "temp".to_string();
        let target = None;
        let package_type = None;
        let features = None;

        let current_exe = std::env::current_exe()?;
        let binary_name = current_exe
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| "Could not determine binary name")?
            .to_string();

        let build_artifact = BuildArtifact::Main;

        let (bundle_settings, package) = Settings::find_bundle_package(load_metadata(&current_dir)?)?;
        // let workspace_dir = Settings::get_workspace_dir(current_dir);
        // let target_dir = Settings::get_target_dir(&workspace_dir, &target, &profile, &build_artifact);
        // let target_dir = Settings::get_target_dir(&workspace_dir, &target, &profile);
        let target_dir = Settings::get_target_dir(&current_dir, &target, &profile);

        // let binary_extension = match package_type {
        //     Some(x) => match x {
        //         PackageType::OsxBundle | PackageType::Deb => "",
        //         PackageType::WindowsMsi => ".exe",
        //     },
        //     None => if cfg!(windows) { ".exe" } else { "" },
        // };

        let binary_path = target_dir.join(&binary_name);

        Ok(Settings {
            package,
            package_type,
            target,
            features,
            project_out_directory: target_dir, // Odd one out
            build_artifact,
            profile,
            binary_path,
            binary_name,
            bundle_settings,
        })
    }

    fn get_target_dir(
        project_dir: &Path,
        target: &Option<(String, TargetInfo)>,
        profile: &str,
    ) -> PathBuf {
        let mut cargo = std::process::Command::new(
            std::env::var_os("CARGO").unwrap_or_else(|| OsString::from("cargo")),
        );
        cargo.args(["metadata", "--no-deps"]);

        let target_dir = cargo.output().ok().and_then(|output| {
            let json_string = String::from_utf8(output.stdout).ok()?;
            let json: Value = serde_json::from_str(&json_string).ok()?;
            Some(PathBuf::from(json.get("target_directory")?.as_str()?))
        });

        let mut path = target_dir.unwrap_or(project_dir.join("target"));

        if let &Some((ref triple, _)) = target {
            path.push(triple);
        }

        path.push(profile);
        path.into()
    }

    /*
        The specification of the Cargo.toml Manifest that covers the "workspace" section is here:
        https://doc.rust-lang.org/cargo/reference/manifest.html#the-workspace-section

        Determining if the current project folder is part of a workspace:
            - Walk up the file system, looking for a Cargo.toml file.
            - Stop at the first one found.
            - If one is found before reaching "/" then this folder belongs to that parent workspace
    */

    fn get_workspace_dir(metadata: Metadata) -> PathBuf {
        metadata.workspace_root.clone().into()
    }

    fn find_bundle_package(
        metadata: Metadata,
    ) -> crate::cli::bundle::Result<(BundleSettings, cargo_metadata::Package)> {
        for package_id in metadata.workspace_members.iter() {
            let package = &metadata[package_id];
            if let Some(bundle) = package.metadata.get("bundle") {
                let settings = serde_json::from_value::<BundleSettings>(bundle.clone())?; // DESERIALIZEEEEEEEEEE
                return Ok((settings, package.clone()));
            }
        }
        print_warning("No package in workspace has [package.metadata.bundle] section")?;
        if let Some(root_package) = metadata.root_package() {
            Ok((BundleSettings::default(), root_package.clone()))
        } else {
            bail!("unable to find root package")
        }
    }

    /// Returns the directory where the bundle should be placed.
    pub fn project_out_directory(&self) -> &Path {
        &self.project_out_directory
    }

    /// Returns the architecture for the binary being bundled (e.g. "arm" or
    /// "x86" or "x86_64").
    pub fn binary_arch(&self) -> &str {
        // if let Some((_, ref info)) = self.target {
        //     info.target_arch()
        // } else {
            std::env::consts::ARCH
        // }
    }

    /// Returns the file name of the binary being bundled.
    pub fn binary_name(&self) -> &str {
        &self.binary_name
    }

    /// Returns the path to the binary being bundled.
    pub fn binary_path(&self) -> &Path {
        &self.binary_path
    }

    /// If a specific package type was specified by the command-line, returns
    /// that package type; otherwise, if a target triple was specified by the
    /// command-line, returns the native package type(s) for that target;
    /// otherwise, returns the native package type(s) for the host platform.
    /// Fails if the host/target's native package type is not supported.
    pub fn package_types(&self) -> crate::cli::bundle::Result<Vec<PackageType>> {
        if let Some(package_type) = self.package_type {
            Ok(vec![package_type])
        } else {
            let target_os = if let Some((_, ref info)) = self.target {
                info.target_os()
            } else {
                std::env::consts::OS
            };
            match target_os {
                "macos" => Ok(vec![PackageType::OsxBundle]),
                "linux" => Ok(vec![PackageType::Deb]),
                "windows" => Ok(vec![PackageType::WindowsMsi]),
                os => bail!("Native {} bundles not yet supported.", os),
            }
        }
    }

    /// If the bundle is being cross-compiled, returns the target triple string
    /// (e.g. `"x86_64-apple-darwin"`).  If the bundle is targeting the host
    /// environment, returns `None`.
    pub fn target_triple(&self) -> Option<&str> {
        match self.target {
            Some((ref triple, _)) => Some(triple.as_str()),
            None => None,
        }
    }

    pub fn features(&self) -> Option<&str> {
        match self.features {
            Some(ref features) => Some(features.as_str()),
            None => None,
        }
    }

    /// Returns the artifact that is being bundled.
    pub fn build_artifact(&self) -> &BuildArtifact {
        &self.build_artifact
    }

    /// Returns true if the bundle is being compiled in release mode, false if
    /// it's being compiled in debug mode.
    pub fn build_profile(&self) -> &str {
        &self.profile
    }

    pub fn bundle_name(&self) -> &str {
        self.bundle_settings
            .name
            .as_ref()
            .unwrap_or(&self.package.name)
    }

    pub fn bundle_identifier(&self) -> Cow<'_, str> {
        if let Some(identifier) = &self.bundle_settings.identifier {
            identifier.into()
        } else {
            match &self.build_artifact {
                BuildArtifact::Main => "".into(),
                BuildArtifact::Bin(name) => format!("{name}.{}", self.package.name).into(),
            }
        }
    }

    /// Returns an iterator over the icon files to be used for this bundle.
    pub fn icon_files(&self) -> ResourcePaths {
        match self.bundle_settings.icon {
            Some(ref paths) => ResourcePaths::new(paths.as_slice(), false),
            None => ResourcePaths::new(&[], false),
        }
    }

    /// Returns an iterator over the resource files to be included in this
    /// bundle.
    pub fn resource_files(&self) -> ResourcePaths {
        match self.bundle_settings.resources {
            Some(ref paths) => ResourcePaths::new(paths.as_slice(), true),
            None => ResourcePaths::new(&[], true),
        }
    }

    pub fn version_string(&self) -> &dyn Display {
        match self.bundle_settings.version.as_ref() {
            Some(v) => v,
            None => &self.package.version,
        }
    }

    pub fn copyright_string(&self) -> Option<&str> {
        self.bundle_settings.copyright.as_deref()
    }

    pub fn author_names(&self) -> &[String] {
        &self.package.authors
    }

    pub fn authors_comma_separated(&self) -> Option<String> {
        let names = self.author_names();
        if names.is_empty() {
            None
        } else {
            Some(names.join(", "))
        }
    }

    pub fn homepage_url(&self) -> &str {
        self.package.homepage.as_deref().unwrap_or("")
    }

    pub fn app_category(&self) -> Option<AppCategory> {
        self.bundle_settings.category
    }

    pub fn short_description(&self) -> &str {
        self.bundle_settings
            .short_description
            .as_deref()
            .unwrap_or_else(|| self.package.description.as_deref().unwrap_or(""))
    }

    pub fn long_description(&self) -> Option<&str> {
        self.bundle_settings.long_description.as_deref()
    }

    pub fn debian_dependencies(&self) -> &[String] {
        match self.bundle_settings.deb_depends {
            Some(ref dependencies) => dependencies.as_slice(),
            None => &[],
        }
    }

    pub fn linux_mime_types(&self) -> &[String] {
        match self.bundle_settings.linux_mime_types {
            Some(ref mime_types) => mime_types.as_slice(),
            None => &[],
        }
    }

    pub fn linux_use_terminal(&self) -> Option<bool> {
        self.bundle_settings.linux_use_terminal
    }

    pub fn linux_exec_args(&self) -> Option<&str> {
        self.bundle_settings.linux_exec_args.as_deref()
    }

    pub fn osx_frameworks(&self) -> &[String] {
        match self.bundle_settings.osx_frameworks {
            Some(ref frameworks) => frameworks.as_slice(),
            None => &[],
        }
    }

    pub fn osx_minimum_system_version(&self) -> Option<&str> {
        self.bundle_settings.osx_minimum_system_version.as_deref()
    }

    pub fn osx_url_schemes(&self) -> &[String] {
        match self.bundle_settings.osx_url_schemes {
            Some(ref urlosx_url_schemes) => urlosx_url_schemes.as_slice(),
            None => &[],
        }
    }
}

// fn bundle_settings_from_table(
//     opt_map: &Option<HashMap<String, BundleSettings>>,
//     map_name: &str,
//     bundle_name: &str,
// ) -> crate::cli::bundle::Result<BundleSettings> {
//     if let Some(bundle_settings) = opt_map.as_ref().and_then(|map| map.get(bundle_name)) {
//         Ok(bundle_settings.clone())
//     } else {
//         print_warning(&format!(
//             "No [package.metadata.bundle.{}.{}] section in Cargo.toml",
//             map_name, bundle_name
//         ))?;
//         Ok(BundleSettings::default())
//     }
// }

pub struct ResourcePaths<'a> {
    pattern_iter: std::slice::Iter<'a, String>,
    glob_iter: Option<glob::Paths>,
    walk_iter: Option<walkdir::IntoIter>,
    allow_walk: bool,
}

impl<'a> ResourcePaths<'a> {
    fn new(patterns: &'a [String], allow_walk: bool) -> ResourcePaths<'a> {
        ResourcePaths {
            pattern_iter: patterns.iter(),
            glob_iter: None,
            walk_iter: None,
            allow_walk,
        }
    }
}

impl<'a> Iterator for ResourcePaths<'a> {
    type Item = crate::cli::bundle::Result<PathBuf>;

    fn next(&mut self) -> Option<crate::cli::bundle::Result<PathBuf>> {
        loop {
            if let Some(ref mut walk_entries) = self.walk_iter {
                if let Some(entry) = walk_entries.next() {
                    let entry = match entry {
                        Ok(entry) => entry,
                        Err(error) => return Some(Err(crate::cli::bundle::Error::from(error))),
                    };
                    let path = entry.path();
                    if path.is_dir() {
                        continue;
                    }
                    return Some(Ok(path.to_path_buf()));
                }
            }
            self.walk_iter = None;
            if let Some(ref mut glob_paths) = self.glob_iter {
                if let Some(glob_result) = glob_paths.next() {
                    let path = match glob_result {
                        Ok(path) => path,
                        Err(error) => return Some(Err(crate::cli::bundle::Error::from(error))),
                    };
                    if path.is_dir() {
                        if self.allow_walk {
                            let walk = walkdir::WalkDir::new(path);
                            self.walk_iter = Some(walk.into_iter());
                            continue;
                        } else {
                            let msg = format!("{path:?} is a directory");
                            return Some(Err(crate::cli::bundle::Error::from(msg)));
                        }
                    }
                    return Some(Ok(path));
                }
            }
            self.glob_iter = None;
            if let Some(pattern) = self.pattern_iter.next() {
                let glob = match glob::glob(pattern) {
                    Ok(glob) => glob,
                    Err(error) => return Some(Err(crate::cli::bundle::Error::from(error))),
                };
                self.glob_iter = Some(glob);
                continue;
            }
            return None;
        }
    }
}
