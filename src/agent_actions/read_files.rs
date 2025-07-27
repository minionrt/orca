use crate::openai;
use crate::tools_interface::ToolInstance;
use serde::Deserialize;
use std::collections::HashMap;
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
pub struct ReadFilesToolArgs {
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
    fn return_choice() -> openai::Tool {
        openai::Tool::function(
            "read_files".to_owned(),
            "Reads the content of a file.".to_owned(),
            HashMap::from([(
                "path".to_owned(),
                openai::FunctionParameter::new("string", "Path to the file to read"),
            )]),
            HashMap::new(),
        )
    }
}
