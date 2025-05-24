mod llm;
mod openai;
mod task_handler;

use std::env;

use task_handler::{Task, TaskHandler, TaskOutcome};
use url::Url;

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

    let task_handler = TaskHandler::new(&minion_token, &minion_api);
    let task = Task{ 
        request: "Please write a simple FizzBuzz program.".to_string()
    };
    let response = task_handler.run(&task);
    let response = match response {
        TaskOutcome::Complete(a) => a[1].content.clone(),
        TaskOutcome::Failure => "didn't work, sorry".to_string(),
    };

    // The agent uses an HTTP API to fetch the task and report the result.
    // See https://github.com/autominion/spec/blob/main/spec/http.md
    //
    // Since nothing is implemented yet, we just mark the task as failed.
    // This will exit the `minion` CLI.
    reqwest::blocking::Client::new()
        .post(minion_api.join("agent/task/fail").unwrap())
        .bearer_auth(minion_token)
        .header("Content-Type", "application/json")
        .body(r#"{"description": "Not yet implemented"}"#)
        .send()
        .unwrap();
}
