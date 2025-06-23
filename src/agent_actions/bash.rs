use crate::openai;
use crate::tools_interface::ToolInstance;
use std::io;
use std::process::{Command, Stdio};
pub struct BashTool;

impl BashTool {
    pub fn new() -> Self {
        BashTool
    }
}

/// Executes the given bash code and returns the output (stdout and stderr).
/// # Arguments
///
/// * `code` - The bash code to execute.
///
/// # Returns
///
/// Returns a Result containing the combined stdout and stderr output, or an error if execution fails.
pub fn run_bash(code: &str) -> io::Result<String> {
    let output = Command::new("bash")
        .arg("-c")
        .arg(code)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    let mut result = String::new();
    result.push_str(&String::from_utf8_lossy(&output.stdout));
    result.push_str(&String::from_utf8_lossy(&output.stderr));
    Ok(result)
}

impl Default for BashTool {
    fn default() -> Self {
        BashTool::new()
    }
}

impl ToolInstance for BashTool {
    fn run(&self, params: Vec<String>) -> Result<String, Box<dyn std::error::Error>> {
        if params.is_empty() {
            return Err("No bash code provided".into());
        }
        let code = &params[0];
        let output = run_bash(code)?;
        Ok(output)
    }

    fn return_choice() -> openai::Tool {
        openai::Tool {
        function: openai::Function {
            name: "bash".to_string(),
            description: "Executes bash code and returns the output (stdout and stderr). The first parameter is the bash code to execute.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "code": {
                        "type": "string",
                        "description": "The bash code to execute"
                    }
                },
                "required": ["code"]
            }),
        },
        tool_type: "function".to_string(),
    }
    }
}
