use crate::openai;
use crate::tools_interface::ToolInstance;
use std::io;
use std::process::{Command, Stdio};
use tracing::{debug, info, warn};
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
    debug!("Executing bash command: {}", code);

    let output = Command::new("bash")
        .arg("-c")
        .arg(code)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !stdout.is_empty() {
        debug!("Command stdout: {}", stdout);
    }
    if !stderr.is_empty() {
        warn!("Command stderr: {}", stderr);
    }

    let mut result = String::new();
    result.push_str(&stdout);
    result.push_str(&stderr);

    info!("Bash command completed with exit status: {}", output.status);
    Ok(result)
}

impl Default for BashTool {
    fn default() -> Self {
        BashTool::new()
    }
}

impl ToolInstance for BashTool {
    fn run(
        &self,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        debug!("BashTool::run called with params: {:?}", params);

        let code = match params.get("code") {
            None => {
                warn!("Missing 'code' parameter in bash tool call");
                return Err("The parameter \"code\" doesn't exist in the given tool call".into());
            }
            Some(serde_json::Value::String(s)) => s,
            Some(_) => {
                warn!("'code' parameter is not a string");
                return Err("The parameter \"code\" isn't given as string.".into());
            }
        };

        info!("Running bash tool with code: {}", code);
        let output = run_bash(code)?;
        Ok(serde_json::Value::String(output))
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
