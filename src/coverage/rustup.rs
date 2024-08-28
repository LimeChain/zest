use std::{
    env,
    process::{Command, Stdio},
};

use eyre::bail;
use spinners::{Spinner, Spinners};

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

pub fn install_llvm_tools(
    compiler_version: impl AsRef<str>
) -> eyre::Result<()> {
    let mut spinner =
        Spinner::new(Spinners::Dots, "Installing toolchain (with `llvm-tools-preview` component)...".to_string());
    let cmd = Command::new("rustup")
        .arg("toolchain")
        .arg("install")
        .arg("--no-self-update")
        .args(["--profile", "minimal"])
        .arg(compiler_version.as_ref())
        .args(["--component", "llvm-tools-preview"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let output = cmd.wait_with_output()?;
    if !output.status.success() {
        spinner.stop_and_persist(
            "❌",
            "Rustup toolchain install failed!".to_string(),
        );
        eprintln!("rustup stdout:");
        eprintln!("{}", std::str::from_utf8(&output.stdout)?);
        eprintln!("rustup build stderr:");
        eprintln!("{}", std::str::from_utf8(&output.stderr)?);
        bail!("`rustup` failed");
    }
    spinner.stop_and_persist("✅", "Toolchain installed!".to_string());

    Ok(())
}
