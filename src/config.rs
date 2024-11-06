use clap::Parser;
use clap_serde_derive::clap;

use crate::{config_parsing::WithConfigFile, coverage, generate};

#[derive(Parser)]
#[command(version, about)]
pub struct Config {
    #[command(subcommand)]
    pub command: Option<Subcommands>,
}

#[derive(Parser)]
pub enum Subcommands {
    /// Run coverage on a Solana project
    #[command(alias = "c")]
    Coverage(WithConfigFile<coverage::Config>),

    /// Generate Solana projects and tests
    #[command(alias = "g")]
    Generate(generate::Config),
}
