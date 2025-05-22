#![allow(dead_code)]

use crate::openai;
use std::fmt;

#[derive(Clone, Debug)]

//struct containing the content of the response
pub struct Completion {
    pub content: String,
}
impl Completion {
    pub fn new(content: String, _role: MessageRole) -> Self {
        Completion { content }
    }
}
impl TryFrom<openai::Choice> for Completion {
    type Error = LLMAPIError;

    fn try_from(choice: openai::Choice) -> std::result::Result<Self, Self::Error> {
        Ok(Self {
            content: choice.message.content.ok_or_else(|| {
                if choice.message.refusal.is_some() {
                    LLMAPIError::RefusedCompletion(choice.message.refusal.unwrap())
                } else {
                    LLMAPIError::UnknownError("Empty completion".to_owned())
                }
            })?,
        })
    }
}

#[derive(Clone)]
pub struct Message {
    pub content: String,
    pub role: MessageRole,
    pub name: Option<String>,
}

impl Message {
    pub fn new(content: String, role: MessageRole) -> Self {
        Self {
            content,
            role,
            name: None,
        }
    }
    pub fn new_named(content: String, role: MessageRole, name: String) -> Self {
        Self {
            content,
            role,
            name: Some(name),
        }
    }
}

impl From<Message> for openai::Message {
    fn from(m: Message) -> Self {
        openai::Message {
            content: m.content,
            role: m.role.into(),
            name: m.name,
        }
    }
}

#[derive(Debug)]
pub struct PromptResponse {
    pub completions: Vec<Completion>,
}

impl PromptResponse {
    pub fn new(completions: Vec<Completion>) -> Self {
        PromptResponse { completions }
    }
}

impl TryFrom<openai::Completion> for PromptResponse {
    type Error = LLMAPIError;
    fn try_from(completion: openai::Completion) -> Result<Self> {
        let checked_completions = completion
            .choices
            .iter()
            .map(|c| c.clone().try_into())
            .collect::<Vec<_>>();

        if let Some(err) = checked_completions.iter().find(|c| c.is_err()) {
            Err(err.clone().unwrap_err())
        } else {
            Ok(Self {
                completions: checked_completions
                    .iter()
                    .map(|c| c.clone().unwrap())
                    .collect(),
            })
        }
    }
}

#[derive(Clone, Debug)]
pub enum LLMAPIError {
    RefusedCompletion(String),
    NetworkError(String),
    UnknownError(String),
}

impl std::error::Error for LLMAPIError {}

impl fmt::Display for LLMAPIError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LLMAPIError::RefusedCompletion(refusal) => write!(
                f,
                "The LLM refused completion with the following response: {}",
                refusal
            ),
            LLMAPIError::NetworkError(err) => write!(f, "Network Error: {}", err),
            LLMAPIError::UnknownError(err) => write!(f, "Unknown Error: {}", err),
        }
    }
}

type Result<T> = std::result::Result<T, LLMAPIError>;

#[derive(Clone, Copy)]
pub enum MessageRole {
    Developer,
    System,
    User,
    Assistant,
    Tool,
}

impl From<MessageRole> for String {
    fn from(message: MessageRole) -> Self {
        match message {
            MessageRole::Developer => "developer".to_owned(),
            MessageRole::System => "system".to_owned(),
            MessageRole::User => "user".to_owned(),
            MessageRole::Assistant => "assistant".to_owned(),
            MessageRole::Tool => "tool".to_owned(),
        }
    }
}

#[derive(Clone)]
#[allow(clippy::upper_case_acronyms)]
pub struct LLM {
    api_key: String,
    base_url: String,
    model: String,

    max_tokens: Option<i32>,
    client: Option<reqwest::blocking::Client>,
}

impl LLM {
    pub fn new() -> Self {
        LLM {
            api_key: "".to_string(),
            base_url: "".to_string(),
            model: "".to_string(),
            max_tokens: None,
            client: None,
        }
    }
    pub fn full(api_key: String, base_url: String, model: String) -> Self {
        Self::new()
            .with_api_key(api_key)
            .with_base_url(base_url)
            .with_model(model)
            .clone()
    }

    pub fn with_api_key(&mut self, api_key: String) -> &mut Self {
        self.api_key = api_key;
        self
    }
    pub fn with_base_url(&mut self, base_url: String) -> &mut Self {
        self.base_url = base_url;
        self
    }
    pub fn with_model(&mut self, model: String) -> &mut Self {
        self.model = model;
        self
    }

    pub fn with_max_tokens(&mut self, max_tokens: i32) -> &mut Self {
        self.max_tokens = Some(max_tokens);
        self
    }
    pub fn with_unlimited_tokens(&mut self) -> &mut Self {
        self.max_tokens = None;
        self
    }
    pub fn with_client(&mut self, client: reqwest::blocking::Client) -> &mut Self {
        self.client = Some(client);
        self
    }

    pub fn prompt(&self, messages: &[Message]) -> Result<PromptResponse> {
        // TODO reuse client
        let client = self.client.clone().unwrap_or(self.default_client());

        let completion = openai::fetch_completion(
            &self.base_url,
            &self.api_key,
            &self.make_body(messages),
            client,
        );

        println!("{:?}", completion);

        match completion {
            Ok(ok) => ok.try_into(),
            Err(err) => Err(LLMAPIError::NetworkError(err.to_string())),
        }
    }

    pub fn prompt_single(&self, message: Message) -> Result<PromptResponse> {
        self.prompt(&[message])
    }

    pub fn prompt_unwrapped(&self, content: String, role: MessageRole) -> Result<PromptResponse> {
        self.prompt_single(Message::new(content, role))
    }

    pub fn prompt_unwrapped_named(
        &self,
        content: String,
        role: MessageRole,
        name: String,
    ) -> Result<PromptResponse> {
        self.prompt_single(Message::new_named(content, role, name))
    }

    fn make_body(&self, messages: &[Message]) -> openai::CompletionBody {
        openai::CompletionBody {
            messages: messages.iter().map(|m| m.clone().into()).collect(),
            model: self.model.clone(),
            temperature: None,
            top_p: None,
            stream: None,
            frequency_penalty: None,
            presence_penalty: None,
            n: None,
            logit_bias: None,
            logprobs: None,
            max_completion_tokens: self.max_tokens,
            max_tokens: self.max_tokens,
            user: None,
        }
    }

    fn default_client(&self) -> reqwest::blocking::Client {
        reqwest::blocking::Client::new()
    }
}
