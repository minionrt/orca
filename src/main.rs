mod agent_actions;
mod llm;
mod memory;
mod models;
mod openai;
mod task_handler;
mod fetch_task;mod tools;
mod tools_interface;

//use llm::Completion;
use reqwest::blocking::Client;
use std::env;
use task_handler::{Task, TaskHandler, TaskOutcome};
use url::Url;
mod report;
use report::try_report_failure;
mod repo_clone;
use crate::{fetch_task::get_task, repo_clone::GitRepository};

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
    let mut task_handler = TaskHandler::new(&minion_token, &minion_api);

    //Create new Client for get_task
    let client: Client = Client::new();

    //Fetch the raw task data 
    let raw_task = get_task(&minion_api, &minion_token, client);

    let (meta_data, description) = match &raw_task {
        Ok(res) =>{
            //Match the GitRepository data to get important information for repo_clone
            let meta_data = GitRepository::new(
                &res.git_repo_url,
                &res.git_branch, 
                &res.git_user_name, 
                &res.git_user_email, 
                "/github_in_here");
                
            //Match the raw task data to only get the description of the task.
            let description = res.description.to_string();
            (meta_data, description)
        }
        Err(_res) =>{
            //Empty meta_data in case of Error
            let meta_data = GitRepository::new("", "", "", "", "");
            //Clarify to the Model, that there has been an error.
            let description = "There has been an error while receiving the task".to_string();
            (meta_data, description)


        }
    };
    match meta_data.prepare_repository(){
        Ok(()) => println!("Repository prepared successfully"),
        Err(err)=> eprintln!("Preparing repository failed: {err}")
    }

    let path = format!("/n The path to the File you should work on is this one: {}", meta_data.target_dir);
    let task = Task{
        request: description + &path
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
