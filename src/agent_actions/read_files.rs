use crate::openai::{Function, Tool};
use crate::tools_interface::ToolInstance;
use serde_json::json;
use std::fs;

/// Tool to read the contents of a file from disk
pub struct ReadFilesTool;

impl ReadFilesTool {
    /// Reads the entire content of a file at the given path
    /// returns the content as a String or an I/O error
    pub fn read_file(&self, path: &str) -> std::io::Result<String> {
        fs::read_to_string(path)
    }
}

impl ToolInstance for ReadFilesTool {
    /// Runs the tool using JSON parameters
    /// Expects a "path" parameter, returns file content and a success status
    fn run(
        &self,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        // extract the path parameter from the JSON input
        let path = params
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'path' parameter")?;

        // read the file content
        let content = self.read_file(path)?;

        // return JSON with status and content
        Ok(json!({
            "status": "success",
            "content": content
        }))
    }

    /// returns the tool's definition and JSON schema for LLM integration
    fn return_choice() -> Tool {
        Tool {
            tool_type: "function".to_string(),
            function: Function {
                name: "read_files".to_string(),
                description: "Reads the content of a file.".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Path to the file to read."
                        }
                    },
                    "required": ["path"]
                }),
            },
        }
    }
}
