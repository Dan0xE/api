use clap::Parser;
use codedefender_config::{YAML_CONFIG_VERSION, YamlConfig};
use std::{fs, io::Error, path::PathBuf};
use codedefender_api::

const CLI_DOWNLOAD_LINK: &str = "https://github.com/codedefender-io/cli/releases";

#[derive(Parser)]
#[command(name = "codedefender-cli")]
#[command(about = "Commandline interface for CodeDefender", long_about = None)]
struct Cli {
    /// Path to the YAML configuration file
    #[arg(short, long, value_name = "FILE")]
    config: PathBuf,

    /// Log level (error, warn, info, debug, trace)
    #[arg(long, value_enum, default_value = "info")]
    log_level: log::LevelFilter,
}

fn main() -> Result<(), Error> {
    let cli = Cli::parse();
    env_logger::builder().filter_level(cli.log_level).init();
    let config_contents = fs::read_to_string(&cli.config).expect("Failed to read the config file");
    let config: YamlConfig =
        serde_yaml::from_str(&config_contents).expect("Failed to parse YAML config");

    if config.version != YAML_CONFIG_VERSION {
        log::error!(
            "Invalid config file version: {}, this build only support: {}",
            config.version,
            YAML_CONFIG_VERSION
        );
        log::error!("Latest here: {CLI_DOWNLOAD_LINK}");
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Invalid config version!",
        ));
    }

    let client = reqwest::Client::new();
    let binary_file_bytes = fs::read(config.input_file)?;
    Ok(())
}
