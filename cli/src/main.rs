use clap::Parser;
use codedefender_api::DownloadStatus;
use codedefender_config::{
    AnalysisResult, CDConfig, CDProfile, YAML_CONFIG_VERSION, YamlConfig, YamlSymbol,
};
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

/// Commandline interface for CodeDefender
#[derive(Parser, Debug)]
#[command(name = "codedefender-cli")]
#[command(about = "Commandline interface for CodeDefender", long_about = None)]
pub struct Cli {
    /// Path to the YAML configuration file
    #[arg(short, long, value_name = "FILE")]
    pub config: PathBuf,

    /// Log level (error, warn, info, debug, trace)
    #[arg(long, value_enum, default_value = "info")]
    pub log_level: log::LevelFilter,

    /// API key provided by the CodeDefender web service. You can either pass it on the commandline or assign it to "CD_API_KEY" env variable.
    #[arg(long, env = "CD_API_KEY")]
    pub api_key: String,

    /// Poll timeout for downloading the obfuscated program (in milliseconds)
    /// Do not go below 500 otherwise you will be timed out.
    #[arg(long, default_value_t = 500)]
    pub timeout: u64,

    /// Input binary to process
    #[arg(long, value_name = "INPUT")]
    pub input_file: PathBuf,

    /// Optional debug symbol (PDB) file
    #[arg(long, value_name = "PDB")]
    pub pdb_file: Option<PathBuf>,

    /// Output path for the Zip file containing the obfuscated binary and dbg file
    #[arg(long, value_name = "OUTPUT")]
    pub output: PathBuf,
}

// Resolve symbol names to RVA's. If a symbol is specified via RVA
// then validate it before including it in the result.
fn resolve_symbols(
    symbols: &[YamlSymbol],
    analysis: &AnalysisResult,
) -> Result<Vec<u64>, Box<dyn std::error::Error>> {
    let mut resolved = Vec::new();

    for symbol in symbols {
        match symbol {
            YamlSymbol::Name(name) => {
                // Search in returned functions and rejects for symbol by name.
                // If it was rejected for "ReadWriteToCode" we will force resolve it.
                let rva = analysis
                    .functions
                    .iter()
                    .find(|f| f.symbol == *name)
                    .map(|e| e.rva)
                    .or_else(|| {
                        analysis
                            .rejects
                            .iter()
                            .find(|r| r.symbol == *name && r.ty == "ReadWriteToCode")
                            .map(|e| e.rva)
                    });

                match rva {
                    Some(rva) => resolved.push(rva),
                    None => {
                        log::error!("Symbol `{}` not found in analysis result", name);
                        return Err("Missing symbol".into());
                    }
                }
            }
            YamlSymbol::Rva(rva) => {
                if !is_valid_rva(*rva, analysis) {
                    log::error!("RVA {:X} not found in analysis", rva);
                    return Err("Invalid RVA".into());
                }
                resolved.push(*rva);
            }
        }
    }

    Ok(resolved)
}

fn is_valid_rva(rva: u64, analysis: &AnalysisResult) -> bool {
    analysis.functions.iter().any(|f| f.rva == rva)
        || analysis
            .rejects
            .iter()
            .any(|r| r.rva == rva && r.ty == "ReadWriteToCode")
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    env_logger::builder().filter_level(cli.log_level).init();

    let config_contents = fs::read_to_string(&cli.config)?;
    let config: YamlConfig = serde_yaml::from_str(&config_contents)?;

    if config.version != YAML_CONFIG_VERSION {
        log::error!(
            "Invalid config version: {}, expected: {}",
            config.version,
            YAML_CONFIG_VERSION
        );
        log::error!("Latest version available at: {CLI_DOWNLOAD_LINK}");
        return Ok(());
    }

    let client = reqwest::blocking::Client::new();
    let binary_file_bytes = fs::read(&cli.input_file)?;
    let binary_file_uuid = api::upload(binary_file_bytes, &client, &cli.api_key)?;

    let pdb_file_uuid = match &cli.pdb_file {
        Some(path) => Some(api::upload(fs::read(path)?, &client, &cli.api_key)?),
        None => None,
    };

    log::info!("Uploaded file(s)...");
    log::info!("Analyzing program...");

    let analysis = api::analyze(
        binary_file_uuid.clone(),
        pdb_file_uuid,
        &client,
        &cli.api_key,
    )?;

    log::info!("Analysis finished...");
    log::info!("Constructing config...");

    let mut cdconfig = CDConfig {
        module_settings: config.module_settings,
        profiles: vec![],
    };

    for profile in &config.profiles {
        let symbols = resolve_symbols(profile.symbols.as_slice(), &analysis)?;
        cdconfig.profiles.push(CDProfile {
            name: profile.name.clone(),
            passes: profile.passes.clone(),
            compiler_settings: profile.compiler_settings.clone(),
            symbols,
        });
    }

    for macro_profile in &analysis.macros {
        let profile = cdconfig
            .profiles
            .iter_mut()
            .find(|p| p.name == macro_profile.name);
        match profile {
            Some(p) => {
                for rva in &macro_profile.rvas {
                    if !is_valid_rva(*rva, &analysis) {
                        log::error!("Macro-decorated function {:X} cannot be protected", rva);
                        return Ok(());
                    }
                }
                p.symbols.extend(macro_profile.rvas.clone());
            }
            None => {
                log::error!(
                    "Macro specifies profile `{}` which is not defined in the config",
                    macro_profile.name
                );
                return Ok(());
            }
        }
    }

    log::info!("Obfuscating program...");
    let execution_id = api::defend(binary_file_uuid, cdconfig, &client, &cli.api_key)?;
    let start_time = Instant::now();
    let timeout_duration = Duration::from_secs(300); // 5 min

    loop {
        if start_time.elapsed() > timeout_duration {
            log::error!("Timeout: obfuscation exceeded 5 minutes");
            return Ok(());
        }

        match api::download(execution_id.clone(), &client, &cli.api_key) {
            DownloadStatus::Ready(bytes) => {
                fs::write(&cli.output, bytes)?;
                log::info!("Obfuscated binary written to {:?}", cli.output);
                return Ok(());
            }
            DownloadStatus::Processing => {
                log::info!("Still Obfuscating...");
            }
            DownloadStatus::Failed(e) => {
                log::error!("Obfuscation failed: {}", e);
                return Ok(());
            }
        }

        std::thread::sleep(Duration::from_millis(cli.timeout));
    }
}
