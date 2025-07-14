use crate::agent_actions::{bash, edit_files, read_files, submit_code};
use crate::openai::Tool;
use crate::tools_interface::ToolInstance;
use std::error::Error;
use tracing::{debug, error};

/// Returns all available tools expected by the LLM
pub fn get_tools() -> Option<Vec<Tool>> {
    let tools = vec![
        read_files::ReadFilesTool::return_choice(),
        edit_files::EditFilesTool::return_choice(),
        bash::BashTool::return_choice(),
        submit_code::GitSubmissionTool::return_choice(),
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
    debug!("Calling tool: {} with args: {:?}", tool_name, args);
    
    let result = match tool_name {
        "read_files" => {
            let tool = read_files::ReadFilesTool;
            tool.run(args)
        }
        "edit_files" => {
            let tool = edit_files::EditFilesTool;
            tool.run(args)
        }
        "bash" => {
            let tool = bash::BashTool;
            tool.run(args)
        }
        "git_submission" => {
            let tool = submit_code::GitSubmissionTool::default();
            tool.run(args)
        }
        _ => {
            error!("Unknown tool requested: {}", tool_name);
            Err(format!("Tool '{tool_name}' not found.").into())
        },
    };
    
    match &result {
        Ok(_) => debug!("Tool {} executed successfully", tool_name),
        Err(e) => error!("Tool {} failed: {}", tool_name, e),
    }
    
    result
}
