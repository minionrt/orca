mod agent_actions;
mod llm;
mod memory;
mod models;
mod openai;
mod task_handler;
mod tools;
mod tools_interface;

//use llm::Completion;
use std::env;
use task_handler::{Task, TaskHandler, TaskOutcome};
use url::Url;
mod report;
use report::try_report_failure;

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

    //task handler interaction example
    //let task_handler = TaskHandler::new_set_model(&minion_token, &minion_api, models::Model::Smart);
    let mut task_handler =
        TaskHandler::new(&minion_token, &minion_api);
    let task = Task {
        request: "Please write a simple FizzBuzz program.".to_string(),
    };
    let response = task_handler.run(&task);

    let _response = match response {
        TaskOutcome::Complete(a) => a,
        TaskOutcome::Failure(_, _) => "didn't work, sorry".to_string(),
    };

    // The agent uses an HTTP API to fetch the task and report the result.
    // See https://github.com/autominion/spec/blob/main/spec/http.md
    //
    // This will exit the `minion` CLI.

    let client = reqwest::blocking::Client::new();
    try_report_failure(
        minion_api,
        minion_token,
        "Not implemented yet",
        None,
        client,
    );
}
