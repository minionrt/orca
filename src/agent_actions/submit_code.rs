use crate::openai;
use crate::tools_interface::ToolInstance;
use serde::Deserialize;
use std::collections::HashMap;
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
        let status = Command::new("git")
            .arg("add")
            .arg(".")
            .current_dir("/github_in_here") //temporary workaround! TODO!
            .status()?;
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
    /// # Return
    /// Returns an io::Result indicating either success or failure
    fn commit_changes(commit_message: Option<String>) -> io::Result<()> {
        let c_message = commit_message.unwrap_or("No message :(".to_owned());
        let status = Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg(c_message)
            .current_dir("/github_in_here") //temporary workaround! TODO!
            .status()?;
        if !status.success() {
            return Err(io::Error::other("git commit failed"));
        }
        Ok(())
    }

    /// all committed changes will be pushed
    ///
    /// # Return
    /// Returns an io::Result indicating success or failure
    fn push_changes() -> io::Result<()> {
        let status = Command::new("git")
            .arg("push")
            .arg("origin")
            .arg("HEAD")
            .current_dir("/github_in_here") //temporary workaround! TODO!
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
    /// # Return
    /// Returns an io::Result indicating either success or failure
    fn submit_changes(commit_message: Option<String>) -> io::Result<()> {
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

#[derive(Deserialize)]
pub struct GitSubmissionToolArgs {
    action: String,
    #[serde(default)]
    commit_message: Option<String>,
}

/// Implements the ToolInstance trait for GitSubmissionTool, allowing it to be used
/// as a dynamic tool
impl ToolInstance for GitSubmissionTool {
    type Args = GitSubmissionToolArgs;
    type Out = String;

    /// runs the requested git action based in the parameters provided.
    ///
    /// # Parameters
    /// * `params`: a serde::json::Value containing the following keys:
    ///     - "action": String. One of "add", "commit", "push" or "submit".
    ///     - "commit_message": String (technically optional, but highly encuraged). Used for "commit" and "submit".
    /// # Return
    /// Returns a JSON String describing the outcome or an error.
    fn run(params: Self::Args) -> Result<Self::Out, Box<dyn std::error::Error>> {
        // Match the action to the corresponding git operation.
        let commit_message = params
            .commit_message
            .clone()
            .unwrap_or("No message :(".to_owned());
        Ok(match params.action.as_str() {
            "add" => {
                Self::add_changes()?;
                "git add . executed".to_string()
            }
            "commit" => {
                Self::commit_changes(params.commit_message)?;
                format!("git commit executed with message: {:?}", commit_message,)
            }
            "push" => {
                Self::push_changes()?;
                "git push origin HEAD executed".to_string()
            }
            "submit" => {
                Self::submit_changes(params.commit_message)?;
                format!(
                    "submission successful (add, commit, push) with message: {:?}",
                    commit_message,
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
                )
            ]),
            HashMap::from([
                (
                    "commit_message".to_owned(),
                    openai::FunctionParameter::new("string", "Commit message, which summarizes the changes made."),
                )
            ]),
        )
    }
}
