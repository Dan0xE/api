use clap::Parser;
use codedefender_config::{YAML_CONFIG_VERSION, YamlConfig};
use std::{fs, io::Error, path::PathBuf};

mod api {
    pub use codedefender_api::analyze_program as analyze;
    pub use codedefender_api::upload_file as upload;
}

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

    let client = reqwest::blocking::Client::new();
    let binary_file_bytes = fs::read(config.input_file)?;

    // Upload binary and optionally PDB file
    let binary_file_uuid = api::upload(binary_file_bytes, &client, &config.api_key)
        .expect("Failed to upload binary file!");

    let pdb_file_uuid = match config.pdb_file {
        Some(path) => {
            let pdb_file_bytes = fs::read(path)?;
            Some(
                api::upload(pdb_file_bytes, &client, &config.api_key)
                    .expect("Failed to upload PDB file!"),
            )
        }
        _ => None,
    };

    match api::analyze(binary_file_uuid, pdb_file_uuid, &client, &config.api_key) {
        Ok(analysis) => {
            
        }
        Err(e) => {
            panic!("Analysis failed: {}", e.to_string());
        }
    }
    Ok(())
}
