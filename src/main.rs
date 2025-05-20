use std::env;
use url::Url;
mod report;
use report::{report_failure, TaskFailureReason};

fn main() {
    println!();
    println!();
    println!("This is a message from the agent.");
    println!();
    println!();
    // The agent receives the HTTP API base url and token via the following environment variables.
    // See https://github.com/autominion/spec/blob/main/spec/runtime.md
    let minion_api: Url = env::var("MINION_API_BASE_URL").unwrap().parse().unwrap();
    let minion_token = env::var("MINION_API_TOKEN").unwrap();
    // The agent uses an HTTP API to fetch the task and report the result.
    // See https://github.com/autominion/spec/blob/main/spec/http.md
    //
    // This will exit the `minion` CLI.
   report_failure(minion_api,   minion_token,"Not implemented yet", Some(TaskFailureReason::TechnicalIssues));
}
