use crate::agent_actions::submit_code::GitSubmissionToolArgs;
use crate::agent_actions::{bash, dir_tree, edit_files, read_files, submit_code, ask_user};
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
        ask_user::AskUserTool::return_choice(),
        dir_tree::DirTreeTool::return_choice(),
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
        "read_files" => serde_json::to_value(read_files::ReadFilesTool::run(
            serde_json::from_value(args)?,
        )?)?,
        "edit_files" => serde_json::to_value(edit_files::EditFilesTool::run(
            serde_json::from_value(args)?,
        )?)?,
        "bash" => serde_json::to_value(bash::BashTool::run(serde_json::from_value(args)?)?)?,
        "git_submission" => {
            // this implicitly sets "args.this" to the default
            let args: GitSubmissionToolArgs = serde_json::from_value(args)?;
            serde_json::to_value(submit_code::GitSubmissionTool::run(args)?)?
        }
        "dir_tree" => {
            serde_json::to_value(dir_tree::DirTreeTool::run(serde_json::from_value(args)?)?)?
        }
        "ask_user" =>{
            serde_json::to_value(ask_user::AskUserTool::run(serde_json::from_value(args)?)?)?
        }
        _ => {
            error!("Unknown tool requested: {}", tool_name);
            return Err(format!("Tool '{tool_name}' not found.").into());
        }
    };

    debug!("Tool {} executed successfully", tool_name);
    Ok(result)
}
