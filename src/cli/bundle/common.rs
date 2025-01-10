use super::ResultExt;

use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::{self, BufWriter, Write};
use std::path::{Component, Path, PathBuf};

/// Returns true if the path has a filename indicating that it is a high-density
/// "retina" icon. Specifically, returns true if the file stem ends with
/// "@2x" (a convention specified by the [Apple developer docs](
/// https://developer.apple.com/library/mac/documentation/GraphicsAnimation/Conceptual/HighResolutionOSX/Optimizing/Optimizing.html)).
pub fn is_retina<P: AsRef<Path>>(path: P) -> bool { // Linux code also uses this
    path.as_ref()
        .file_stem()
        .and_then(OsStr::to_str)
        .map(|stem| stem.ends_with("@2x"))
        .unwrap_or(false)
}

/// Creates a new file at the given path, creating any parent directories as
/// needed.
pub fn create_file(path: &Path) -> super::Result<BufWriter<File>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .chain_err(|| format!("Failed to create directory {parent:?}"))?;
    }
    let file = File::create(path).chain_err(|| format!("Failed to create file {path:?}"))?;
    Ok(BufWriter::new(file))
}

fn symlink_dir(src: &Path, dst: &Path) -> io::Result<()> {
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(src, dst)
    }

    #[cfg(windows)]
    {
        std::os::windows::fs::symlink_dir(src, dst)
    }
}

fn symlink_file(src: &Path, dst: &Path) -> io::Result<()> {
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(src, dst)
    }

    #[cfg(windows)]
    {
        std::os::windows::fs::symlink_file(src, dst)
    }
}

/// Copies a regular file from one path to another, creating any parent
/// directories of the destination path as necessary.  Fails if the source path
/// is a directory or doesn't exist.
pub fn copy_file(from: &Path, to: &Path) -> super::Result<()> {
    if !from.exists() {
        eprintln!("{:?} does not exist", from);
    }
    if !from.is_file() {
        eprintln!("{:?} is not a file", from);
    }
    let dest_dir = to.parent().unwrap();
    fs::create_dir_all(dest_dir).chain_err(|| format!("Failed to create {dest_dir:?}"))?;
    fs::copy(from, to).chain_err(|| format!("Failed to copy {from:?} to {to:?}"))?;
    Ok(())
}

/// Recursively copies a directory file from one path to another, creating any
/// parent directories of the destination path as necessary.  Fails if the
/// source path is not a directory or doesn't exist, or if the destination path
/// already exists.
pub fn copy_dir(from: &Path, to: &Path) -> super::Result<()> {
    if !from.exists() {
        eprintln!("{:?} does not exist", from);
    }
    if !from.is_dir() {
        eprintln!("{:?} is not a directory", from);
    }
    if to.exists() {
        eprintln!("{:?} already exists", to);
    }
    let parent = to.parent().unwrap();
    fs::create_dir_all(parent).chain_err(|| format!("Failed to create {parent:?}"))?;
    for entry in walkdir::WalkDir::new(from) {
        let entry = entry?;
        debug_assert!(entry.path().starts_with(from));
        let rel_path = entry.path().strip_prefix(from).unwrap();
        let dest_path = to.join(rel_path);
        if entry.file_type().is_symlink() {
            let target = fs::read_link(entry.path())?;
            if entry.path().is_dir() {
                symlink_dir(&target, &dest_path)?;
            } else {
                symlink_file(&target, &dest_path)?;
            }
        } else if entry.file_type().is_dir() {
            fs::create_dir_all(dest_path)?;
        } else {
            fs::copy(entry.path(), dest_path)?;
        }
    }
    Ok(())
}

/// Given a path (absolute or relative) to a resource file, returns the
/// relative path from the bundle resources directory where that resource
/// should be stored.
pub fn resource_relpath(path: &Path) -> PathBuf {
    let mut dest = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Prefix(_) => {}
            Component::RootDir => dest.push("_root_"),
            Component::CurDir => {}
            Component::ParentDir => dest.push("_up_"),
            Component::Normal(string) => dest.push(string),
        }
    }
    dest
}

/// Prints a message to stderr, in the same format that `cargo` uses,
/// indicating that we are creating a bundle with the given filename.
pub fn print_bundling(filename: &str) -> super::Result<()> {
    print_progress("Bundling", filename)
}

/// Prints a message to stderr, in the same format that `cargo` uses,
/// indicating that we have finished the given bundles.
pub fn print_finished(output_paths: &Vec<PathBuf>) -> super::Result<()> {
    let pluralised = if output_paths.len() == 1 {
        "bundle"
    } else {
        "bundles"
    }; // the fk is this?
    let msg = format!("{} {} at:", output_paths.len(), pluralised);
    print_progress("Finished", &msg)?;
    for path in output_paths {
        println!("        {}", path.display());
    }
    Ok(())
}

fn print_progress(step: &str, msg: &str) -> super::Result<()> {
    let mut output = io::stderr();
    write!(output, "    \x1b[1;32m{step}")?; // Text color is green, bolded
    writeln!(output, " \x1b[0m{msg}")?; // Text color is reverted
    Ok(())
}

/// Prints a warning message to stderr, in the same format that `cargo` uses.
pub fn print_warning(message: &str) -> super::Result<()> {
    let mut output = io::stderr();
    write!(output, "\x1b[1;33mwarning:")?; // Text color is yellow, bolded
    writeln!(output, " \x1b[0m{message}")?; // Text color is reverted
    Ok(())
}

/// Prints an error to stderr, in the same format that `cargo` uses.
pub fn print_error(error: &crate::cli::bundle::Error) -> super::Result<()> {
    let mut output = io::stderr();
    write!(output, "\x1b[1;31merror:")?; // Text color is red, bolded
    writeln!(output, " \x1b[0m{error}")?; // Text color is reverted
    for cause in error.iter().skip(1) {
        writeln!(output, "  Caused by: {cause}")?;
    }
    if let Some(backtrace) = error.backtrace() {
        writeln!(output, "{backtrace:?}")?;
    }
    Ok(())
}
