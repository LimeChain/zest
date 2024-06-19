#![allow(unused)]

use std::{
    collections::HashMap,
    env, fs, io,
    path::{Path, PathBuf},
    process::{Command, ExitStatus},
    time::SystemTime,
};

use chrono::Local;
use clap::Parser;
use eyre::{bail, Context, OptionExt, Result};

mod from_grcov;
mod util;

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    #[arg(long, default_value = ".", help = "Path to the solana project")]
    path: PathBuf,

    #[arg(
        long,
        // default_value = "stable",
        help = "Version of the compiler to use. Nightly required for branch coverage",
    )]
    compiler_version: Option<String>,

    #[arg(
        long,
        default_value_t = false,
        help = "Whether to enable branch coverage (nightly compiler required)"
    )]
    branch: bool,
}

fn main() -> Result<()> {
    let Cli {
        path,
        compiler_version,
        branch,
    } = Cli::try_parse()?;

    // Check the conditions after parsing
    let is_nightly: bool = compiler_version
        .as_ref()
        .map(|v| !v.contains("nightly"))
        .unwrap_or(true);
    if branch && is_nightly {
        eprintln!("Error: The --branch option requires the compiler_version to be 'nightly'.");
        std::process::exit(1);
    }

    env::set_current_dir(path.clone())?;

    let target_dir = "./target";
    let coverage_dir = "./target/coverage";

    // NOTE: prepare coverage_dir
    {
        let res = fs::create_dir_all(coverage_dir);

        match res {
            Ok(_) => {}
            Err(err) if err.kind() == io::ErrorKind::AlreadyExists => {
                eprintln!("{} already exists, attempting to remove", coverage_dir);
                util::remove_contents(coverage_dir)?;
            }
            Err(err) => {
                bail!("Could not create {}: {}", coverage_dir, err);
            }
        }

        // Clean old `profraw` files
        fs::read_dir(coverage_dir)?
            .collect::<Result<Vec<_>, _>>()?
            .iter()
            .filter_map(|entry| {
                let path = entry.path();
                let ext = path.extension()?;
                util::to_option(ext == "profraw", path)
            })
            .try_for_each(fs::remove_file)?;
    }

    env::set_var(
        "LLVM_PROFILE_FILE",
        format!(
            "{}/solcov-%p-%m.profraw",
            PathBuf::from(coverage_dir).canonicalize()?.display()
        ),
    );

    {
        // TODO: inherit old `${RUSTFLAGS}`
        let mut rustflags = "-C instrument-coverage".to_string();

        if branch {
            rustflags.push_str(" -Z coverage-options=mcdc");
        }

        env::set_var("RUSTFLAGS", rustflags);
    }

    // Build
    {
        let mut cmd = Command::new("cargo")
            .args(compiler_version.clone().map(|v| format!("+{}", v)))
            .args(["build", "--target-dir", target_dir])
            .envs(HashMap::from([
                ("CARGO_INCREMENTAL", "0"),
                ("RUST_BACKTRACE", "1"),
                ("RUST_MIN_STACK", "8388608"),
            ]))
            .spawn()?;

        let output = cmd.wait_with_output()?;
        if !output.status.success() {
            bail!(
                "`cargo build` failed: {}",
                std::str::from_utf8(&output.stderr)?
            );
        }
    }

    // Test
    let before_tests_time = SystemTime::now();
    {
        let mut cmd = Command::new("cargo")
            .args(compiler_version.map(|v| format!("+{}", v)))
            .args(["test", "--target-dir", target_dir])
            .envs(HashMap::from([
                ("CARGO_INCREMENTAL", "0"),
                ("RUST_BACKTRACE", "1"),
                ("RUST_MIN_STACK", "8388608"),
            ]))
            .spawn()?;

        let output = cmd.wait_with_output()?;
        if !output.status.success() {
            bail!(
                "`cargo test` failed: {}",
                std::str::from_utf8(&output.stderr)?
            );
        }
    }
    let after_tests_time = SystemTime::now();

    // TODO: `{before,after}-test` shenanigans/optimizations

    // NOTE: run grcov
    {
        // NOTE: for filtering out "irrelevant" lines, i.e. only leaving the contract code
        let re = regex::Regex::new(
            r#"(?x)
            ^\#\[(program|account)\]$      # Matches #[program] or #[account]
            |
            ^\#\[derive\(\s*[^\)]+\s*\)\]$ # Matches #[derive(Trait, ...)]
            |
            ^declare_id!\(\s*.*\s*\);$     # Matches declare_id!(...)
            |
            ^\s*$                          # Matches "empty" lines
            |
            ^\s*[\(\)\[\]\{\}]*\s*$        # Matches lines with only brackets
        "#,
        )?;

        // NOTE: extrapolated from running the original `grcov` CLI with appropriate arguments
        let opt = from_grcov::Opt {
            paths: vec![coverage_dir.to_string()],
            binary_path: Some(target_dir.into()),
            llvm_path: None,
            output_types: vec![from_grcov::OutputType::Html],
            output_path: Some(coverage_dir.into()),
            output_config_file: None,
            source_dir: Some(".".into()),
            prefix_dir: None,
            ignore_not_existing: true,
            ignore_dir: vec!["tests".to_string(), "target".to_string()], // format!("{}/tests", path.display())],
            keep_dir: vec![],
            path_mapping: None,
            branch,
            filter: Some(from_grcov::Filter::Covered),
            sort_output_types: vec![from_grcov::OutputType::Html],
            llvm: true,
            token: None,
            commit_sha: None,
            service_name: None,
            service_number: None,
            service_job_id: None,
            service_pull_request: None,
            service_flag_name: None,
            parallel: false,
            threads: None,
            precision: 2,
            guess_directory: false,
            vcs_branch: "master".to_string(),
            log: PathBuf::from("stderr"),
            log_level: from_grcov::LevelFilterArg(log::LevelFilter::Error),
            excl_line: Some(re),
            excl_start: None,
            excl_stop: None,
            excl_br_line: None,
            excl_br_start: None,
            excl_br_stop: None,
            no_demangle: false,
        };

        from_grcov::main(opt);
    }

    // NOTE: experimentation with `tarpaulin` as a backend
    // {
    //     // `cargo tarpaulin --skip-clean --out Html --engine Llvm --output-dir target/coverage`
    //     let mut config: tarpaulin::config::Config = Default::default();
    //     config.command = tarpaulin::config::Mode::Test;
    //     config.set_clean(false);
    //     config.generate = vec![tarpaulin::config::OutputFile::Html];
    //     config.set_engine(tarpaulin::config::TraceEngine::Llvm);
    //     config.branch_coverage = branch;
    //     config.output_directory = Some("./target/coverage/".into());
    //     config.test_timeout = std::time::Duration::from_secs(120);
    //     config.exclude_path(&PathBuf::from("*tests*"));
    //
    //     tarpaulin::run(&[config]);
    // }

    // NOTE: run grcov (old way, through CLI)
    // {
    //     // grcov program/coverage --llvm -t html -o program/coverage
    //     let mut cmd = Command::new("grcov")
    //         .args([program_DIR, "--llvm", "-t", "html", "-o", program_DIR])
    //         .spawn()?;
    //
    //     cmd.wait()?;
    // }

    // NOTE: open resulting report
    eprintln!("Successfully generated report, opening...");
    // open::that("./target/coverage/tarpaulin-report.html")?;
    open::that("./target/coverage/html/index.html")?;

    Ok(())
}
