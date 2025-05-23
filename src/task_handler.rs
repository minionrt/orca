
use crate::llm::{LLM, Message, MessageRole};

const SMART_MODEL: &str = "o1-mini";
const BASIC_MODEL: &str = "gpt-4o-mini";
const TEST_MODEL: &str = "test-model"; //no functionality yet
pub enum Model{
    SmartModel,
    BasicModel,
    TestModel
}

impl From<Model> for String {
    fn from(m: Model) -> String {
        match m {
            Model::SmartModel => SMART_MODEL.to_string(),
            Model::BasicModel => BASIC_MODEL.to_string(),
            Model::TestModel => TEST_MODEL.to_string()
        }
    }
}

const INTRO_1: &str = r#"You are an autonomous agent that solves coding tasks. You keep your explanations as concise as possible.
You are connected to a Linux-based development environment. You are in the project directory. Your current task is as follows:"#;

pub enum TaskOutcome {
    Complete(String),
    Failure
}

pub struct Task{
    request: String,
}

/** Instance of TaskHandler contains the functions new and run.
* `new()` expects an api_key, base_url and a model -> returns TaskHandler
* `run()` handles interaction with LLM and expects a Task
*/
pub struct TaskHandler{
    llm: LLM
}

/** Responsible to handle the interaction with the LLM
 *  Use for sending tasks to llm and so interaction
 */
impl TaskHandler{
    pub fn new(api_key: String, base_url: String) ->  Self{
        TaskHandler { llm: LLM::full(api_key, base_url, BASIC_MODEL.to_string())}
    }

    pub fn new__set_model(api_key: String, base_url: String, model: Model) ->  Self{
        TaskHandler { llm: LLM::full(api_key, base_url, String::from(model))}
    }

    /// runs interaction with the llm for a given task
    pub fn run(&self, task: &Task) -> TaskOutcome{ //we might have to add a container here if we want to use the result

        //we cann add interaction loop here @Frodo
        self.single_request(&task.request) 
    }
    
    /// send single code task request without memory
    fn single_request(&self, request: &String) -> TaskOutcome{
        let history = "".to_string();
        self.send_request(&request, &history)
    }
    /// send code task request with memory - meant for longer interaction loops
    fn send_request(&self, request: &String, history: &String) -> TaskOutcome{
        //dev message so user cannot mess with LLM
        let dev_message = Message::new(INTRO_1.to_string(), MessageRole::Developer);
        //history posted as Assistant to make the LLM know what happened before
        let history_message = Message::new(history.clone(), MessageRole::Assistant);
        //build user request as Message
        let user_message = Message::new(request.clone(), MessageRole::User);
        let messages = vec![dev_message, history_message, user_message];

        //send all the messages to the LLM and take result
        let response = self.llm.prompt(&messages);
        let response = match response {
            Ok(r) => TaskOutcome::Complete(r.completions),
            Err(_) => TaskOutcome::Failure
        }
        response
    }

}