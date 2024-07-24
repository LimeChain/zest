use std::{fs::File, io::Read, path::PathBuf};

use clap_serde_derive::{
    clap::{self, Parser},
    ClapSerde,
};

#[derive(Parser)] // , Debug, Clone)]
#[command(version, about)]
pub struct WithConfigFile<Config>
where
    Config: ClapSerde,
    // <Config as ClapSerde>::Opt: Debug + Clone,
{
    /// Config file
    #[arg(short, long = "config")]
    pub config_path: PathBuf,

    /// Rest of arguments
    #[command(flatten)]
    pub config: <Config as ClapSerde>::Opt,
}

pub trait ParseWithConfigFile
where
    Self: ClapSerde,
{
    fn parse_with_config_file(args: Option<WithConfigFile<Self>>) -> eyre::Result<Self>;
}

impl<Config> ParseWithConfigFile for Config
where
    Config: ClapSerde,
    // <Config as ClapSerde>::Opt: Debug + Clone,
{
    fn parse_with_config_file(args: Option<WithConfigFile<Self>>) -> eyre::Result<Self> {
        // Parse from CLI
        let mut args =
            args.unwrap_or_else(<WithConfigFile<Self> as Parser>::parse);

        let config = if let Ok(mut f) = File::open(&args.config_path) {
            // Parse config with serde
            let mut config_string = String::new();
            f.read_to_string(&mut config_string)?;
            let config: <Config as ClapSerde>::Opt =
                toml::from_str(config_string.as_str())?;
            Self::from(config).merge(&mut args.config)
        } else {
            // If there is no config file - return only the config parsed from clap
            Self::from(&mut args.config)
        };

        Ok(config)
    }
}