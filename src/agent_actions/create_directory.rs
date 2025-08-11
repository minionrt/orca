use crate::openai;
use crate::tools_interface::ToolInstance;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::io;
use tracing::{debug, info};

pub struct CreateDirectoryTool;

/// Arguments for the create directory tool
#[derive(Deserialize)]
pub struct CreateDirectoryToolArgs {
    pub path: String,
}

impl CreateDirectoryTool {
    pub fn new() -> Self {
        CreateDirectoryTool
    }
}

impl Default for CreateDirectoryTool {
    fn default() -> Self {
        CreateDirectoryTool::new()
    }
}

/// Creates a directory at the specified path.
///
/// # Arguments
///
/// * `path` - The path where the directory should be created.
///
/// # Returns
///
/// Returns Ok(()) if successful, or an error if the directory cannot be created.
pub fn create_directory(path: &str) -> io::Result<()> {
    debug!("Creating directory: {}", path);
    fs::create_dir_all(path)?;
    info!("Directory created successfully: {path}");
    Ok(())
}

impl ToolInstance for CreateDirectoryTool {
    type Args = CreateDirectoryToolArgs;
    type Out = String;

    fn run(input: Self::Args) -> Result<Self::Out, Box<dyn std::error::Error>> {
        debug!("CreateDirectoryTool::run called with path: {}", input.path);
        create_directory(&input.path)?;
        info!("Creating directory: {}", input.path);
        Ok(format!("Directory created: {}", input.path))
    }

    fn return_choice() -> openai::Tool {
        openai::Tool::function(
            "create_directory".to_string(),
            "Creates a directory at the specified path. Creates parent directories if they don't exist.".to_string(),
            HashMap::from([(
                "path".to_string(),
                openai::FunctionParameter::new("string", "The path where the directory should be created"),
            )]),
            HashMap::new(),
        )
    }
}
