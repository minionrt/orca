use crate::llm::{LLM, Message, MessageRole};

const SMART_MODEL: &str = "o1-mini";
const BASIC_MODEL: &str = "gpt-4o-mini";
const TEST_MODEL: &str = "test-model"; //no functionality yet

const INTRO_1: &str = r#"You are an autonomous agent that solves coding tasks. You keep your explanations as concise as possible.
You are connected to a Linux-based development environment. You are in the project directory. Your current task is as follows:"#;

pub enum TaskOutcome {
    Complete(String),
    Failure
}

pub struct Task{
    request: String,
}

pub struct TaskHandler{
    llm: LLM
}

//! Instance of TaskHandler contains the functions new and run.
//! `new()` expects an api_key, base_url and a model -> returns TaskHandler
//! `run()` handles interaction with LLM and expects a Task
impl TaskHandler{
    pub fn new(mut self, api_key: String, base_url: String) ->  Self{
        self.llm = LLM::full(api_key, base_url, BASIC_MODEL.to_string());
        self
    }

    //! expects a &Task and returns a TaskOutcome
    pub fn run(task: &Task) -> TaskOutcome{ //we might have to add a container here if we want to use the result

    let mut history = "";
    
    single_request(task.request)
    }

    fn single_request(&self, request: String) -> TaskOutcome{
        //dev message so user cannot mess with LLM
        let dev_message = Message::new(INTRO_1.to_string(), MessageRole::Developer);
        let user_message = Message::new(request, MessageRole::User);
        let messages = vec![dev_message, user_message];

        let response = self.llm.prompt(&messages).await;
        let response = match response {
            Ok(r) => TaskOutcome::Complete(r.completions),
            Err(_) => TaskOutcome::Failure
        }
        response
    }
    /*fn fetch_task() -> Task{

    }*/
}