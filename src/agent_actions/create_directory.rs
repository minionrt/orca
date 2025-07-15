use std::fs;
use std::io;
//use tracing::{debug, info};
use crate::openai;
use crate::tools_interface::ToolInstance;

pub struct CreateDirectoryTool;

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
    //debug!("Creating directory: {}", path);
    fs::create_dir_all(path)?;
    //info!("Directory created successfully: {path}");
    Ok(())
}

impl ToolInstance for CreateDirectoryTool {
    fn run(
        &self,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        //debug!("CreateDirectoryTool::run called with params: {:?}", params);

        let path = match params.get("path") {
            None => {
                return Err("The parameter \"path\" doesn't exist in the given tool call".into());
            }
            Some(serde_json::Value::String(s)) => s,
            Some(_) => {
                return Err("The parameter \"path\" isn't given as string.".into());
            }
        };

        //info!("Creating directory: {}", path);
        create_directory(path)?;

        Ok(serde_json::Value::String(format!(
            "Directory created: {path}"
        )))
    }

    fn return_choice() -> openai::Tool {
        openai::Tool {
            function: openai::Function {
                name: "create_directory".to_string(),
                description: "Creates a directory at the specified path. Creates parent directories if they don't exist.".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "The path where the directory should be created"
                        }
                    },
                    "required": ["path"]
                }),
            },
            tool_type: "function".to_string(),
        }
    }
}
