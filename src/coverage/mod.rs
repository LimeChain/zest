use std::path::PathBuf;

use clap_serde_derive::{
    clap::{self, ValueEnum},
    ClapSerde,
};
use serde::{Deserialize, Serialize};

use crate::from_grcov;

// #[derive(Debug, Clone, PartialEq, Parser, Serialize, Deserialize)]
#[derive(ClapSerde, Debug, Clone)]
pub struct Config {
    #[arg(long, help = "Path to the solana project")]
    #[default(".".into())]
    pub path: PathBuf,

    #[arg(
        long,
        help = "Version of the compiler to use. Nightly required for branch coverage"
    )]
    #[default(None)]
    pub compiler_version: Option<String>,

    #[arg(
        long,
        help = "Whether to enable branch coverage (nightly compiler required)"
    )]
    #[default(false)]
    pub branch: bool,

    #[arg(long, value_enum, help = "Coverage strategy to use")]
    #[default(CoverageStrategy::InstrumentCoverage)]
    pub coverage_strategy: CoverageStrategy,

    // TODO: `-- --exact`?
    #[arg(long, help = "Which tests to run (same as `cargo test`)")]
    #[default(None)]
    pub tests: Option<String>,

    #[arg(long, value_enum, help = "Output type of coverage")]
    #[default(OutputType::Html)]
    pub output_type: OutputType,
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
