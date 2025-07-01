#![allow(dead_code)]
use reqwest::blocking::Client;
use serde::Serialize;
use serde_json::json;
use std::fmt;
use std::io::{self, Write};
use url::Url;

/// Represents the reason for a task failure as reported to the API.
#[derive(Debug, Serialize)]
pub enum TaskFailureReason {
    /// The agent failed due to technical problems unrelated to the task itself.
    TechnicalIssues,
    /// The agent failed due to a problem with the task itself (e.g., unclear or impossible).
    TaskIssues,
    /// The agent failed due to not succeeding at task-specific problem-solving.
    ProblemSolving,
}

impl fmt::Display for TaskFailureReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            TaskFailureReason::TechnicalIssues => "TechnicalIssues",
            TaskFailureReason::TaskIssues => "TaskIssues",
            TaskFailureReason::ProblemSolving => "ProblemSolving",
        };
        write!(f, "{s}")
    }
}

/// Reports successful completion of a task to the API.
///
/// # Arguments
///
/// * `minion_api` - The base URL of the minion API.
/// * `minion_token` - The authentication token for the minion.
/// * `description` - A natural-language description of the result.
///
/// # Returns
///
/// A Result with the Response or an Error
///
/// # Panics
///
/// Panics if the API Url join does not succeed
pub fn report_success(
    minion_api: Url,
    minion_token: String,
    description: &str,
    client: Client,
) -> Result<reqwest::blocking::Response, reqwest::Error> {
    let body = json!({ "description": description }).to_string();
    client
        .post(minion_api.join("agent/task/complete").unwrap())
        .bearer_auth(minion_token)
        .header("Content-Type", "application/json")
        .body(body)
        .send()
}

/// Reports a task failure to the API.
///
/// # Arguments
///
/// * `minion_api` - The base URL of the minion API.
/// * `minion_token` - The authentication token for the minion.
/// * `description` - A natural-language description of the task failure.
/// * `reason` - Optional reason for the task failure (`TaskFailureReason`).
///
/// # Returns
///
/// A Result with the Response or an Error
///
/// # Panics
///
/// Panics if the API Url join does not succeed
pub fn report_failure(
    minion_api: Url,
    minion_token: String,
    description: &str,
    reason: Option<TaskFailureReason>,
    client: Client,
) -> Result<reqwest::blocking::Response, reqwest::Error> {
    let body = match reason {
        Some(r) => json!({
            "reason": r.to_string(),
            "description": description
        })
        .to_string(),
        None => json!({
            "description": description
        })
        .to_string(),
    };

    client
        .post(minion_api.join("agent/task/fail").unwrap())
        .bearer_auth(minion_token)
        .header("Content-Type", "application/json")
        .body(body)
        .send()
}

/// Wrapper for reporting success, prints error to stderr if the API call fails.
pub fn try_report_success(
    minion_api: Url,
    minion_token: String,
    description: &str,
    client: Client,
) -> bool {
    match report_success(minion_api, minion_token, description, client) {
        Ok(_) => true,
        Err(e) => {
            let _ = writeln!(io::stderr(), "Failed to report success: {e}");
            false
        }
    }
}

/// Wrapper for reporting failure, prints error to stderr if the API call fails.
pub fn try_report_failure(
    minion_api: Url,
    minion_token: String,
    description: &str,
    reason: Option<TaskFailureReason>,
    client: Client,
) -> bool {
    match report_failure(minion_api, minion_token, description, reason, client) {
        Ok(_) => true,
        Err(e) => {
            let _ = writeln!(io::stderr(), "Failed to report failure: {e}");
            false
        }
    }
}
