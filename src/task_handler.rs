#![allow(dead_code)]

use url::Url;

use crate::llm::{Completion, LLM, Message, MessageRole};
use crate::models::Model;

const INTRO_1: &str = r#"You are an autonomous agent that solves coding tasks. 
You keep your explanations as concise as possible.
You are connected to a Linux-based development environment. You are in the 
project directory. Your current task is as follows:"#;

pub enum TaskOutcome {
    Complete(Completion),
    Failure,
}

pub struct Task {
    pub request: String,
}

/// Instance of TaskHandler contains the functions new and run.
/// * `new()` expects an api_key, base_url and a model -> returns TaskHandler
/// * `new_set_model()` like new(), but also expects a model
/// * `run()` handles interaction with LLM and expects a Task
pub struct TaskHandler {
    llm: LLM,
}

/// Responsible to handle the interaction with the LLM
/// Use for sending tasks to llm and so interaction
impl TaskHandler {
    pub fn new(api_key: &str, base_url: &Url) -> Self {
        TaskHandler {
            //llm: LLM::full(api_key.to_string(), base_url.clone(), Model::Basic.into()),
            llm: LLM::full(
                api_key.to_string(),
                base_url.clone(),
                Model::Smart.into(),
                None,
                None,
            ),
        }
    }

    pub fn new_set_model(api_key: &str, base_url: &Url, model: Model) -> Self {
        TaskHandler {
            llm: LLM::full(
                api_key.to_string(),
                base_url.clone(),
                String::from(model),
                None,
                None,
            ),
        }
    }
    pub fn new_set_model_and_tools(
        api_key: &str,
        base_url: &Url,
        model: Model,
        tools: Option<Vec<crate::openai::Tool>>,
        tool_choice: Option<crate::openai::ToolChoice>,
    ) -> Self {
        TaskHandler {
            llm: LLM::full(
                api_key.to_string(),
                base_url.clone(),
                String::from(model),
                tools,
                tool_choice,
            ),
        }
    }

    /// runs interaction with the llm for a given task
    pub fn run(&self, task: &Task) -> TaskOutcome {
        //we might have to add a container here if we want to use the result

        //we cann add interaction loop here @Frodo
        self.single_request(&task.request)
    }

    /// send single code task request without memory
    fn single_request(&self, request: &str) -> TaskOutcome {
        let history = "no history".to_string();
        self.send_request(request, &history)
    }
    /// send code task request with memory - meant for longer interaction loops
    fn send_request(&self, request: &str, history: &str) -> TaskOutcome {
        //dev message so user cannot mess with LLM
        let dev_message = Message::new(INTRO_1.to_string(), MessageRole::Developer);
        //history posted as Assistant to make the LLM know what happened before
        let history_message = Message::new(history.to_string(), MessageRole::Assistant);
        //build user request as Message
        let user_message = Message::new(request.to_string(), MessageRole::User);
        let messages = vec![dev_message, history_message, user_message];

        //send all the messages to the LLM and take result
        let response = self.llm.prompt(&messages);

        match response {
            Ok(r) => TaskOutcome::Complete(r),
            Err(_) => TaskOutcome::Failure,
        }
    }
}
