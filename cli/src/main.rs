use clap::Parser;
use codedefender_api::DownloadStatus;
use codedefender_config::{CDConfig, CDProfile, YAML_CONFIG_VERSION, YamlConfig, YamlSymbol};
use std::{
    fs,
    path::PathBuf,
    time::{Duration, Instant},
};

mod api {
    pub use codedefender_api::analyze_program as analyze;
    pub use codedefender_api::defend;
    pub use codedefender_api::download;
    pub use codedefender_api::upload_file as upload;
}

const CLI_DOWNLOAD_LINK: &str = "https://github.com/codedefender-io/api/releases";

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

fn main() {
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
        return;
    }

    let client = reqwest::blocking::Client::new();
    let binary_file_bytes =
        fs::read(config.input_file).expect("Failed to read binary! invalid path?");

    // Upload binary and optionally PDB file
    let binary_file_uuid = api::upload(binary_file_bytes, &client, &config.api_key)
        .expect("Failed to upload binary file!");

    let pdb_file_uuid = match config.pdb_file {
        Some(path) => {
            let pdb_file_bytes = fs::read(path).expect("Failed to read PDB bytes!");
            Some(
                api::upload(pdb_file_bytes, &client, &config.api_key)
                    .expect("Failed to upload PDB file!"),
            )
        }
        _ => None,
    };

    match api::analyze(
        binary_file_uuid.clone(),
        pdb_file_uuid,
        &client,
        &config.api_key,
    ) {
        Ok(analysis) => {
            let mut cdconfig = CDConfig {
                module_settings: config.module_settings,
                profiles: Default::default(),
            };

            for profile in config.profiles.iter() {
                let mut symbols = Vec::<u64>::new();
                for symbol in profile.symbols.iter() {
                    match symbol {
                        YamlSymbol::Name(name) => {
                            if let Some(rva) = analysis
                                .functions
                                .iter()
                                .find(|sym| sym.symbol == *name)
                                .map(|e| e.rva)
                            {
                                symbols.push(rva);
                            } else {
                                if let Some(rva) = analysis
                                    .rejects
                                    .iter()
                                    .find(|sym| sym.symbol == *name && sym.ty == "ReadWriteToCode")
                                    .map(|e| e.rva)
                                {
                                    symbols.push(rva);
                                } else {
                                    log::error!(
                                        "Could not find function with symbol name: {}",
                                        *name
                                    );
                                    return;
                                }
                            }
                        }
                        YamlSymbol::Rva(rva) => {
                            // If the symbol is not found within the rejects or the valid functions
                            // we will panic and let the user know we have an issue!
                            if analysis
                                .functions
                                .iter()
                                .find(|sym| sym.rva == *rva)
                                .is_none()
                                && analysis
                                    .rejects
                                    .iter()
                                    .find(|sym| sym.rva == *rva && sym.ty == "ReadWriteToCode")
                                    .is_none()
                            {
                                log::error!("No function with RVA({:X}) found in analysis!", *rva);
                                return;
                            }
                            symbols.push(*rva);
                        }
                    }
                }

                cdconfig.profiles.push(CDProfile {
                    name: profile.name.clone(),
                    passes: profile.passes.clone(),
                    compiler_settings: profile.compiler_settings.clone(),
                    symbols,
                });
            }

            for value in analysis.macros.iter() {
                let profile = cdconfig.profiles.iter_mut().find(|p| p.name == value.name);
                match profile {
                    Some(p) => {
                        // Validate that the rvas in this macro profile can actually be protected.
                        for rva in value.rvas.iter() {
                            if analysis
                                .functions
                                .iter()
                                .find(|sym| sym.rva == *rva)
                                .is_none()
                                && analysis
                                    .rejects
                                    .iter()
                                    .find(|reject| {
                                        reject.rva == *rva && reject.ty == "ReadWriteToCode"
                                    })
                                    .is_none()
                            {
                                log::error!(
                                    "Function {:X} decorated with a macro cant be protected.",
                                    *rva
                                );
                                return;
                            }
                        }
                        p.symbols.extend(value.rvas.clone());
                    }
                    None => {
                        log::error!(
                            "Program uses source macros, specified a profile `{}` but no profile exists in the config!",
                            value.name
                        );
                        return;
                    }
                }
            }

            // Upload config to defend endpoint now.
            match api::defend(binary_file_uuid, cdconfig, &client, &config.api_key) {
                Ok(execution_id) => {
                    let start_time = Instant::now();
                    // The backend will only run for 5 minutes.
                    let timeout_duration = Duration::from_secs(300); // 5 minutes
                    
                    loop {
                        if start_time.elapsed() > timeout_duration {
                            log::error!(
                                "Timeout: obfuscation took longer than 5 minutes, if you need this much obfuscation contact us for enterprise deployment!"
                            );
                            return;
                        }

                        // Start polling for the result.
                        match api::download(execution_id.clone(), &client, &config.api_key) {
                            DownloadStatus::Ready(bytes) => {
                                fs::write(config.output_file.clone(), bytes)
                                    .expect("Failed to write output file!");
                            }
                            DownloadStatus::Processing => {}
                            DownloadStatus::Failed(error) => {
                                log::error!("Obfuscation failed: {}", error.to_string());
                                return;
                            }
                        }
                        std::thread::sleep(Duration::from_millis(config.timeout));
                    }
                }
                Err(e) => {
                    log::error!("Failed to defend file: {}", e.to_string());
                }
            }
        }
        Err(e) => {
            log::error!("Analysis failed: {}", e.to_string());
            return;
        }
    }
}
