use crate::openai;
use crate::tools_interface::ToolInstance;
use serde_json::Value;
use std::io;
use std::process::Command;

/// a tool for automating git submissions
pub struct GitSubmissionTool;

impl GitSubmissionTool {
    pub fn new() -> Self {
        GitSubmissionTool
    }

    /// all changes will be added, if possible
    ///
    /// # Returns
    /// Returns an io::Result indicating either success or failure
    fn add_changes() -> io::Result<()> {
        let status = Command::new("git").arg("add").arg(".").status()?;
        if !status.success() {
            return Err(io::Error::other("git add failed"));
        }
        Ok(())
    }
    /// all added changes will be committed (if possible)
    ///
    /// # Arguments
    /// * `commit_message` - The commit message, provided by the llm. If none is provided, it'll still work.
    ///
    /// # Returns
    /// Returns an io::Result indicating either success or failure
    fn commit_changes(commit_message: Option<&str>) -> io::Result<()> {
        let c_message = commit_message.unwrap_or("No message :(");
        let status = Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg(c_message)
            .status()?;
        if !status.success() {
            return Err(io::Error::other("git commit failed"));
        }
        Ok(())
    }

    /// all committed changes will be pushed
    ///
    /// # Returns
    /// Returns an io::Result indicating success or failure
    fn push_changes() -> io::Result<()> {
        let status = Command::new("git")
            .arg("push")
            .arg("origin")
            .arg("HEAD")
            .status()?;
        if !status.success() {
            return Err(io::Error::other("git push failed"));
        }
        Ok(())
    }
    /// full action cycle of all commands. So it adds, commits and pushes all changes.
    ///
    /// # Arguments
    /// * `commit_message` - the commit messsage provided by the llm. If none is provided it'll still work.
    ///
    /// # Returns
    /// Returns an io::Result indicating either success or failure
    fn submit_changes(commit_message: Option<&str>) -> io::Result<()> {
        Self::add_changes()?;
        Self::commit_changes(commit_message)?;
        Self::push_changes()?;
        Ok(())
    }
}

// Implements Default trait for GitSubmissionTool, allowing creation with ::default()
impl Default for GitSubmissionTool {
    fn default() -> Self {
        GitSubmissionTool::new()
    }
}

/// Implements the ToolInstance trait for GitSubmissionTool, allowing it to be used
/// as a dynamic tool
impl ToolInstance for GitSubmissionTool {
    /// runs the requested git action based in the parameters provided.
    ///
    /// # Parameters
    /// * `params`: a serde::json::Value containing the following keys:
    ///     - "action": String. One of "add", "commit", "push" or "submit".
    ///     - "commit_message": String (technically optional, but highly encuraged). Used for "commit" and "submit".
    /// # Returns
    /// Returns a JSON String describing the outcome or an error.
    fn run(&self, params: Value) -> Result<Value, Box<dyn std::error::Error>> {
        let action = params
            .get("action")
            .and_then(|v| v.as_str())
            .ok_or("Missing parameter: action")?;
        let commit_message = params.get("commit_message").and_then(|v| v.as_str());

        // Match the action to the corresponding git operation.
        let result = match action {
            "add" => {
                Self::add_changes()?;
                "git add . executed".to_string()
            }
            "commit" => {
                Self::commit_changes(commit_message)?;
                format!(
                    "git commit executed with message: {:?}",
                    commit_message.unwrap_or("No message :(")
                )
            }
            "push" => {
                Self::push_changes()?;
                "git push origin HEAD executed".to_string()
            }
            "submit" => {
                Self::submit_changes(commit_message)?;
                format!(
                    "submission successful (add, commit, push) with message: {:?}",
                    commit_message.unwrap_or("No message :(")
                )
            }
            _ => {
                return Err(
                    "Incorrect action parameter. Allowed are: add, commit, push, submit.".into(),
                );
            }
        };
        Ok(Value::String(result))
    }

    /// Returns the tool definition for this tool, including parameters and descriptions.
    fn return_choice() -> openai::Tool {
        openai::Tool {
            function: openai::Function {
                name: "git_submission".to_string(),
                description: "Executes the full git submission using add, commit and push. Please always provide commit message.".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "action": {
                            "type": "string",
                            "description": "Which action? add, commit, push oder submit"
                        },
                        "commit_message": {
                            "type": "string",
                            "description": "Commit message, which summarizes the changes made."
                        }
                    },
                    "required": ["action"]
                }),
            },
            tool_type: "function".to_string(),
        }
    }
}
