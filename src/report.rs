#![allow(dead_code)] 
use std::fmt;
use url::Url;

/// Represents the reason for a task failure as reported to the API.
#[derive(Debug)]
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
        write!(f, "{}", s)
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
) -> Result<reqwest::blocking::Response, reqwest::Error> {
    reqwest::blocking::Client::new()
        .post(minion_api.join("agent/task/complete").unwrap())
        .bearer_auth(minion_token)
        .header("Content-Type", "application/json")
        .body(format!("{{\"description\": \"{}\"}}", description))
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
) -> Result<reqwest::blocking::Response, reqwest::Error> {
    let body = match reason {
        Some(r) => format!(
            "{{\"reason\": \"{}\", \"description\": \"{}\"}}",
            r, description
        ),
        None => format!("{{\"description\": \"{}\"}}", description),
    };

    reqwest::blocking::Client::new()
        .post(minion_api.join("agent/task/fail").unwrap())
        .bearer_auth(minion_token)
        .header("Content-Type", "application/json")
        .body(body)
        .send()
}
