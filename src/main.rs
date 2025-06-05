mod llm;
mod models;
mod openai;
mod task_handler;

use std::env;
use task_handler::{Task, TaskHandler, TaskOutcome};
use llm::CompletionKind;
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
    let task_handler =
        TaskHandler::new_set_model(&minion_token, &minion_api, models::Model::Gemini);
    let task = Task {
        request: "Please write a simple FizzBuzz program.".to_string(),
    };
    let response = task_handler.run(&task);

    let _response = match response {
    TaskOutcome::Complete(a) => match &a[1].kind {
        // If the completion kind is Text, clone the text
        CompletionKind::Text(txt) => txt.clone(),
        // If the completion kind is ToolCalls, format it as a string
        CompletionKind::ToolCalls(tc) => format!("ToolCalls: {:?}", tc),
    },
    TaskOutcome::Failure => "didn't work, sorry".to_string(),
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
