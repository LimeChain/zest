use eyre::{Context, Result};
use std::{env, fs, os::unix, path::Path, process::Command};

#[rustfmt::skip]
pub fn remove_contents(path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();

    if path.is_dir() {
        fs::remove_dir_all(path)
            .with_context(|| format!(
                "Error removing directory {}",
                path.display(),
            ))?;
    } else {
        fs::remove_file(path)
            .with_context(|| format!(
                "Error removing file {}",
                path.display(),
            ))?;
    }

    Ok(())
}

pub fn is_newer_than(file: &Path, reference: &Path) -> Result<bool> {
    let file_time = fs::metadata(file)?.modified()?;
    let reference_time = fs::metadata(reference)?.modified()?;
    Ok(file_time > reference_time)
}

pub fn symlink(target: impl AsRef<Path>, link: impl AsRef<Path>) -> Result<()> {
    let (target, link) = (target.as_ref(), link.as_ref());

    if link.exists() {
        fs::remove_file(link)?;
    }

    unix::fs::symlink(target, link)
        .with_context(|| format!("Error creating symbolic link for {}", target.display()))
}

pub fn to_option<A>(predicate: bool, value: A) -> Option<A> {
    if predicate {
        Some(value)
    } else {
        None
    }
}

pub fn is_rustup_managed() -> bool {
    // Check if the `RUSTUP_HOME` or `CARGO_HOME` environment variables are set
    if env::var("RUSTUP_HOME").is_ok() || env::var("CARGO_HOME").is_ok() {
        return true;
    }

    // Check if the `rustup` command is available
    if let Ok(output) = Command::new("rustup").arg("--version").output() {
        if output.status.success() {
            return true;
        }
    }

    false
}
