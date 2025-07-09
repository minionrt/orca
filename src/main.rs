mod agent_actions;
mod fetch_task;
mod llm;
mod memory;
mod models;
mod openai;
mod report;
mod task_handler;
mod tools;
mod tools_interface;

//use llm::Completion;
use report::{try_report_failure, try_report_success};
use reqwest::blocking::Client;
use std::env;
use task_handler::{Task, TaskHandler, TaskOutcome};
use url::Url;
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

    let http_client = reqwest::blocking::Client::new();

    //Fetch the raw task data
    let raw_task = get_task(&minion_api, &minion_token, client);

    let (meta_data, description) = match &raw_task {
        Ok(res) => {
            //Change Url for neccessary authing.
            let mut repo_url = res.git_repo_url.clone();
            match Url::parse(&repo_url) {
                Ok(mut url) => {
                    url.set_username("x-access-token").unwrap();
                    url.set_password(Some(minion_token.as_str())).unwrap();
                    repo_url = url.to_string();
                }
                Err(e) => {
                    eprint!("No Valid URL: {e}")
                }
            };
            //Match the GitRepository data to get important information for repo_clone
            let git_data = GitRepository::new(
                &repo_url,
                &res.git_branch,
                &res.git_user_name,
                &res.git_user_email,
                "/workspace",
            );

            //Match the raw task data to only get the description of the task.
            let description = res.description.to_string();
            (git_data, description)
        }
        Err(_res) => {
            //Empty meta_data in case of Error
            //let meta_data = GitRepository::new("", "", "", "", "");
            //Clarify to the Model, that there has been an error.
            //let description = "There has been an error while receiving the task".to_string();
            //(meta_data, description)
            try_report_failure(
                minion_api,
                minion_token,
                "There has been an error while receiving the task",
                Some(report::TaskFailureReason::TaskIssues),
                http_client,
            );
            return;
        }
    };

    // clone git repo
    match meta_data.prepare_repository() {
        Ok(()) => println!("Repository prepared successfully"),
        Err(err) => eprintln!("Preparing repository failed: {err}"),
    }

    // The target_dir is the directory that should be used as the working directory for all tools that interact with the repository
    let working_dir = meta_data.target_dir.clone();

    // No need to add path info to the user prompt; instead, pass it to the dev prompt via Task
    let task = Task {
        request: description,
        working_dir: working_dir.clone(),
    };

    let response = task_handler.run(&task);
    //let response = TaskOutcome::Complete("Test".to_string());  //you can use that if you just want to test the lifecycle

    /*let _response = match response {
        TaskOutcome::Complete(a) => a,
        TaskOutcome::Failure(_, _) => "didn't work, sorry".to_string(),
    };*/

    match response {
        TaskOutcome::Complete(a) => try_report_success(minion_api, minion_token, &a, http_client),
        TaskOutcome::Failure(v, r) => {
            try_report_failure(minion_api, minion_token, &v, r, http_client)
        }
    };
}
mod agent_actions;
mod fetch_task;
mod llm;
mod memory;
mod models;
mod openai;
mod report;
mod task_handler;
mod tools;
mod tools_interface;

//use llm::Completion;
use report::{try_report_failure, try_report_success};
use reqwest::blocking::Client;
use std::env;
use task_handler::{Task, TaskHandler, TaskOutcome};
use url::Url;
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

    let http_client = reqwest::blocking::Client::new();

    //Fetch the raw task data
    let raw_task = get_task(&minion_api, &minion_token, client);

    let (meta_data, description) = match &raw_task {
        Ok(res) => {
            //Change Url for neccessary authing.
            let mut repo_url = res.git_repo_url.clone();
            match Url::parse(&repo_url) {
                Ok(mut url) => {
                    url.set_username("x-access-token").unwrap();
                    url.set_password(Some(minion_token.as_str())).unwrap();
                    repo_url = url.to_string();
                }
                Err(e) => {
                    eprint!("No Valid URL: {e}")
                }
            };
            //Match the GitRepository data to get important information for repo_clone
            let git_data = GitRepository::new(
                &repo_url,
                &res.git_branch,
                &res.git_user_name,
                &res.git_user_email,
                "/workspace",
            );

            //Match the raw task data to only get the description of the task.
            let description = res.description.to_string();
            (git_data, description)
        }
        Err(_res) => {
            //Empty meta_data in case of Error
            //let meta_data = GitRepository::new("", "", "", "", "");
            //Clarify to the Model, that there has been an error.
            //let description = "There has been an error while receiving the task".to_string();
            //(meta_data, description)
            try_report_failure(
                minion_api,
                minion_token,
                "There has been an error while receiving the task",
                Some(report::TaskFailureReason::TaskIssues),
                http_client,
            );
            return;
        }
    };

    // clone git repo
    match meta_data.prepare_repository() {
        Ok(()) => println!("Repository prepared successfully"),
        Err(err) => eprintln!("Preparing repository failed: {err}"),
    }

    // The target_dir is the directory that should be used as the working directory for all tools that interact with the repository
    let working_dir = meta_data.target_dir.clone();

    // No need to add path info to the user prompt; instead, pass it to the dev prompt via Task
    let task = Task {
        request: description,
        working_dir: working_dir.clone(),
    };

    let response = task_handler.run(&task);
    //let response = TaskOutcome::Complete("Test".to_string());  //you can use that if you just want to test the lifecycle

    /*let _response = match response {
        TaskOutcome::Complete(a) => a,
        TaskOutcome::Failure(_, _) => "didn't work, sorry".to_string(),
    };*/

    match response {
        TaskOutcome::Complete(a) => try_report_success(minion_api, minion_token, &a, http_client),
        TaskOutcome::Failure(v, r) => {
            try_report_failure(minion_api, minion_token, &v, r, http_client)
        }
    };
}
