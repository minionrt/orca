mod llm;
mod memory;
mod models;
mod openai;
mod task_handler;
mod fetch_task;
use llm::Completion;
use reqwest::blocking::Client;
use std::env;
use task_handler::{Task, TaskHandler, TaskOutcome};
use url::Url;
mod report;
use report::try_report_failure;
mod repo_clone;
use crate::fetch_task::get_task;

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

    //Task handler interaction example
    //Let task_handler = TaskHandler::new_set_model(&minion_token, &minion_api, models::Model::Smart);
    let task_handler = TaskHandler::new_set_model(&minion_token, &minion_api, models::Model::Smart);

    //Create new Client for get_task
    let client: Client = Client::new();

    //Fetch the raw task data 
    let raw_task = get_task(&minion_api, &minion_token, client);

    //Match the raw task data to only get the description of the task.
    let task = match raw_task{
        Ok(res) =>  Task { request: res.description.to_string() },
        Err(_res) => Task {request: "There has been an error on passing the task.".to_string()}
    };

    let response = task_handler.run(&task);

    let _response = match response {
        TaskOutcome::Complete(a) => match &a {
            // If the completion is Text, clone the text
            Completion::Text(txt) => txt.clone(),
            // If the completion is ToolCalls, format it as a string
            Completion::ToolCalls(tc) => format!("ToolCalls: {:?}", tc),
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
