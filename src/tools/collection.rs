use crate::agent_actions::submit_code::GitSubmissionToolArgs;
use crate::agent_actions::{bash, create_directory, dir_tree, edit_files, read_files, submit_code};
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
        create_directory::CreateDirectoryTool::return_choice(),
        submit_code::GitSubmissionTool::return_choice(),
        dir_tree::DirTreeTool::return_choice(),
    ];

    if tools.is_empty() { None } else { Some(tools) }
}

/// Calls the specified tool by name with the given arguments
/// Returns a JSON result or an error string
pub fn call_tool(
    tool_name: &str,
    args: serde_json::Value,
    dir: &str,
) -> Result<serde_json::Value, Box<dyn Error>> {
    debug!("Calling tool: {} with args: {:?}", tool_name, args);
    let result = Ok(match tool_name {
        "read_files" => serde_json::to_value(read_files::ReadFilesTool::run(
            serde_json::from_value(args)?,
        )?)?,
        "edit_files" => {
            #[allow(clippy::let_unit_value)]
            // Ignore that tool always returns null in case we change the edit_files return type
            let result = edit_files::EditFilesTool::run(serde_json::from_value(args)?)?;
            let value = serde_json::to_value(result)?;
            if value == serde_json::Value::Null {
                serde_json::to_value("The file has been sucessfully edited.")?
            } else {
                value
            }
        }
        "bash" => serde_json::to_value(bash::BashTool::run(serde_json::from_value(args)?)?)?,
        "git_submission" => {
            // This implicitly sets "args.this" to the default
            let mut args: GitSubmissionToolArgs = serde_json::from_value(args)?;
            args.working_dir = Some(dir.to_string());
            serde_json::to_value(submit_code::GitSubmissionTool::run(args)?)?
        }
        "create_directory" => serde_json::to_value(create_directory::CreateDirectoryTool::run(
            serde_json::from_value(args)?,
        )?)?,
        "dir_tree" => {
            serde_json::to_value(dir_tree::DirTreeTool::run(serde_json::from_value(args)?)?)?
        }
        _ => {
            error!("Unknown tool requested: {}", tool_name);
            return Err(format!("Tool '{tool_name}' not found.").into());
        }
    });

    debug!("Tool {} executed successfully", tool_name);
    match &result {
        // If a tool returns null, that's automatically interpreted as success, but please handle your tool returns correctly in the match above
        Ok(val) if val == &serde_json::Value::Null => {
            Ok(serde_json::to_value("The action was successful.")?)
        }
        _ => result,
    }
}
