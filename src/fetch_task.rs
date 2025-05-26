#![allow(dead_code)]
use serde::Deserialize;
use url::Url;

/// Represents the status of a task as returned by the API.
#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum TaskStatus {
    Queued,
    Running,
    Completed,
    Failed,
}

/// Represents the response from the GET /agent/task endpoint.
#[derive(Debug, Deserialize)]
pub struct TaskResponse {
    pub status: TaskStatus,
    pub description: String,
    pub git_user_name: String,
    pub git_user_email: String,
    pub git_repo_url: String,
    pub git_branch: String,
}

/// Fetches the current task from the API.
///
/// # Arguments
///
/// * `minion_api` - The base URL of the minion API.
/// * `minion_token` - The authentication token for the minion.
///
/// # Returns
///
/// A Result containing the `TaskResponse` or a `reqwest::Error`.
pub fn get_task(
    minion_api: Url,
    minion_token: String,
) -> Result<TaskResponse, reqwest::Error> {
    let url = minion_api.join("agent/task").unwrap();
    let response = reqwest::blocking::Client::new()
        .get(url)
        .bearer_auth(minion_token)
        .send()?
        .error_for_status()?
        .json::<TaskResponse>()?;
    Ok(response)
}