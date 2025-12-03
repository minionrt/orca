#![allow(dead_code)]

use http_server::HttpRequest;
use orca::openai::ToolCall;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

mod api;
pub mod builder;
pub mod container;
pub mod environment;
mod git;
mod git_proxy;
mod http_server;
pub mod runner;

use environment::AgentTestEnvironment;

// ==============================

#[derive(Clone)]
pub enum TestLog {
    HttpRequest(HttpRequest),
    CallToFailure,
    CallToSuccess,
    CallToLLM(String),
    CouldNotStopAgent,
}

#[derive(Clone, Debug)]
pub enum TestFailure {
    CallToFailure,
    CallToSuccess,
    CallToTaskInfo,
    CallToLLM(String),
    Git(GitCmd),
    MalformedCallToLLM(String),
    MalformedCallToTaskSuccess,
    MalformedCallToTaskFailure,
    MalformedCallToTaskInfo,
    AgentCrashed(String),
    ActionsMissing(Vec<AgentExpectedAction>),
    Multiple(Box<TestFailure>, Box<TestFailure>),
}

impl Display for TestFailure {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TestFailure::CallToFailure => write!(f, "Unexpected call to the failure endpoint"),
            TestFailure::CallToSuccess => write!(f, "Unexpected call to the success endpoint"),
            TestFailure::CallToTaskInfo => write!(f, "Unexpected call to the Task API"),
            TestFailure::CallToLLM(prompt) => {
                write!(f, "Unexpected LLM call with prompt: {prompt}")
            }
            TestFailure::Git(cmd) => {
                write!(f, "Unexpected git command: {cmd:?}")
            }
            TestFailure::MalformedCallToLLM(info) => {
                write!(f, "Malformed LLM call: {info}")
            }
            TestFailure::MalformedCallToTaskFailure => write!(f, "Malformed Call to Task Failure"),
            TestFailure::MalformedCallToTaskSuccess => write!(f, "Malformed Call to Task Success"),
            TestFailure::MalformedCallToTaskInfo => write!(f, "Malformed Call to Task Info"),
            TestFailure::AgentCrashed(err) => write!(f, "The agent crashed with error: {err}"),
            TestFailure::ActionsMissing(actions) => write!(
                f,
                "There are actions missing, but the agent already stopped: {actions:?}"
            ),
            TestFailure::Multiple(a, b) => {
                write!(f, "Multiple failure reasons:\n    {}\n    {}", *a, *b)
            }
        }
    }
}

#[derive(Clone, Default)]
struct AgentTestFinishedRun {
    /// All registered actions of the agent
    pub log: Vec<TestLog>,

    /// The failure reason (in case the test failed)
    pub failure: Option<TestFailure>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GitCmd {
    Push,
    Pull,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LLMCall {
    pub prompt: Option<String>,
    pub response: LLMResponse,
}
impl LLMCall {
    pub fn new(response: LLMResponse, prompt: Option<&str>) -> Self {
        Self {
            response,
            prompt: prompt.map(|s| s.to_owned()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LLMResponse {
    Message(String),
    ToolCall(ToolCall),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AgentExpectedAction {
    /// Filename, Content
    File(String, Option<String>),
    GitCmd(GitCmd),
    LLMCall(LLMCall),
    TaskInfo,
    /// Description
    TaskSuccess(Option<String>),
    /// reason (optional), description
    TaskFailure(Option<Option<String>>, Option<String>),
}

// temp
#[derive(Clone, Debug, Serialize)]
pub enum TaskStatus {
    Queued,
    Running,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize)]
pub struct Task {
    pub description: String,

    // temp
    pub status: TaskStatus,
    pub git_user_name: String,
    pub git_user_email: String,
    pub git_repo_url: String,
    pub git_branch: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TaskSuccess {
    pub description: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TaskFailure {
    pub description: String,
    #[serde(default)]
    pub reason: Option<String>,
}

const AGENT_TEST_GIT_USER_NAME: &str = "LordTestington";
const AGENT_TEST_GIT_USER_EMAIL: &str = "lordtestington@mctestface.com";
const AGENT_TEST_REPO_URL: &str = "http://host.docker.internal:3000/git/";
const AGENT_TEST_GIT_BRANCH: &str = "test-feature";

impl Task {
    pub fn new(description: &str) -> Self {
        Self {
            description: description.to_owned(),

            status: TaskStatus::Queued,
            git_user_name: AGENT_TEST_GIT_USER_NAME.to_owned(),
            git_user_email: AGENT_TEST_GIT_USER_EMAIL.to_owned(),
            git_repo_url: AGENT_TEST_REPO_URL.to_owned(),
            git_branch: AGENT_TEST_GIT_BRANCH.to_owned(),
        }
    }
}

#[derive(Clone)]
struct ServerContext {
    pub environment: AgentTestEnvironment,
    pub test_data: AgentTestFinishedRun,

    pub terminate_server: bool,
}

impl http_server::ServerContext for ServerContext {
    fn should_terminate_server(&self) -> bool {
        self.terminate_server
    }
}
impl ServerContext {
    pub fn terminate_server(&mut self) {
        self.terminate_server = true
    }
}

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
