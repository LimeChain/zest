use std::path::PathBuf;

use clap_serde_derive::{
    clap::{self, Parser},
    ClapSerde,
};

use crate::config_parsing::ConfigFileName;

pub mod example_project;
pub mod single_test;

#[derive(Parser, ClapSerde, Debug, Clone)]
pub struct Config {
    #[arg(
        long,
        help = "Path to the where the generated file should be generated"
    )]
    #[default("./test.rs".into())]
    pub path: PathBuf,
}

impl ConfigFileName for Config {
    const NAME: &'static str = "generate";
}

pub fn run(config: Config) -> eyre::Result<()> {
    single_test::realise(config.path)?;

    Ok(())
}
