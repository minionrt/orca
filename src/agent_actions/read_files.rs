use crate::openai::{Function, Tool};
use crate::tools_interface::ToolInstance;
use serde::Deserialize;
use serde_json::json;
use std::fs;

/// Tool to read the contents of a file from disk
pub struct ReadFilesTool;

impl ReadFilesTool {
    /// Reads the entire content of a file at the given path
    /// returns the content as a String or an I/O error
    pub fn read_file(path: &str) -> std::io::Result<String> {
        fs::read_to_string(path)
    }
}

#[derive(Deserialize)]
pub(crate) struct ReadFilesToolArgs {
    pub path: String,
}

impl ToolInstance for ReadFilesTool {
    type Args = ReadFilesToolArgs;
    type Out = String;

    /// Runs the tool using JSON parameters
    /// Expects a "path" parameter, returns file content and a success status
    fn run(input: Self::Args) -> Result<Self::Out, Box<dyn std::error::Error>> {
        Ok(Self::read_file(&input.path)?)
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
