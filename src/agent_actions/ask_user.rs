use crate::openai;
use crate::tools_interface::ToolInstance;
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use tracing::{debug, warn};
use url::Url;

pub struct AskUserTool;

impl AskUserTool {
    pub fn new() -> Self {
        AskUserTool
    }
}

/// Sends the given inquiry to the CLI.
/// Once its processed by the CLI it will return a String.
/// # Arguments
///
/// * `inquiry` - The clarification request sent to the user.
///
/// # Returns
///
/// Returns a String containing a clarification.
pub fn ask_user(inquiry: &str) -> String {
    let client = reqwest::blocking::Client::new();
    let minion_api: Url = env::var("MINION_API_BASE_URL").unwrap().parse().unwrap();
    let minion_token = env::var("MINION_API_TOKEN").unwrap();
    let url: Url = minion_api.join("agent/inquiry").unwrap();
    debug!("Sending inquiry to URL: {}", url);

    let response = client
        .post(url)
        .bearer_auth(minion_token.clone())
        .json(&serde_json::json!({"inquiry": inquiry}))
        .send();

    match response {
        Ok(resp) => {
            if !resp.status().is_success() {
                warn!("Received error status from inquries endpoint: {}", resp.status());
                return "[ERROR] Received non-success status code".to_string();
            }
            match resp.text() {
                Ok(text) => text,
                Err(e) => {
                    warn!("Could not read response body: {}", e);
                    "[ERROR] Could not read response body".to_string()
                }
            }
        }
        Err(e) => {
            warn!("Could not contact inquiries endpoint: {}", e);
            "[ERROR] Could not contact inquiries endpoint".to_string()
        }
    }
}

impl Default for AskUserTool {
    fn default() -> Self {
        AskUserTool::new()
    }
}

#[derive(Deserialize)]
pub struct AskUserArgs {
    pub inquiry: String,
}

impl ToolInstance for AskUserTool {
    type Args = AskUserArgs;
    type Out = String;

    fn run(input: Self::Args) -> Result<Self::Out, Box<dyn std::error::Error>> {
        debug!("AskUserTool::run called with code: {}", input.inquiry);
        let output = ask_user(&input.inquiry);
        Ok(output)
    }
    fn return_choice() -> openai::Tool {
        openai::Tool::function(
            "ask_user".to_owned(),
            "Ask the user a clarifying question".to_owned(),
            HashMap::from([(
                "inquiry".to_owned(),
                openai::FunctionParameter::new("string", "the question to present to the user."),
            )]),
            HashMap::new(),
        )
    }
}
