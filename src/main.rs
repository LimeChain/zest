use clap_serde_derive::clap::Parser;

use zest::{
    config::{Config, Subcommands},
    config_parsing::ParseWithConfigFile,
    coverage, generate,
};

fn main() -> eyre::Result<()> {
    let Config { command } = Config::parse();
    match command.unwrap_or(Subcommands::Coverage(Default::default())) {
        Subcommands::Coverage(config) => {
            let config = coverage::Config::parse_with_config_file(Some(config))?;

            coverage::run(config)
        }
        Subcommands::Generate(config) => {
            generate::run(config)
        }
    }
}
