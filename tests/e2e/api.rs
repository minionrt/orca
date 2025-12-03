use std::sync::Arc;

use nom::AsBytes;
use tokio::sync::Mutex;
use tracing::{debug, error, info};

use crate::e2e::{
    AgentExpectedAction, LLMResponse, ServerContext, TaskFailure, TaskSuccess, TestFailure,
    environment::AgentTestEnvironment,
    http_server::{HttpRequest, HttpResponse},
};

use orca::openai::{ChatCompletionMessage, Choice, Completion, CompletionBody, UsageStatistic};

pub(super) enum APIEndpoint {
    TaskSuccess,
    TaskFailure,
    TaskInfo,

    LLMRequest,

    GitInfoRefs,
    GitReceivePack,
    GitUploadPack,
}

impl From<APIEndpoint> for &str {
    fn from(val: APIEndpoint) -> Self {
        match val {
            APIEndpoint::TaskSuccess => "/api/agent/task/complete",
            APIEndpoint::TaskFailure => "/api/agent/task/fail",
            APIEndpoint::TaskInfo => "/api/agent/task",
            APIEndpoint::LLMRequest => "/api/chat/completions",

            APIEndpoint::GitInfoRefs => "/git/info/refs",
            APIEndpoint::GitReceivePack => "/git/git-receive-pack",
            APIEndpoint::GitUploadPack => "/git/git-upload-pack",
        }
    }
}

pub async fn task_success(
    request: HttpRequest,
    context: Arc<Mutex<ServerContext>>,
) -> HttpResponse {
    info!(
        "Task succeeded: {:?}",
        String::from_utf8(request.body.to_vec()).unwrap()
    );
    let mut context = context.lock().await;
    let task: TaskSuccess = match serde_json::from_slice(&request.body) {
        Ok(task) => task,
        Err(err) => {
            error!("Parsing Task Success Failed: {}", err);
            context.test_data.failure = Some(TestFailure::MalformedCallToTaskSuccess);
            return HttpResponse::bad_request();
        }
    };
    context.test_data.failure = context.environment.invalid_action_to_failure(
        &crate::e2e::AgentExpectedAction::TaskSuccess(Some(task.description)),
        TestFailure::CallToSuccess,
    );

    tracing::debug!("terminating server...");
    context.terminate_server();
    HttpResponse::ok().clone()
}

pub async fn task_failure(
    request: HttpRequest,
    context: Arc<Mutex<ServerContext>>,
) -> HttpResponse {
    info!(
        "Task failed: {:?}",
        String::from_utf8(request.body.to_vec()).unwrap()
    );
    let mut context = context.lock().await;
    let task: TaskFailure = match serde_json::from_slice(&request.body) {
        Ok(task) => task,
        Err(err) => {
            error!("Parsing Task Failure Failed: {}", err);
            context.test_data.failure = Some(TestFailure::MalformedCallToTaskFailure);
            return HttpResponse::bad_request();
        }
    };
    context.test_data.failure = context.environment.invalid_action_to_failure(
        &crate::e2e::AgentExpectedAction::TaskFailure(Some(task.reason), Some(task.description)),
        TestFailure::CallToFailure,
    );
    context.terminate_server();
    HttpResponse::ok()
}

pub async fn task_info(_request: HttpRequest, context: Arc<Mutex<ServerContext>>) -> HttpResponse {
    info!("Received Task Info Request");
    let mut context = context.lock().await;
    AgentTestEnvironment::shutdown_on_invalid_action(
        &mut context,
        crate::e2e::AgentExpectedAction::TaskInfo,
        TestFailure::CallToTaskInfo,
    )
    .await;

    debug!("Returned Task: {:?}", &context.environment.task.clone());
    HttpResponse::ok().json(&context.environment.task.clone())
}

pub async fn llm_request(request: HttpRequest, context: Arc<Mutex<ServerContext>>) -> HttpResponse {
    info!("Received LLM Request");

    let mut context = context.lock().await;

    let llm_request: serde_json::error::Result<CompletionBody> =
        serde_json::from_slice(request.body.as_bytes());
    if let Err(err) = llm_request {
        context.test_data.failure = Some(TestFailure::MalformedCallToLLM(format!(
            "Incoming request body: {:?}, serde error: {1}",
            request.body.as_bytes(),
            err
        )));
        context.terminate_server();
        return HttpResponse::bad_request();
    }

    let llm_request = llm_request.unwrap();

    if llm_request.messages.is_empty() {
        context.test_data.failure = Some(TestFailure::MalformedCallToLLM(
            "Incoming LLM prompt has no messages".to_owned(),
        ));
        context.terminate_server();
        return HttpResponse::bad_request();
    }

    let llm_prompt = llm_request.messages.first().unwrap().content.clone();

    AgentTestEnvironment::shutdown_on_invalid_action(
        &mut context,
        AgentExpectedAction::LLMCall(crate::e2e::LLMCall::new(
            LLMResponse::Message("".to_owned()),
            Some(&llm_prompt),
        )),
        TestFailure::CallToLLM(llm_prompt.clone()),
    )
    .await;

    let (content, tool_calls) = match context.environment.next_llm_response() {
        LLMResponse::Message(msg) => (Some(msg), None),
        LLMResponse::ToolCall(tool_call) => (None, Some(vec![tool_call])),
    };

    let llm_response = Completion {
        choices: vec![Choice {
            finish_reason: "done".to_string(),
            index: 0,
            message: ChatCompletionMessage {
                content: content.clone(),
                refusal: None,
                role: "assistant".to_string(),
                tool_calls,
            },
        }],
        id: None,
        object: None,
        created: None,
        model: None,
        usage: UsageStatistic {
            completion_tokens: 0,
            prompt_tokens: 0,
            total_tokens: 0,
            completion_tokens_details: None,
        },
    };

    debug!(
        "Return LLM Response: {}",
        content.unwrap_or("(empty)".to_owned())
    );

    HttpResponse::ok().json(&llm_response)
}
