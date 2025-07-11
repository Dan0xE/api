use codedefender_config::{AnalysisResult, CDConfig};
use reqwest::blocking::Client;
use std::collections::HashMap;

const UPLOAD_EP: &str = "https://app.codedefender.io/api/upload";
const ANALYZE_EP: &str = "https://app.codedefender.io/api/analyze";

/// Result from calling `download`
pub enum DownloadStatus {
    Ready(Vec<u8>),
    Processing,
    Failed(reqwest::Error),
}

/// Uploads a binary file to the server and returns its UUID.
///
/// This UUID can be used in subsequent API calls (e.g., for analysis).
///
/// # Arguments
/// - `file_bytes`: The raw contents of the file to upload.
/// - `client`: A preconfigured HTTP client.
/// - `api_key`: Your API key for authentication.
///
/// # Returns
/// A `String` representing the UUID assigned to the uploaded file.
///
/// # Errors
/// Returns a `reqwest::Error` if the request fails or if the server returns a non-success status.
pub fn upload_file(
    file_bytes: Vec<u8>,
    client: &Client,
    api_key: &str,
) -> Result<String, reqwest::Error> {
    let response = client
        .put(UPLOAD_EP)
        .header("Authorization", format!("ApiKey {}", api_key))
        .body(file_bytes)
        .send()?
        .error_for_status()?;

    response.text()
}

/// Performs a remote analysis on a given program file, optionally including a PDB file.
///
/// # Arguments
/// - `file_id`: The identifier for the binary file to analyze.
/// - `pdb_file_id`: Optional identifier for the associated PDB file.
/// - `client`: A preconfigured HTTP client instance.
/// - `api_key`: Your API key for authorization.
///
/// # Returns
/// An `AnalysisResult` containing information about the analyzed binary, including
/// function details and rejection reasons.
///
/// # Errors
/// Returns a `reqwest::Error` if the network request fails or if the server returns a non-success status.
/// Panics if JSON deserialization fails (consider replacing `unwrap()` with proper error handling).
pub fn analyze_program(
    file_id: &str,
    pdb_file_id: Option<&str>,
    client: &Client,
    api_key: &str,
) -> Result<AnalysisResult, reqwest::Error> {
    let mut query_params = HashMap::new();
    query_params.insert("fileId", file_id);
    if let Some(pdb_id) = pdb_file_id {
        query_params.insert("pdbFileId", pdb_id);
    }

    let response = client
        .put(ANALYZE_EP)
        .header("Authorization", format!("ApiKey {}", api_key))
        .query(&query_params)
        .send()?
        .error_for_status()?;

    let result_bytes = response.bytes()?;
    let analysis_result: AnalysisResult =
        serde_json::from_slice(&result_bytes).expect("Failed to deserialize analysis result");

    Ok(analysis_result)
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
