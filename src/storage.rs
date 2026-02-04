use crate::models::AnalysisResult;
use anyhow::Result;
use std::fs;
use std::path::Path;

pub const DATA_FILE: &str = "data/results.json";

/// Save analysis results to JSON file
pub fn save_results(results: &[AnalysisResult]) -> Result<()> {
    // Ensure data directory exists
    if let Some(parent) = Path::new(DATA_FILE).parent() {
        fs::create_dir_all(parent)?;
    }

    let json = serde_json::to_string_pretty(results)?;
    fs::write(DATA_FILE, json)?;
    Ok(())
}

/// Load analysis results from JSON file
pub fn load_results() -> Result<Vec<AnalysisResult>> {
    if !Path::new(DATA_FILE).exists() {
        return Ok(Vec::new());
    }

    let json = fs::read_to_string(DATA_FILE)?;
    let results: Vec<AnalysisResult> = serde_json::from_str(&json)?;
    Ok(results)
}
