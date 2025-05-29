#![allow(dead_code)]

use crate::openai;
use std::fmt;

/// One completion-response of a LLM
#[derive(Clone, Debug)]
pub struct Completion {
    pub content: String,
}
impl Completion {
    pub fn new(content: String, _role: MessageRole) -> Self {
        Completion { content }
    }
}

/// conversion from the underlying OpenAI interface equivalent
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

/// One message that can be sent to the LLM in a chain of other messages
#[derive(Clone)]
pub struct Message {
    /// the contents of the message
    pub content: String,
    /// the role of the message author
    pub role: MessageRole,
    /// an optional name for the participant, to be used for differentiating between participants of the same role
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

/// Conversion to the underlying OpenAI interface
impl From<Message> for openai::Message {
    fn from(m: Message) -> Self {
        openai::Message {
            content: m.content,
            role: m.role.into(),
            name: m.name,
        }
    }
}

/// The response of a LLM containing a chain of completion-responses
#[derive(Debug)]
pub struct PromptResponse {
    pub completions: Vec<Completion>,
}

impl PromptResponse {
    pub fn new(completions: Vec<Completion>) -> Self {
        PromptResponse { completions }
    }
}

/// Conversion from the underlying OpenAI interface
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

/// All possible errors that can be returned in a Result
#[derive(Clone, Debug)]
pub enum LLMAPIError {
    RefusedCompletion(String),
    NetworkError(String),
    UnknownError(String),
    MissingConfig(String),
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
            LLMAPIError::MissingConfig(err) => write!(
                f,
                "The config option \"{}\" is missing, but needed for prompting",
                err
            ),
        }
    }
}

type Result<T> = std::result::Result<T, LLMAPIError>;

/// The roles a LLM thinks a message is sent from
#[derive(Clone, Copy)]
pub enum MessageRole {
    /// specification on how the LLM should behave (newer version of system)
    Developer,
    /// specification on how the LLM should behave
    System,
    /// normal prompt
    User,
    /// a message the LLM thinks it has done
    Assistant,
    /// responses from tools
    Tool,
}

/// Conversion to the underlying OpenAI API
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

/// A struct representing a client who can talk to LLMs.
/// The base URL, the model and an API key have to be assigned before prompting.
/// More config includes limiting the max amount of tokens and setting a custom `reqwest::blocking::Client`
#[derive(Clone)]
#[allow(clippy::upper_case_acronyms)]
pub struct LLM {
    api_key: Option<String>,
    base_url: Option<reqwest::Url>,
    model: Option<String>,

    max_tokens: Option<i32>,
    client: Option<reqwest::blocking::Client>,
}

impl LLM {
    /// create a new LLM with an uninitialized configuration
    pub fn new() -> Self {
        LLM {
            api_key: None,
            base_url: None,
            model: None,
            max_tokens: None,
            client: None,
        }
    }
    /// create a new LLM with all configuration necessary for prompting (API key, base URL and model)
    pub fn full(api_key: String, base_url: reqwest::Url, model: String) -> Self {
        Self::new()
            .with_api_key(api_key)
            .with_base_url(base_url)
            .with_model(model)
            .clone()
    }

    /// set the used API key
    pub fn with_api_key(&mut self, api_key: String) -> &mut Self {
        self.api_key = Some(api_key);
        self
    }

    /// set the base URL, this is *not* the URL of the LLM endpoint, but on which `/chat/completions` will be appended for the OpenAI API
    pub fn with_base_url(&mut self, base_url: reqwest::Url) -> &mut Self {
        self.base_url = Some(base_url);
        self
    }

    /// set the used model
    pub fn with_model(&mut self, model: String) -> &mut Self {
        self.model = Some(model);
        self
    }

    /// limit the amount of tokens used in a prompt
    pub fn with_max_tokens(&mut self, max_tokens: i32) -> &mut Self {
        self.max_tokens = Some(max_tokens);
        self
    }
    /// remove the limit on tokens
    pub fn with_unlimited_tokens(&mut self) -> &mut Self {
        self.max_tokens = None;
        self
    }
    /// set a custom `reqwest::blocking::Client` which is then used to query the LLM endpoint
    pub fn with_client(&mut self, client: reqwest::blocking::Client) -> &mut Self {
        self.client = Some(client);
        self
    }

    /// prompt the LLM with a chain of `Message`
    pub fn prompt(&self, messages: &[Message]) -> Result<PromptResponse> {
        let client = self.client.clone().unwrap_or(self.default_client());

        let completion = openai::fetch_completion(
            &self
                .base_url
                .clone()
                .ok_or(LLMAPIError::MissingConfig("base URL".to_owned()))?,
            &self
                .api_key
                .clone()
                .ok_or(LLMAPIError::MissingConfig("API key".to_owned()))?,
            &self.make_body(
                messages,
                self.model
                    .clone()
                    .ok_or(LLMAPIError::MissingConfig("model".to_owned()))?,
            ),
            client,
        );

        println!("{:?}", completion);

        match completion {
            Ok(ok) => ok.try_into(),
            Err(err) => Err(LLMAPIError::NetworkError(err.to_string())),
        }
    }

    /// prompt the LLM with a single `Message`
    pub fn prompt_single(&self, message: Message) -> Result<PromptResponse> {
        self.prompt(&[message])
    }

    /// prompt the LLM with a single `Message` which is created over the given parameters
    pub fn prompt_unwrapped(&self, content: String, role: MessageRole) -> Result<PromptResponse> {
        self.prompt_single(Message::new(content, role))
    }

    /// prompt the LLM with a single `Message` which is created over the given parameters, also sets the name from which the LLM thinks the message was sent
    pub fn prompt_unwrapped_named(
        &self,
        content: String,
        role: MessageRole,
        name: String,
    ) -> Result<PromptResponse> {
        self.prompt_single(Message::new_named(content, role, name))
    }

    fn make_body(&self, messages: &[Message], model: String) -> openai::CompletionBody {
        openai::CompletionBody {
            messages: messages.iter().map(|m| m.clone().into()).collect(),
            model,
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
