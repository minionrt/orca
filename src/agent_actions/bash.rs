use crate::openai;
use crate::tools_interface::ToolInstance;
use serde::Deserialize;
use std::collections::HashMap;
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

#[derive(Deserialize)]
pub(crate) struct BashToolArgs {
    pub code: String,
}

impl ToolInstance for BashTool {
    type Args = BashToolArgs;
    type Out = String;
    fn run(args: Self::Args) -> Result<Self::Out, Box<dyn std::error::Error>> {
        Ok(run_bash(&args.code)?)
    }

    fn return_choice() -> openai::Tool {
        openai::Tool::function("bash".to_owned(),
            "Executes bash code and returns the output (stdout and stderr). The first parameter is the bash code to execute.".to_owned(),
           HashMap::from([
               ("code".to_owned(), openai::FunctionParameter::new("string", "The bash code to execute")),
           ]),
           HashMap::new(),
        )
    }
}
