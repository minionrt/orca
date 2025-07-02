use crate::agent_actions::{bash, edit_files, read_files};
use crate::openai::Tool;
use crate::tools_interface::ToolInstance;
use std::error::Error;

/// Returns all available tools expected by the LLM
pub fn get_tools() -> Option<Vec<Tool>> {
    let tools = vec![
        read_files::ReadFilesTool::return_choice(),
        edit_files::EditFilesTool::return_choice(),
        bash::BashTool::return_choice(),
        // more tools can be added here
    ];

    if tools.is_empty() { None } else { Some(tools) }
}

/// Calls the specified tool by name with the given arguments
/// Returns a JSON result or an error string
pub fn call_tool(
    tool_name: &str,
    args: serde_json::Value,
) -> Result<serde_json::Value, Box<dyn Error>> {
    Ok(match tool_name {
        "read_files" => serde_json::to_value(read_files::ReadFilesTool::run(
            serde_json::from_value(args)?,
        )?)?,
        "edit_files" => serde_json::to_value(edit_files::EditFilesTool::run(
            serde_json::from_value(args)?,
        )?)?,
        "bash" => serde_json::to_value(bash::BashTool::run(serde_json::from_value(args)?)?)?,
        _ => return Err(format!("Tool '{tool_name}' not found.").into()),
    })
}
