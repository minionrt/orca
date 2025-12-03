use crate::openai;
use crate::tools_interface::ToolInstance;
use serde::Deserialize;
use std::collections::HashMap;
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
    /// * `commit_message` - The commit message, provided by the LLM. If none is provided, it'll still work.
    ///
    /// # Return
    /// Returns an io::Result indicating either success or failure
    fn commit_changes(&self, commit_message: &str) -> io::Result<()> {
        let status = Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg(commit_message)
            .current_dir(&self.working_dir)
            .status()?;
        if !status.success() {
            return Err(io::Error::other("git commit failed"));
        }
        Ok(())
    }

    /// Pushes all committed changes.
    ///
    /// # Return
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
    /// * `commit_message` - the commit messsage provided by the LLM. If none is provided it'll still work.
    ///
    /// # Return
    /// Returns an io::Result indicating either success or failure
    pub fn submit_changes(&self, commit_message: &str) -> io::Result<()> {
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

#[derive(Deserialize)]
pub struct GitSubmissionToolArgs {
    action: String,
    commit_message: String,
    #[serde(default)]
    pub working_dir: Option<String>,
    #[serde(skip)]
    this: GitSubmissionTool,
}

/// Implements the ToolInstance trait for GitSubmissionTool, allowing it to be used
/// as a dynamic tool
impl ToolInstance for GitSubmissionTool {
    type Args = GitSubmissionToolArgs;
    type Out = String;

    /// Runs the requested git action based on the parameters provided.
    ///
    /// # Parameters
    /// * `params`: a serde_json::Value containing the following keys:
    ///     - "action": String. One of "add", "commit", "push" or "submit".
    ///     - "commit_message": String. Used for "commit" and "submit".
    ///     - "working_dir": String (optional). Overrides the working directory for this invocation.
    /// # Return
    /// Returns a JSON String describing the outcome or an error.
    fn run(params: Self::Args) -> Result<Self::Out, Box<dyn std::error::Error>> {
        // Match the action to the corresponding git operation.
        let tool = if let Some(working_dir) = params.working_dir.clone() {
            GitSubmissionTool::new(working_dir)
        } else {
            params.this // <- we could refactor this
        };

        Ok(match params.action.as_str() {
            "add" => {
                tool.add_changes()?;
                "git add . executed".to_string()
            }
            "commit" => {
                tool.commit_changes(&params.commit_message)?;
                format!(
                    "git commit executed with message: {:?}",
                    params.commit_message
                )
            }
            "push" => {
                tool.push_changes()?;
                "git push origin HEAD executed".to_string()
            }
            "submit" => {
                tool.submit_changes(&params.commit_message)?;
                format!(
                    "submission successful (add, commit, push) with message: {:?}",
                    params.commit_message,
                )
            }
            _ => {
                return Err(
                    "Incorrect action parameter. Allowed are: add, commit, push, submit.".into(),
                );
            }
        })
    }

    /// Returns the tool definition for this tool, including parameters and descriptions.
    fn return_choice() -> openai::Tool {
        openai::Tool::function(
            "git_submission".to_owned(),
            "Executes the full git submission using add, commit and push. Please always provide commit message.".to_owned(),
            HashMap::from([
                (
                    "action".to_owned(),
                    openai::FunctionParameter::new("string", "Please choose one of these actions: \"add\", \"commit\", \"push\" or \"submit\"")
                ),
                (
                    "commit_message".to_owned(),
                    openai::FunctionParameter::new("string", "Commit message, which summarizes the changes made."),
                ),
            ]),
            HashMap::from([
                (
                    "working_dir".to_owned(),
                    openai::FunctionParameter::new("string", "Directory where git commands are executed. Defaults to current directory"),
                )
            ]),
        )
    }
}
