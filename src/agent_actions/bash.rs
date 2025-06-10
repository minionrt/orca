use std::io;
use std::process::{Command, Stdio};

/// Executes the given bash code and returns the output (stdout and stderr).
///
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
