use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use serde::{Deserialize, Serialize};

pub mod from_grcov;
pub mod util;

#[derive(Debug, Clone, PartialEq, Parser, Serialize, Deserialize)]
#[command(version, about)]
pub struct Args {
    #[arg(long, default_value = ".", help = "Path to the solana project")]
    pub path: PathBuf,

    #[arg(
        long,
        // default_value = "stable",
        help = "Version of the compiler to use. Nightly required for branch coverage",
    )]
    pub compiler_version: Option<String>,

    #[arg(
        long,
        default_value_t = false,
        help = "Whether to enable branch coverage (nightly compiler required)"
    )]
    pub branch: bool,

    #[arg(
        long,
        value_enum,
        default_value_t = CoverageStrategy::InstrumentCoverage,
        help = "Coverage strategy to use",
    )]
    pub coverage_strategy: CoverageStrategy,

    // TODO: `-- --exact`?
    #[arg(long, help = "Which tests to run (same as `cargo test`)")]
    pub tests: Option<String>,

    #[arg(long, value_enum, default_value_t = OutputType::Html, help = "Output type of coverage")]
    pub output_type: OutputType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, ValueEnum, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, ValueEnum, Serialize, Deserialize)]
pub enum CoverageStrategy {
    #[default]
    InstrumentCoverage,
    // BUG: does not currently work
    ZProfile,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            path: ".".into(),
            compiler_version: None,
            branch: false,
            coverage_strategy: CoverageStrategy::InstrumentCoverage,
            tests: None,
            output_type: OutputType::Html,
        }
    }
}

impl Args {
    pub fn parse_from_config_and_cli() -> eyre::Result<Self> {
        let res = Self::try_parse()?;

        // TODO: parse config file and merge with args

        Ok(res)
    }
}
