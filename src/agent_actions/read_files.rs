use std::fs;
use serde_json::{json, Value};
use anyhow::{Result, anyhow};

/// Reads the contents of a file and returns it as a `String`.
///
/// # Arguments
///
/// * `path` - The path to the file to read.
///
/// # Returns
///
/// * `Some(String)` containing the file contents if successful.
/// * `None` if the file cannot be read.
pub fn read_file(path: String) -> Option<String> {
    match fs::read_to_string(&path) {
        Ok(content) => Some(content),
        Err(e) => {
            eprintln!("Failed to read file '{}': {}", path, e);
            None
        }
    }
}

/// Returns the JSON Schema for the parameters of the `read_files` tool
pub fn parameters_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "path": {
                "type": "string",
                "description": "Path to the file to read."
            }
        },
        "required": ["path"]
    })
}

/// Executes the `read_files` tool using JSON arguments
/// Expected argument: { "path": "<path-to-file>" }
pub fn run(args: Value) -> Result<Value> {
    let path = args
        .get("path")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("Missing or invalid 'path' parameter"))?;

    let content = fs::read_to_string(path)?;
    Ok(json!({ "content": content }))
}
