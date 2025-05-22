mod llm;
mod openai;

use std::env;

use url::Url;
use crate::llm::MessageRole;

fn main() {
    println!("{:?}", tokio::runtime::Runtime::new().unwrap().block_on(async {
        llm::LLM::new()
            .with_base_url("http://localhost:11434/v1/chat/completions".to_owned())
            .with_api_key("123".to_owned())
            .with_model("deepseek-r1".to_owned())
            .prompt_unwrapped("hello".to_owned(), MessageRole::User)
            .await
    }));

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
