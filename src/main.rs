#![allow(unused)]

use std::{
    collections::HashMap,
    env, fs, io,
    path::{Path, PathBuf},
    process::{Command, ExitStatus, Stdio},
    time::SystemTime,
};

use chrono::Local;
use clap::{Parser, ValueEnum};
use eyre::{bail, Context, OptionExt, Result};

mod from_grcov;
mod util;

#[derive(Debug, Parser)]
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

    #[arg(
        long,
        value_enum,
        default_value_t = CoverageStrategy::InstrumentCoverage,
        help = "Coverage strategy to use",
    )]
    coverage_strategy: CoverageStrategy,

    // TODO: `-- --exact`?
    #[arg(long, help = "Which tests to run (same as `cargo test`)")]
    tests: Option<String>,

    #[arg(long, value_enum, default_value_t = OutputType::Html, help = "Output type of coverage")]
    output_type: OutputType,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum OutputType {
    Html,
    Lcov,
}

impl From<OutputType> for from_grcov::OutputType {
    fn from(value: OutputType) -> Self {
        match value {
            OutputType::Html => Self::Html,
            OutputType::Lcov => Self::Lcov,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum CoverageStrategy {
    InstrumentCoverage,
    // BUG: does not currently work
    ZProfile,
}

fn main() -> Result<()> {
    let Cli {
        path,
        compiler_version,
        branch,
        coverage_strategy,
        tests,
        output_type,
    } = Cli::try_parse()?;

    // Check the conditions after parsing
    let is_nightly: bool = compiler_version
        .as_ref()
        .map(|v| v.contains("nightly"))
        .unwrap_or(true);

    if compiler_version.is_some() && !util::is_rustup_managed() {
        eprintln!("Error: specifying the `compiler_version` requires usage of a `rustup`-managed Rust installation");
        std::process::exit(1);
    }
    if matches!(coverage_strategy, CoverageStrategy::ZProfile) && !is_nightly {
        eprintln!(
            "Error: The `-Z profile` strategy requires the `compiler_version` to be 'nightly'"
        );
        std::process::exit(1);
    }
    if branch && !is_nightly {
        eprintln!("Error: The --branch option requires the `compiler_version` to be 'nightly'.");
        std::process::exit(1);
    }

    env::set_current_dir(&path)
        .with_context(|| format!("Could not `cd` to `{}`", path.display()))?;

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
        // TODO: test if works
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

    // TODO: only for `CoverageStrategy::InstrumentCoverage`
    env::set_var(
        "LLVM_PROFILE_FILE",
        format!(
            "{}/solcov-%p-%m.profraw",
            PathBuf::from(coverage_dir).canonicalize()?.display()
        ),
    );

    // NOTE: set compiler env vars
    {
        // TODO: inherit old `${RUSTFLAGS}`
        match coverage_strategy {
            CoverageStrategy::InstrumentCoverage => {
                let mut rustflags = "-C instrument-coverage".to_string();

                if branch {
                    rustflags.push_str(" -Z coverage-options=mcdc");
                }

                env::set_var("RUSTFLAGS", rustflags);
            }
            CoverageStrategy::ZProfile => {
                // NOTE: "can't instrument with gcov profiling when compiling incrementally"
                env::set_var("CARGO_INCREMENTAL", "0");
                env::set_var("RUSTFLAGS", "-Z profile");
            }
        }

        env::set_var("RUST_BACKTRACE", "1");
        env::set_var("RUST_MIN_STACK", "8388608");
    }

    // Build
    {
        let mut cmd = Command::new("cargo")
            .args(compiler_version.as_ref().map(|v| format!("+{}", v)))
            .args(["build", "--tests", "--target-dir", target_dir])
            .stderr(Stdio::piped())
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
            .args(compiler_version.as_ref().map(|v| format!("+{}", v)))
            .arg("test")
            // NOTE: no filter is passed if `None`
            .args(tests)
            .args(["--target-dir", target_dir])
            .stderr(Stdio::piped())
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
            ^\s*\#\[(program|account)\]$      # Matches #[program] or #[account]
            |
            ^\s*\#\[(tokio::)?test\]$         # Matches #[test] or #[tokio::test]
            |
            ^\s*\#\[derive\(\s*[^\)]+\s*\)\]$ # Matches #[derive(Trait, ...)]
            |
            ^\s*declare_id!\(\s*.*\s*\);$     # Matches declare_id!(...)
            # |
            # ^\s*$                          # Matches "empty" lines
            # |
            # ^\s*[\(\)\[\]\{\}]*\s*$        # Matches lines with only brackets
        "#,
        )?;

        // NOTE: extrapolated from running the original `grcov` CLI with appropriate arguments
        let opt = from_grcov::Opt {
            paths: vec![coverage_dir.to_string()],
            binary_path: Some(target_dir.into()),
            llvm_path: None,
            output_types: vec![output_type.into()],
            output_path: Some(coverage_dir.into()),
            output_config_file: None,
            // NOTE: `.` because we `cd`-ed into the correct directory already
            source_dir: Some(".".into()),
            prefix_dir: None,
            ignore_not_existing: true,
            // NOTE: parsed as globs, see [globset::Globset]
            ignore_dir: vec!["target/*".to_string()],
            keep_dir: vec![],
            path_mapping: None,
            branch,
            filter: None,
            // NOTE: only sorting for `Html`, `LCov` users can sort themselves
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
    //       `branch_coverage` is stubbed, not useful to us
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
    //       depends on CLI, tradeoffs
    // {
    //     // grcov program/coverage --llvm -t html -o target/coverage
    //     let mut cmd = Command::new("grcov")
    //         .args([program_DIR, "--llvm", "-t", "html", "-o", program_DIR])
    //         .spawn()?;
    //
    //     cmd.wait()?;
    // }

    // NOTE: open resulting report
    match output_type {
        OutputType::Html => {
            eprintln!("Successfully generated html report at {}/target/coverage/html/index.html, opening...", path.display());
            // open::that("./target/coverage/tarpaulin-report.html")?;
            open::that("./target/coverage/html/index.html")?;
        }
        OutputType::Lcov => {
            eprintln!("Successfully generated lcov report, you can find it at {}/target/coverage/lcov", path.display());
        }
    }

    Ok(())
}
