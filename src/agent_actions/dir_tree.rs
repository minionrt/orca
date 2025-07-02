use crate::openai;
use crate::tools_interface::ToolInstance;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize)]
pub struct DirEntry {   
    pub name: String,
    pub is_dir: bool,
    pub children: Option<Vec<DirEntry>>,
}

pub struct DirTreeTool;

impl DirTreeTool {
    pub fn new() -> Self {
        DirTreeTool
    }

    /// Recursively reads a directory and builds the tree structure.
    pub fn read_dir_tree<P: AsRef<Path>>(path: P) -> std::io::Result<DirEntry> {
        let path = path.as_ref();
        let name = path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.display().to_string());
        if path.is_dir() {
            let children = fs::read_dir(path)?
                .filter_map(|e| e.ok())
                .map(|e| DirTreeTool::read_dir_tree(e.path()))
                .filter_map(Result::ok)
                .collect();
            Ok(DirEntry { name, is_dir: true, children: Some(children) })
        } else {
            Ok(DirEntry { name, is_dir: false, children: None })
        }
    }
}

// Implements Default trait for DirTreeTool
impl Default for DirTreeTool {
    fn default() -> Self {
        DirTreeTool::new()
    }
}

impl ToolInstance for DirTreeTool {
    /// Runs the directory tree tool
    /// # Parameters
    /// * `params`: serde_json::Value with optional "path" String (default: ".")
    /// # Returns
    /// Returns a JSON representation of the directory tree or an error.
    fn run(&self, params: Value) -> Result<Value, Box<dyn std::error::Error>> {
        let path = params.get("path").and_then(|v| v.as_str()).unwrap_or(".");
        let tree = Self::read_dir_tree(path)?;
        Ok(serde_json::to_value(tree)?)
    }

    /// Returns the tool definition for this tool, including parameters and descriptions
    fn return_choice() -> openai::Tool {
        openai::Tool {
            function: openai::Function {
                name: "dir_tree".to_string(),
                description: "Recursively returns the directory structure as JSON.".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "The directory to scan (default: current directory \".\")"
                        }
                    }
                }),
            },
            tool_type: "function".to_string(),
        }
    }
}