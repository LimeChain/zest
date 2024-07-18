use std::{fs::File, io::Read, path::PathBuf};

use clap_serde_derive::{
    clap::{self, Parser, ValueEnum},
    ClapSerde,
};
use serde::{Deserialize, Serialize};

pub mod from_grcov;
pub mod util;

#[derive(Parser)]
#[command(version, about)]
pub struct Args {
    /// Config file
    #[arg(short, long = "config", default_value = "solcov.toml")]
    pub config_path: PathBuf,

    /// Rest of arguments
    #[command(flatten)]
    pub config: <Config as ClapSerde>::Opt,
}

// #[derive(Debug, Clone, PartialEq, Parser, Serialize, Deserialize)]
#[derive(Debug, ClapSerde)]
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

impl Config {
    pub fn parse() -> eyre::Result<Self> {
        // Parse from CLI
        let mut args = <Args as Parser>::parse();

        let config = if let Ok(mut f) = File::open(&args.config_path) {
            // Parse config with serde
            let mut config_string = String::new();
            f.read_to_string(&mut config_string)?;
            let config: <Config as ClapSerde>::Opt = toml::from_str(config_string.as_str())?;
            Config::from(config).merge(&mut args.config)
        } else {
            // If there is no config file - return only the config parsed from clap
            Config::from(&mut args.config)
        };

        Ok(config)
    }
}
