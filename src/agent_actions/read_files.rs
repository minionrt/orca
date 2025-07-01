use crate::openai::{Function, Tool};
use crate::tools_interface::ToolInstance;
use serde_json::json;
use std::fs;

/// Reads the contents of a file and returns it as a `String`.
///
/// # Arguments
///
/// * `path` -  The path to the file to read.
///
/// # Returns
///
/// * `Some(String)` containing the file contents if successful.
/// * `None` if the file cannot be read.
pub fn read_file(path: String) -> Option<String> {
    match fs::read_to_string(&path) {
        Ok(content) => Some(content),
        Err(e) => {
            eprintln!("Failed to read file '{path}': {e}");
            None
        }
    }
}
