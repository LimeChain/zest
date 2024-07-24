use clap::Parser;
use clap_serde_derive::clap;

use crate::{config_parsing::WithConfigFile, coverage};

#[derive(Parser)]
pub struct Config {
    #[command(subcommand)]
    pub command: Subcommands,
}

#[derive(Parser)]
pub enum Subcommands {
    #[command(alias = "c")]
    Coverage(WithConfigFile<coverage::Config>),

    #[command(alias = "g")]
    // TODO: test file, whole repo
    Generate,
}
