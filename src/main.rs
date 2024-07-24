use clap_serde_derive::clap::Parser;

use solime::{
    config::{Config as MasterConfig, Subcommands},
    config_parsing::ParseWithConfigFile,
    coverage::{self, Config as CoverageConfig}, generate,
};

fn main() -> eyre::Result<()> {
    let MasterConfig { command } = MasterConfig::parse();
    match command {
        Subcommands::Coverage(config) => {
            let config = CoverageConfig::parse_with_config_file(Some(config))?;

            coverage::run(config)
        }
        Subcommands::Generate(config) => {
            generate::run(config)
        }
    }
}
