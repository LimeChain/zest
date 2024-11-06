use std::collections::HashMap;
use std::io;
use std::process::{Command, Stdio};
use std::time::SystemTime;
use std::{env, fs, path::PathBuf};

use clap_serde_derive::{
    clap::{self, ArgAction, ValueEnum},
    ClapSerde,
};
use eyre::{bail, Context};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use spinners::{Spinner, Spinners};

use crate::{config_parsing::ConfigFileName, from_grcov, util};

mod rustup;
use rustup::is_rustup_managed;

use self::rustup::install_llvm_tools;

// #[derive(Debug, Clone, PartialEq, Parser, Serialize, Deserialize)]
#[derive(ClapSerde, Debug, Clone)]
pub struct Config {
    #[arg(long, help = "Path to the solana project")]
    #[default(".".into())]
    pub path: PathBuf,

    #[arg(
        long,
        help = "Version of the compiler toolchain to use (overrides project-specific `rust-toolchain.toml`). Nightly required for branch coverage"
    )]
    #[default(None)]
    pub compiler_version: Option<String>,

    #[arg(
        long,
        help = "Whether to enable branch coverage (nightly compiler required)"
    )]
    #[default(false)]
    pub branch: bool,

    #[arg(
        long,
        action(ArgAction::SetTrue),
        help = "Whether to build and test using Solana's `cargo-build-sbf` and `cargo-test-sbf` tools"
    )]
    #[default(false)]
    pub with_sbf: bool,

    #[arg(long, value_enum, help = "Coverage strategy to use")]
    #[default(CoverageStrategy::InstrumentCoverage)]
    pub coverage_strategy: CoverageStrategy,

    // TODO: `-- --exact`?
    #[arg(
        long = "test",
        value_name = "TEST_FILTER",
        help = "Which tests to run (can be stacked) (as per `cargo test`'s spec, see <https://doc.rust-lang.org/book/ch11-02-running-tests.html>)"
    )]
    #[default(vec![])]
    pub tests: Vec<String>,

    #[arg(
        long = "skip",
        value_name = "TEST_FILTER",
        help = "Which tests to skip (same rules as `--test`)"
    )]
    #[default(vec![])]
    pub skips: Vec<String>,

    #[arg(
        long = "output-type",
        value_name = "OUTPUT_TYPE",
        value_enum,
        help = "Output type of coverage (can be stacked)"
    )]
    #[default(vec![OutputType::Html])]
    pub output_types: Vec<OutputType>,

    #[arg(
        long = "contract-style",
        value_name = "CONTRACT_STYLE",
        value_enum,
        help = "Style of contract",
    )]
    #[default(ContractStyle::Anchor)]
    pub contract_style: ContractStyle,
}

impl ConfigFileName for Config {
    const NAME: &'static str = "coverage";
}

#[derive(
    Debug,
    Clone,
    Copy,
    Hash,
    PartialEq,
    Eq,
    Default,
    ValueEnum,
    Serialize,
    Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum ContractStyle {
    #[default]
    Anchor,
    Native,
}

#[derive(
    Debug,
    Clone,
    Copy,
    Hash,
    PartialEq,
    Eq,
    Default,
    ValueEnum,
    Serialize,
    Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum OutputType {
    #[default]
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

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Default,
    ValueEnum,
    Serialize,
    Deserialize,
)]
pub enum CoverageStrategy {
    #[default]
    InstrumentCoverage,
    // BUG: does not currently work
    ZProfile,
}

pub fn run(config: Config) -> eyre::Result<()> {
    let Config {
        path,
        compiler_version,
        branch,
        with_sbf,
        coverage_strategy,
        tests,
        skips,
        output_types,
        contract_style,
    } = config;

    // Check the conditions after parsing
    let is_nightly: bool = compiler_version
        .as_ref()
        .map(|v| v.contains("nightly"))
        .unwrap_or(true);

    if compiler_version.is_some() && !is_rustup_managed() {
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

    // TODO: use `--manifest-path`
    env::set_current_dir(&path)
        .with_context(|| format!("Could not `cd` to `{}`", path.display()))?;

    install_llvm_tools(compiler_version.as_ref())?;

    let target_dir = "./target";
    let coverage_dir = "./target/coverage";

    // NOTE: prepare coverage_dir
    {
        let res = fs::create_dir_all(coverage_dir);

        match res {
            Ok(_) => {}
            Err(err) if err.kind() == io::ErrorKind::AlreadyExists => {
                eprintln!(
                    "{} already exists, attempting to remove",
                    coverage_dir
                );
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

    let mut env_vars: HashMap<&str, String> = HashMap::new();

    // TODO: only for `CoverageStrategy::InstrumentCoverage`
    env_vars.insert(
        "LLVM_PROFILE_FILE",
        format!(
            "{}/zest-%p-%m.profraw",
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

                env_vars.insert("RUSTFLAGS", rustflags);
            }
            CoverageStrategy::ZProfile => {
                // NOTE: "can't instrument with gcov profiling when compiling incrementally"
                env_vars.insert("CARGO_INCREMENTAL", "0".to_string());
                env_vars.insert("RUSTFLAGS", "-Z profile".to_string());
            }
        }

        env_vars.insert("RUST_BACKTRACE", "1".to_string());
        env_vars.insert("RUST_MIN_STACK", "8388608".to_string());
    }

    // Build
    // TODO: limited output
    {
        let mut spinner =
            Spinner::new(Spinners::Dots, "Building the project...".to_string());
        let cmd = Command::new("cargo")
            .args(compiler_version.as_ref().map(|v| format!("+{}", v)))
            .args(if with_sbf { ["build-sbf", "--"].iter() } else { ["build"].iter() })
            // NOTE: force color (for prettier error messages)
            .args(["--color", "always"])
            .args(["--tests", "--target-dir", target_dir])
            .envs(&env_vars)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let output = cmd.wait_with_output()?;
        if !output.status.success() {
            spinner.stop_and_persist("❌", "Build failed!".to_string());
            eprintln!("cargo build stdout:");
            eprintln!("{}", std::str::from_utf8(&output.stdout)?);
            eprintln!("cargo build stderr:");
            eprintln!("{}", std::str::from_utf8(&output.stderr)?);
            bail!("`cargo build` failed");
        }
        spinner.stop_and_persist("✅", "Project built!".to_string());
    }

    // Test
    // TODO: limited output
    let _before_tests_time = SystemTime::now();
    {
        let cargo_test = |tests: Option<&String>| -> eyre::Result<()> {
            let tests_signifier = tests
                .as_ref()
                .map(|test| format!(" ({})", test))
                .unwrap_or_default();

            let mut spinner = Spinner::new(
                Spinners::Dots,
                format!("Running the tests{}...", tests_signifier),
            );

            let cmd = Command::new("cargo")
                .args(compiler_version.as_ref().map(|v| format!("+{}", v)))
                .args(if with_sbf { ["test-sbf", "--"].iter() } else { ["test"].iter() })
                // NOTE: force color (for prettier error messages)
                .args(["--color", "always"])
                // NOTE: no filter is passed if `None`
                .args(tests)
                .args(["--target-dir", target_dir])
                .arg("--")
                .args(skips.iter().flat_map(|skip| ["--skip", skip]))
                .envs(&env_vars)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?;

            let output = cmd.wait_with_output()?;
            if !output.status.success() {
                spinner.stop_and_persist(
                    "❌",
                    format!("Tests{} failed!", tests_signifier),
                );
                eprintln!("cargo test stdout:");
                eprintln!("{}", std::str::from_utf8(&output.stdout)?);
                eprintln!("cargo test stderr:");
                eprintln!("{}", std::str::from_utf8(&output.stderr)?);
                bail!("`cargo test` failed");
            }

            spinner.stop_and_persist(
                "✅",
                format!("Tests{} finished!", tests_signifier),
            );

            Ok(())
        };

        // NOTE: cargo does not support providing multiple ranges
        if tests.is_empty() {
            // NOTE: run all tests
            cargo_test(None)?;
        } else {
            // NOTE: run selected tests
            tests.iter().map(Some).try_for_each(cargo_test)?;
        }
    }
    let _after_tests_time = SystemTime::now();

    // TODO: `{before,after}-test` shenanigans/optimizations

    // NOTE: run grcov
    {
        let mut spinner = Spinner::new(
            Spinners::Dots,
            "Aggregating coverage info...".to_string(),
        );
        // NOTE: for filtering out "irrelevant" lines, i.e. only leaving the contract code
        let re = regex::Regex::new(
            r#"(?x)
            ^\s*\#\[(program|account)\]$       # Matches #[program] or #[account]
            |
            ^\s*\#\[(tokio::)?test\]$          # Matches #[test] or #[tokio::test]
            |
            ^\s*\#\[derive\(\s*[^\)]+\s*\)\]$  # Matches #[derive(Trait, ...)]
            |
            ^\s*declare_id!\(\s*.*\s*\);$      # Matches declare_id!(...)
            |
            ^\s*declare_program!\(\s*.*\s*\);$ # Matches declare_id!(...)
            # |
            # ^\s*$                              # Matches "empty" lines
            # |
            # ^\s*[\(\)\[\]\{\}]*\s*$            # Matches lines with only brackets
        "#,
        )?;

        // NOTE: extrapolated from running the original `grcov` CLI with appropriate arguments
        let opt = from_grcov::Opt {
            paths: vec![coverage_dir.to_string()],
            binary_path: Some(target_dir.into()),
            llvm_path: None,
            // NOTE: `create::OutputType` `into()`-es to `crate::from_grcov::OutputType`
            output_types: output_types
                .iter()
                .cloned()
                .map(std::convert::Into::into)
                .collect(),
            output_path: Some(coverage_dir.into()),
            output_config_file: None,
            // NOTE: `.` because we `cd`-ed into the correct directory already
            source_dir: Some(".".into()),
            prefix_dir: None,
            ignore_not_existing: true,
            // NOTE: parsed as globs, see [globset::Globset]
            ignore_dir: vec!["target/*".to_string(), "*tests*".to_string()],
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
            contract_style,
        };

        from_grcov::main(opt)?;
        spinner.stop_and_persist("✅", "Coverage aggregated!".to_string());
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

    // NOTE: Report regenerated outputs (and possibly open, if applicable)
    output_types.iter().unique().try_for_each(|output_type| {
        match output_type {
            OutputType::Html => {
                eprintln!(
                    "Successfully generated html report at {}, opening...",
                    path.join("target/coverage/html/index.html").display(),
                );
                // open::that("./target/coverage/tarpaulin-report.html")
                open::that("./target/coverage/html/index.html")
            }
            OutputType::Lcov => {
                eprintln!(
                    "Successfully generated lcov report, you can find it at {}",
                    path.join("target/coverage/lcov").display(),
                );
                Ok(())
            }
        }
    })?;

    Ok(())
}
