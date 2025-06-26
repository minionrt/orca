use crate::agent_actions::{edit_files, read_files};
use crate::openai::Tool;
use crate::tools_interface::ToolInstance;
use anyhow::Result;

/// Returns all available tools expected by the LLM
pub fn get_tools() -> Option<Vec<Tool>> {
    let tools = vec![
        read_files::ReadFilesTool::return_choice(),
        edit_files::EditFilesTool::return_choice(),
        // more tools can be added here
    ];

    if tools.is_empty() { None } else { Some(tools) }
}

/// Calls the specified tool by name with the given arguments
/// Returns a JSON result or an error string
pub fn call_tool(tool_name: &str, args: serde_json::Value) -> Result<serde_json::Value> {
    match tool_name {
        "read_files" => read_files::ReadFilesTool::run_from_value(args),
        "edit_files" => edit_files::EditFilesTool::run_from_value(args),
        _ => Err(anyhow::anyhow!("Tool '{}' not found.", tool_name)),
    }
}
