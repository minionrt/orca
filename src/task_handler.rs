#![allow(dead_code)]

use tracing::{debug, error, info, warn};
use url::Url;

use crate::agent_actions::submit_code::GitSubmissionTool;
use crate::llm::{Completion, LLM, LLMAPIError, Message, MessageRole};
use crate::memory::Memory;
use crate::models::Model;
use crate::openai::{ToolCall, ToolChoice};
use crate::report::TaskFailureReason;
use crate::tools::collection::{self, get_tools};

/// Returns the system prompt for the developer, including the working directory path.
///
/// # Arguments
///
/// * `path` - The path to be included in the prompt.
fn intro_with_path(path: &str) -> String {
    format!(
        "You are an autonomous agent that solves coding tasks. \
You should use the given tools to solve the given task. \
Please ONLY use bash tool if none of the others offers what you want to do, don't use bash tool with \"cd\"!\
You are connected to a Linux-based development environment. \
You are in the project directory. The path to the file you should work on is: {path} \
Your current task is as follows:"
    )
}

const MESSAGE_TOOL_RESPONSE: &str = r#"You are an autonomous agent that solves coding tasks. 
You should use the given tools to solve the given task.
You are connected to a Linux-based development environment. You are in the 
project directory. If you think your task is done please call the git_submission tool and in the next step just tell me what you did.
Please ONLY use bash tool if none of the others offers what you want to do, don't use bash tool with \"cd\"!
The response of your last tool call is the following:"#;

/// The possible outcomes of a task.
pub enum TaskOutcome {
    Complete(String),
    Failure(String, Option<TaskFailureReason>), // One field for a description, one for a reason
}

/// Represents a task to be performed by the agent.
pub struct Task {
    /// The user request or task description.
    pub request: String,
    /// The working directory relevant to the task.
    pub working_dir: String,
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
    /// Creates a new TaskHandler with the default model and tools.
    pub fn new(api_key: &str, base_url: &Url) -> Self {
        TaskHandler {
            llm: LLM::full(
                api_key.to_string(),
                base_url.clone(),
                Model::Smart.into(),
                get_tools(),
                Some(ToolChoice::Auto),
            ),
            memory: Memory::new(api_key, base_url),
        }
    }

    /// Creates a new TaskHandler with a specified model.
    pub fn new_set_model(api_key: &str, base_url: &Url, model: Model) -> Self {
        TaskHandler {
            llm: LLM::full(
                api_key.to_string(),
                base_url.clone(),
                String::from(model),
                None,
                None,
            ),
            memory: Memory::new(api_key, base_url),
        }
    }

    /// Creates a new TaskHandler with a specified model and tool choice.
    pub fn new_set_model_and_tools(
        api_key: &str,
        base_url: &Url,
        model: Model,
        tool_choice: Option<crate::openai::ToolChoice>,
    ) -> Self {
        TaskHandler {
            llm: LLM::full(
                api_key.to_string(),
                base_url.clone(),
                String::from(model),
                get_tools(),
                tool_choice,
            ),
            memory: Memory::new(api_key, base_url),
        }
    }

    /// Runs the main interaction loop with the LLM for a given task.
    /// The working directory path from the task is included in the system prompt.
    pub fn run(&mut self, task: &Task) -> TaskOutcome {
        info!("Starting task execution");
        debug!("Task request: {}", task.request);
        debug!("Working directory: {}", task.working_dir);

        let mut input = task.request.clone();
        let mut response = self.single_request(&task.request, &task.working_dir);

        let mut ctr: i8 = 0;
        let mut submitted = false;

        loop {
            ctr += 1;
            debug!("Interaction loop iteration: {}", ctr);

            // Match response, if there was an error, propagate to user
            let completion = match &response {
                Ok(c) => c.clone(),
                Err(e) => {
                    error!("LLM request failed: {}", e);
                    return TaskOutcome::Failure(
                        e.to_string(),
                        Some(TaskFailureReason::TechnicalIssues),
                    );
                }
            };

            // If llm returns a text I expect the task to be done
            if let Completion::Text(value) = completion {
                // If the agent doesn't submit we use that message and submit for it
                if !submitted {
                    let _ = GitSubmissionTool::new(&task.working_dir).submit_changes(&value);
                    debug!(
                        "Git submission tool had to be called manually, the agent didn't submit."
                    );
                }
                info!("Task completed with text response");
                return TaskOutcome::Complete(value);
            } else if let Completion::ToolCalls(value) = completion {
                // If LLM returns a tool call, extract the tool name and arguments and call tool
                let tool_name = TaskHandler::get_tool_name(&value[0]);
                let args = TaskHandler::get_tool_arguments(&value[0]);

                info!("Calling tool: {} with args: {:?}", tool_name, args);
                let tool_result =
                    collection::call_tool(&tool_name, args.clone(), &task.working_dir);

                if tool_name == "git_submission"
                    && matches!(&tool_result, Ok(serde_json::Value::String(s)) if s.starts_with("submission successful"))
                {
                    submitted = true;
                }

                // Add the new interaction to the memory
                self.memory.add(
                    input,
                    format!("You called the tool \"{tool_name}\" with the Arguments: {args}"),
                );

                // Make string from tool return
                input = match &tool_result {
                    Ok(value) => {
                        debug!("Tool execution successful: {}", value);
                        value.clone().to_string()
                    }
                    Err(e) => {
                        warn!("Tool execution failed: {}", e);
                        e.to_string()
                    }
                };

                // Give returned value of the tool to the llm
                response = self.send_tool_answer(&input, self.memory.read());
            }

            // Stop the loop after x runs
            ctr += 1;
            if ctr >= 25 {
                return TaskOutcome::Failure(
                    "The interaction loop run too long, the agent can't stop yappin..".to_string(),
                    Some(TaskFailureReason::ProblemSolving),
                );
            }
        }
    }

    /// Sends a single code task request to the LLM, including the working directory path in the system prompt.
    fn single_request(&self, request: &str, path: &str) -> Result<Completion, LLMAPIError> {
        let history = self.memory.read();
        self.send_request(request, history.as_str(), path)
    }

    /// Sends a code task request with memory and a custom system prompt with the working directory.
    fn send_request(
        &self,
        request: &str,
        history: &str,
        path: &str,
    ) -> Result<Completion, LLMAPIError> {
        // Dev message includes the working directory path
        let dev_message = Message::new(intro_with_path(path), MessageRole::Developer);
        // History posted as Assistant to make the LLM know what happened before
        let history_message = Message::new(history.to_string(), MessageRole::Assistant);
        // Build user request as Message
        let user_message = Message::new(request.to_string(), MessageRole::User);
        let messages = vec![dev_message, history_message, user_message];

        self.llm.prompt(&messages)
    }

    /// Sends the tool's output as the next user message, using the static tool response prompt.
    fn send_tool_answer(&self, request: &str, history: &str) -> Result<Completion, LLMAPIError> {
        let dev_message = Message::new(MESSAGE_TOOL_RESPONSE.to_string(), MessageRole::Developer);
        let history_message = Message::new(history.to_string(), MessageRole::Assistant);
        let user_message = Message::new(request.to_string(), MessageRole::User);
        let messages = vec![dev_message, history_message, user_message];

        self.llm.prompt(&messages)
    }

    /// Extracts the tool name from a ToolCall.
    fn get_tool_name(toolcall: &ToolCall) -> String {
        toolcall.function.name.clone()
    }

    /// Extracts and decodes the tool arguments from a ToolCall.
    fn get_tool_arguments(toolcall: &ToolCall) -> serde_json::Value {
        let double_encoded_string = toolcall.function.arguments.clone();
        if let Some(s) = double_encoded_string.as_str() {
            serde_json::from_str::<serde_json::Value>(s).unwrap()
        } else {
            double_encoded_string
        }
    }
}
