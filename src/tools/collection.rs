use crate::agent_actions::{edit_files, read_files};
use serde_json::Value;
use crate::openai::Tool;




/*/// Struct to represent a tool in the format expected by the LLM
#[derive(Debug, Clone)]
pub struct Tool {
    pub name: &'static str,
    pub description: &'static str,
}
*/

/// Returns all available tools expected by the LLM
pub fn get_tools() -> Option<Vec<Tool>> {
    let tools = vec![
        Tool {
            name: "read_files".to_string(),
            description: "Reads the content of files.".to_string(),
            
        },
        
        Tool {
            name: "edit_files".to_string(),
            description: "Edits the content of files.".to_string(),

        }
    ];

    if tools.is_empty() {
        None
    } else {
        Some(tools)
    }
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
