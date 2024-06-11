use eyre::{Context, Result};
use std::{fs, os::unix, path::Path};

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
