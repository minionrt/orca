use std::path::{Path, PathBuf};

use tokio::sync::MutexGuard;
use tracing::error;

use crate::e2e::{GitCmd, builder::AgentContainerBuilder, container::get_docker};

use super::{AgentExpectedAction, LLMCall, LLMResponse, Result, Task, TestFailure};

#[derive(Clone)]
pub struct AgentTestEnvironment {
    /// Actions which have to be done in the given order.
    pub serial_actions: Vec<AgentExpectedAction>,
    /// Actions which can be done at any time and just have to happen at least once.
    pub post_conditions: Vec<AgentExpectedAction>,

    /// The task which is provided to the agent.
    pub task: Task,

    /// The response the LLM sends back if no other response is specified in the actions
    pub default_llm_response: LLMResponse,

    /// The current index of the serial actions
    serial_action_index: usize,

    /// The temporary local git repo
    pub(super) git_repo_path: PathBuf,

    // The git branch
    pub(super) git_branch: String,
}

impl Default for AgentTestEnvironment {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentTestEnvironment {
    /// Construct a new agent test environment with empty task and no requirements for the agent.
    pub fn new() -> Self {
        Self {
            serial_actions: Vec::new(),
            post_conditions: Vec::new(),
            task: Task::new(""),
            default_llm_response: LLMResponse::Message("No LLM Response was specified.".to_owned()),

            serial_action_index: 0,
            git_repo_path: PathBuf::new(),
            git_branch: String::new(),
        }
    }

    /// Set the task the agent should do
    pub fn with_task(mut self, task: &Task) -> Self {
        self.task = task.clone();
        self
    }

    /// Set the default LLM response
    pub fn with_default_llm_response(mut self, response: LLMResponse) -> Self {
        self.default_llm_response = response;
        self
    }

    /// Expect an LLM prompt and send back the given `response`.
    /// If `exact_prompt` is not `None`, the received LLM prompt has to correspond to it.
    pub fn expect_llm_call(mut self, response: &LLMResponse, exact_prompt: Option<&str>) -> Self {
        self.serial_actions
            .push(AgentExpectedAction::LLMCall(LLMCall::new(
                response.clone(),
                exact_prompt,
            )));
        self
    }

    /// Expect a termination request of the agent indicating success
    pub fn expect_success(mut self, success_report: Option<&str>) -> Self {
        self.serial_actions.push(AgentExpectedAction::TaskSuccess(
            success_report.map(|s| s.to_owned()),
        ));
        self
    }

    /// Expect a termination request of the agent indicating failure with a corresponding failure report.
    /// If `failure_report` is not `None`, the received failure report has to correspond to it.
    pub fn expect_failure(
        mut self,
        reason: Option<Option<&str>>,
        description: Option<&str>,
    ) -> Self {
        self.serial_actions.push(AgentExpectedAction::TaskFailure(
            reason.map(|s| s.map(|s| s.to_owned())),
            description.map(|s| s.to_owned()),
        ));
        self
    }

    /// Test for the existence of a file with file name `file_name`.
    /// If the file is under a specific path, this path has to be included in the file name.
    /// An optional file content can be specified with `content` (is only tested if `content` is not `None`).
    /// The check is done at no specific time.
    pub fn expect_file(mut self, file_name: &str, content: Option<&str>) -> Self {
        self.post_conditions.push(AgentExpectedAction::File(
            file_name.to_owned(),
            content.map(|s| s.to_owned()),
        ));
        self
    }

    /// Expect a git command
    pub fn expect_git_cmd(mut self, git_cmd: GitCmd) -> Self {
        self.serial_actions
            .push(AgentExpectedAction::GitCmd(git_cmd));
        self
    }

    pub fn build<P: AsRef<Path>>(self, path: P) -> Result<AgentContainerBuilder> {
        super::builder::AgentContainerBuilder::new(path, self, get_docker()?)
    }

    fn next_serial_action(&mut self) -> Option<&AgentExpectedAction> {
        let action = self.serial_actions.get(self.serial_action_index);
        self.serial_action_index += 1;
        action
    }
    fn peek_next_serial_action(&mut self) -> Option<&AgentExpectedAction> {
        self.serial_actions.get(self.serial_action_index)
    }
    fn following_serial_actions(&self) -> &[AgentExpectedAction] {
        let actions = self.uncompleted_serial_actions();
        if !actions.is_empty() {
            &actions[1..]
        } else {
            &[]
        }
    }
    pub(super) fn uncompleted_serial_actions(&self) -> &[AgentExpectedAction] {
        if self.serial_action_index >= self.serial_actions.len() {
            return &[];
        }
        &self.serial_actions[self.serial_action_index..]
    }

    pub(super) fn next_llm_response(&self) -> LLMResponse {
        self.uncompleted_serial_actions()
            .iter()
            .find_map(|action| {
                if let AgentExpectedAction::LLMCall(call) = action {
                    Some(call.response.clone())
                } else {
                    None
                }
            })
            .unwrap_or(self.default_llm_response.clone())
    }

    fn is_valid_action_cmp(queued: &AgentExpectedAction, real: &AgentExpectedAction) -> bool {
        let queued = queued.clone();
        let real = real.clone();
        queued == real
            || match queued.clone() {
                AgentExpectedAction::File(name, _) => AgentExpectedAction::File(name, None) == real,
                AgentExpectedAction::TaskSuccess(_) => {
                    AgentExpectedAction::TaskSuccess(None) == real
                }
                AgentExpectedAction::TaskFailure(reason, description) => {
                    AgentExpectedAction::TaskFailure(None, None) == real
                        || AgentExpectedAction::TaskFailure(reason, None) == real
                        || AgentExpectedAction::TaskFailure(None, description) == real
                }
                AgentExpectedAction::LLMCall(llm_call) => match real {
                    AgentExpectedAction::LLMCall(real_call) => llm_call
                        .prompt
                        .is_none_or(|c| real_call.prompt.is_some_and(|r| r == c)),
                    _ => false,
                },
                _ => false,
            }
    }

    /// Checks if the given action is next in queue.
    /// If it is not next in the queue, it also can't be in the back of the queue
    pub(super) fn is_valid_action(&mut self, action: &AgentExpectedAction) -> bool {
        if let Some(a) = self.peek_next_serial_action()
            && Self::is_valid_action_cmp(a, action)
        {
            self.next_serial_action();
            return true;
        }
        !self
            .following_serial_actions()
            .iter()
            .any(|a| Self::is_valid_action_cmp(a, action))
    }

    /// Checks if the given action is valid via `is_valid_action` and then returns Some(failure) if it is invalid
    pub(super) fn invalid_action_to_failure(
        &mut self,
        action: &AgentExpectedAction,
        failure: TestFailure,
    ) -> Option<TestFailure> {
        if self.is_valid_action(action) {
            None
        } else {
            Some(failure)
        }
    }

    /// Prints an error message and shuts down the server over `context` if the action is invalid.
    pub(super) async fn shutdown_on_invalid_action<'a>(
        context: &mut MutexGuard<'a, crate::e2e::ServerContext>,
        action: AgentExpectedAction,
        failure: TestFailure,
    ) {
        context.test_data.failure = context
            .environment
            .invalid_action_to_failure(&action, failure);
        if context.test_data.failure.is_some() {
            error!(
                "Test failure: {}",
                context.test_data.failure.clone().unwrap()
            );
            context.terminate_server()
        }
    }
}
