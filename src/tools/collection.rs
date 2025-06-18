use serde_json::Value;
use crate::agent_actions::{edit_files, read_files};

/// Struct to represent a tool in the format expected by the LLM
#[derive(Debug, Clone)]
pub struct Tool {
    pub name: &'static str,
    pub description: &'static str,
}

/// Returns all available tools expected by the LLM
pub fn get_tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "read_files",
            description: "Reads the content of files in the project directory.",
        },
        Tool {
            name: "edit_files",
            description: "Edits the content of files in the project directory.",
        },
        // here, tools can be added 
    ]
}

/// Calls the specified tool by name with the given arguments
/// Returns a JSON result or an error string
pub fn call_tool(tool_name: &str, args: Value) -> Result<Value, anyhow::Error> {
    match tool_name {
        "read_files" => read_files::run(args),
        "edit_files" => edit_files::run(args),
        _ => Err(anyhow::anyhow!("Tool '{}' not found.", tool_name)),
    }
}

