use crate::openai;
use crate::tools_interface::ToolInstance;
use serde::Deserialize;
use std::collections::HashMap;
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
/// * `working_dir` - The working directory
///
/// # Returns
///
/// Returns a Result containing the combined stdout and stderr output, or an error if execution fails.
pub fn run_bash(code: &str, working_dir: &str) -> io::Result<String> {
    debug!("Executing bash command: {}", code);

    let output = Command::new("bash")
        .arg("-c")
        .arg(code)
        .current_dir(working_dir)
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

#[derive(Deserialize)]
pub struct BashToolArgs {
    pub code: String,
    pub working_dir: String,
}

impl ToolInstance for BashTool {
    type Args = BashToolArgs;
    type Out = String;

    fn run(args: Self::Args) -> Result<Self::Out, Box<dyn std::error::Error>> {
        debug!("BashTool::run called with code: {}", args.code);
        let output = run_bash(&args.code, &args.working_dir)?;
        Ok(output)
    }

    fn return_choice() -> openai::Tool {
        openai::Tool::function(
            "bash".to_owned(),
            "Executes bash code and returns the output (stdout and stderr). The first parameter is the bash code to execute.".to_owned(),
            HashMap::from([
                ("code".to_owned(), openai::FunctionParameter::new("string", "The bash code to execute")),
                ("working_dir".to_owned(), openai::FunctionParameter::new("string", "The working directory the tool should work in."))
            ]),
            HashMap::new(),
        )
    }
}
