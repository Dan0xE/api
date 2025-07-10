use codedefender_config::{AnalysisResult, CDConfig};
use reqwest::blocking::Client;

const UPLOAD_EP: &str = "https://app.codedefender.io/api/upload";
const ANALYZE_EP: &str = "https://app.codedefender.io/api/analyze";

/// Result from calling `download`
pub enum DownloadStatus {
    Ready(Vec<u8>),
    Processing,
    Failed(reqwest::Error),
}

/// Upload a file and get a UUID back which will be used for all other API calls
pub fn upload(bytes: Vec<u8>, client: &Client, api_key: &str) -> Result<String, reqwest::Error> {
    let result = client
        .put(UPLOAD_EP)
        .body(bytes)
        .header("Authorization", format!("ApiKey {}", api_key))
        .send()?;

    let result = result.error_for_status()?;
    Ok(result.text()?)
}

/// Analyze a program, returns analysis information containing functions, rejections, etc.
pub fn analyze(
    file: String,
    pdb: Option<String>,
    client: &Client,
    api_key: &str,
) -> AnalysisResult {
    let result = client.put(ANALYZE_EP);
    if let Some(uuid) = pdb {
        
    }
    todo!()
}

/// Start the process of defending the file, this will return an execution UUID
/// which can be used to poll and download the result.
pub fn defend(file: String, config: CDConfig, client: &Client, api_key: &str) -> String {
    todo!()
}

/// Attempt to download the obfuscated binary, will return
/// an error only if obfuscation failed, will return None if its still being
/// processed by the server.
pub fn download(uuid: String, client: &Client, api_key: &str) -> DownloadStatus {
    todo!()
}
