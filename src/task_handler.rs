#![allow(dead_code)]

use url::Url;

use crate::llm::{Completion, LLM, LLMAPIError, Message, MessageRole};
use crate::memory::Memory;
use crate::models::Model;
use crate::openai::ToolCall;
use crate::report::TaskFailureReason;
use crate::tools::collection::{self, get_tools};

const INTRO_1: &str = r#"You are an autonomous agent that solves coding tasks. 
You should use the given tools to solve the given task.
You are connected to a Linux-based development environment. You are in the 
project directory. Your current task is as follows:"#;

const MESSAGE_TOOL_RESPONSE: &str = r#"You are an autonomous agent that solves coding tasks. 
You should use the given tools to solve the given task.
You are connected to a Linux-based development environment. You are in the 
project directory. If you think your task is done please don't call a tool and just tell me what you did.
The response of your last tool call is the following:"#;

pub enum TaskOutcome {
    Complete(String),
    Failure(String, Option<TaskFailureReason>), //one field for a description, one for a reason
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
    memory: Memory,
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
                get_tools(),
                None,
            ),
            memory: Memory::new(api_key, base_url),
        }
    }

    pub fn new_set_model(api_key: &str, base_url: &Url, model: Model) -> Self {
        TaskHandler {
            llm: LLM::full(
                api_key.to_string(),
                base_url.clone(),
                String::from(model),
                get_tools(),
                None,
            ),
            memory: Memory::new(api_key, base_url),
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
            memory: Memory::new(api_key, base_url),
        }
    }

    /// runs interaction with the llm for a given task
    pub fn run(&mut self, task: &Task) -> TaskOutcome {
        //we might have to add a container here if we want to use the result

        let mut input = task.request.clone();
        let mut response = self.single_request(&task.request);

        let mut ctr: i8 = 0;

        loop {
            //match response, if there was an error, propagate to user
            let completion = match &response {
                Ok(c) => c.clone(),
                Err(e) => {
                    return TaskOutcome::Failure(
                        e.to_string(),
                        Some(TaskFailureReason::TechnicalIssues),
                    );
                }
            };

            //if llm returns a text I expect the task to be done
            if let Completion::Text(value) = completion {
                return TaskOutcome::Complete(value);
            } else if let Completion::ToolCalls(value) = completion {
                //if LLM returns a tool call, extract the tool name and arguments and call tool
                let tool_name = TaskHandler::get_tool_name(&value[0]);
                let args = TaskHandler::get_tool_arguments(&value[0]);
                let tool_result = collection::call_tool(&tool_name, args.clone());

                //add the new interaction to the memory
                self.memory.add(
                    input,
                    format!("You called the tool \"{tool_name}\" with the Arguments: {args}"),
                );

                //make string from tool return
                input = match &tool_result {
                    Ok(value) => value.clone().to_string(),
                    Err(e) => e.to_string(),
                };

                //give returned value of the tool to the llm
                response = self.send_tool_answer(&input, self.memory.read());
            }

            //stop the loop after x runs
            ctr += 1;
            if ctr >= 10 {
                return TaskOutcome::Failure(
                    "The interaction loop run too long, the agent can't top yappin..".to_string(),
                    Some(TaskFailureReason::ProblemSolving),
                );
            }
        }
    }

    /// send single code task request without memory
    fn single_request(&self, request: &str) -> Result<Completion, LLMAPIError> {
        let history = self.memory.read();
        self.send_request(request, history)
    }
    /// send code task request with memory - meant for longer interaction loops
    fn send_request(&self, request: &str, history: &str) -> Result<Completion, LLMAPIError> {
        //dev message so user cannot mess with LLM
        let dev_message = Message::new(INTRO_1.to_string(), MessageRole::Developer);
        //history posted as Assistant to make the LLM know what happened before
        let history_message = Message::new(history.to_string(), MessageRole::Assistant);
        //build user request as Message
        let user_message = Message::new(request.to_string(), MessageRole::User);
        let messages = vec![dev_message, history_message, user_message];

        //send all the messages to the LLM and take result
        self.llm.prompt(&messages)
    }

    fn send_tool_answer(&self, request: &str, history: &str) -> Result<Completion, LLMAPIError> {
        //dev message so user cannot mess with LLM
        let dev_message = Message::new(MESSAGE_TOOL_RESPONSE.to_string(), MessageRole::Developer);
        //history posted as Assistant to make the LLM know what happened before
        let history_message = Message::new(history.to_string(), MessageRole::Assistant);
        //build user request as Message
        let user_message = Message::new(request.to_string(), MessageRole::User);
        let messages = vec![dev_message, history_message, user_message];

        //send all the messages to the LLM and take result
        self.llm.prompt(&messages)
    }

    fn get_tool_name(toolcall: &ToolCall) -> String {
        toolcall.function.name.clone()
    }
    fn get_tool_arguments(toolcall: &ToolCall) -> serde_json::Value {
        //I somehow receive a serde_json::Value of a String that contains the actual serde_json::Value, this is my fix
        let double_encoded_string = toolcall.function.arguments.clone();
        let parsed = if let Some(s) = double_encoded_string.as_str() {
        serde_json::from_str::<serde_json::Value>(s).unwrap()
        } else {
            double_encoded_string
        };
        parsed
    }
}
