use crate::openai;
use crate::tools_interface::ToolInstance;
use std::io;
use std::process::Command;
use serde_json::Value;

pub struct GitSubmissionTool;

impl GitSubmissionTool {
    pub fn new() -> Self {
        GitSubmissionTool
    }

    fn add_changes() -> io::Result<()> {
        let status = Command::new("git")
            .arg("add")
            .arg(".")
            .status()?;
        if !status.success() {
            return Err(io::Error::new(io::ErrorKind::Other, "git add failed"));
        }
        Ok(())
    }

    fn commit_changes(commit_message: Option<&str>) -> io::Result<()> {
        let c_message = commit_message.unwrap_or("No message :(");
        let status = Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg(c_message)
            .status()?;
        if !status.success() {
            return Err(io::Error::new(io::ErrorKind::Other, "git commit failed"));
        }
        Ok(())
    }

    fn push_changes() -> io::Result<()> {
        let status = Command::new("git")
            .arg("push")
            .arg("origin")
            .arg("HEAD")
            .status()?;
        if !status.success() {
            return Err(io::Error::new(io::ErrorKind::Other, "git push failed"));
        }
        Ok(())
    }

    fn submit_changes(commit_message: Option<&str>) -> io::Result<()> {
        Self::add_changes()?;
        Self::commit_changes(commit_message)?;
        Self::push_changes()?;
        Ok(())
    }
}

impl Default for GitSubmissionTool {
    fn default() -> Self {
        GitSubmissionTool::new()
    }
}

impl ToolInstance for GitSubmissionTool {
    fn run(
        &self,
        params: Value,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        let action = params.get("action")
            .and_then(|v| v.as_str())
            .ok_or("Missing parameter: action")?;
        let commit_message = params.get("commit_message").and_then(|v| v.as_str());

        let result = match action {
            "add" => {
                Self::add_changes()?;
                "git add . executed".to_string()
            },
            "commit" => {
                Self::commit_changes(commit_message)?;
                format!("git commit executed with message: {:?}", commit_message.unwrap_or("No message :("))
            },
            "push" => {
                Self::push_changes()?;
                "git push origin HEAD executed".to_string()
            },
            "submit" => {
                Self::submit_changes(commit_message)?;
                format!("submission successful (add, commit, push) with message: {:?}", commit_message.unwrap_or("No message :("))
            },
            _ => return Err("Incorrect action parameter. Allowed are: add, commit, push, submit.".into())
        };
        Ok(Value::String(result))
    }

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