use crate::openai;
use crate::tools_interface::ToolInstance;
use serde_json::Value;
use std::io;
use std::process::Command;

/// A tool for automating git submissions.
/// The working directory is configurable, allowing this tool to be used in any repository directory.
pub struct GitSubmissionTool {
    working_dir: String,
}

impl GitSubmissionTool {
    /// Creates a new GitSubmissionTool for the specified working directory.
    pub fn new(working_dir: impl Into<String>) -> Self {
        GitSubmissionTool {
            working_dir: working_dir.into(),
        }
    }

    /// Adds all changes in the repository.
    ///
    /// # Returns
    /// Returns an io::Result indicating either success or failure
    fn add_changes(&self) -> io::Result<()> {
        let status = Command::new("git")
            .arg("add")
            .arg(".")
            .current_dir(&self.working_dir)
            .status()?;
        if !status.success() {
            return Err(io::Error::other("git add failed"));
        }
        Ok(())
    }

    /// Commits all added changes.
    ///
    /// # Arguments
    /// * `commit_message` - The commit message, provided by the llm. If none is provided, it'll still work.
    ///
    /// # Returns
    /// Returns an io::Result indicating either success or failure
    fn commit_changes(&self, commit_message: Option<&str>) -> io::Result<()> {
        let c_message = commit_message.unwrap_or("No message :(");
        let status = Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg(c_message)
            .current_dir(&self.working_dir)
            .status()?;
        if !status.success() {
            return Err(io::Error::other("git commit failed"));
        }
        Ok(())
    }

    /// Pushes all committed changes.
    ///
    /// # Returns
    /// Returns an io::Result indicating success or failure
    fn push_changes(&self) -> io::Result<()> {
        let status = Command::new("git")
            .arg("push")
            .arg("origin")
            .arg("HEAD")
            .current_dir(&self.working_dir)
            .status()?;
        if !status.success() {
            return Err(io::Error::other("git push failed"));
        }
        Ok(())
    }

    /// Full action cycle: adds, commits, and pushes all changes.
    ///
    /// # Arguments
    /// * `commit_message` - the commit messsage provided by the llm. If none is provided it'll still work.
    ///
    /// # Returns
    /// Returns an io::Result indicating either success or failure
    fn submit_changes(&self, commit_message: Option<&str>) -> io::Result<()> {
        self.add_changes()?;
        self.commit_changes(commit_message)?;
        self.push_changes()?;
        Ok(())
    }
}

// Implements Default trait for GitSubmissionTool, defaulting to current directory.
impl Default for GitSubmissionTool {
    fn default() -> Self {
        GitSubmissionTool::new(".")
    }
}

/// Implements the ToolInstance trait for GitSubmissionTool, allowing it to be used as a dynamic tool
impl ToolInstance for GitSubmissionTool {
    /// Runs the requested git action based on the parameters provided.
    ///
    /// # Paramete  rs
    /// * `params`: a serde_json::Value containing the following keys:
    ///     - "action": String. One of "add", "commit", "push" or "submit".
    ///     - "commit_message": String (optional). Used for "commit" and "submit".
    ///     - "working_dir": String (optional). Overrides the working directory for this invocation.
    ///
    /// # Returns
    /// Returns a JSON String describing the outcome or an error.
    fn run(&self, params: Value) -> Result<Value, Box<dyn std::error::Error>> {
        let action = params
            .get("action")
            .and_then(|v| v.as_str())
            .ok_or("Missing parameter: action")?;
        let commit_message = params.get("commit_message").and_then(|v| v.as_str());
        // Allow override of working directory for this invocation
        let working_dir = params
            .get("working_dir")
            .and_then(|v| v.as_str())
            .unwrap_or(&self.working_dir)
            .to_string();
        // Create a temporary tool instance if override is used
        let tool = if working_dir != self.working_dir {
            GitSubmissionTool::new(working_dir)
        } else {
            GitSubmissionTool {
                working_dir: self.working_dir.clone(),
            }
        };

        let result = match action {
            "add" => {
                tool.add_changes()?;
                "git add . executed".to_string()
            }
            "commit" => {
                tool.commit_changes(commit_message)?;
                format!(
                    "git commit executed with message: {:?}",
                    commit_message.unwrap_or("No message :(")
                )
            }
            "push" => {
                tool.push_changes()?;
                "git push origin HEAD executed".to_string()
            }
            "submit" => {
                tool.submit_changes(commit_message)?;
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
                description: "Executes git add, commit, and push. Please always provide commit message. Optionally, set working_dir.".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "action": {
                            "type": "string",
                            "description": "Choose one: \"add\", \"commit\", \"push\" or \"submit\""
                        },
                        "commit_message": {
                            "type": "string",
                            "description": "Commit message summarizing the changes."
                        },
                        "working_dir": {
                            "type": "string",
                            "description": "Directory where git commands are executed. Defaults to current directory."
                        }
                    },
                    "required": ["action"]
                }),
            },
            tool_type: "function".to_string(),
        }
    }
}
