use std::fmt;
use crate::openai;

pub struct Completion {
    content: String,
    role: MessageRole,
}
impl Completion {
    pub fn new(content: String, role: MessageRole) -> Self {
        Completion {
            content, role
        }
    }
}

pub struct PromptResponse {
    pub completions: Vec<Completion>,
}

impl PromptResponse {
    pub fn new(completions: Vec<Completion>) -> Self {
        PromptResponse {
            completions,
        }
    }
}

#[derive(Clone, Debug)]
pub enum LLMAPIError {
    RefusedCompletion(String),
}

impl std::error::Error for LLMAPIError {}

impl fmt::Display for LLMAPIError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LLMAPIError::RefusedCompletion(refusal) => write!(f, "The LLM refused completion with the following response: {}", refusal)
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
pub struct LLM {
    api_key: String,
    base_url: String,
    model: String,

    max_tokens: Option<u32>,

    messages: Vec<openai::Message>,
}

impl LLM {
    fn new() -> Self {
        LLM {
            api_key: "".to_string(),
            base_url: "".to_string(),
            model: "".to_string(),
            max_tokens: None,
            messages: vec![],
        }
    }
    fn full(api_key: String, base_url: String, model: String) -> Self {
        Self::new()
            .with_api_key(api_key)
            .with_base_url(base_url)
            .with_model(model)
            .clone()
    }

    fn with_api_key(&mut self, api_key: String) -> &mut Self {
        self.api_key = api_key;
        self
    }
    fn with_base_url(&mut self, base_url: String) -> &mut Self {
        self.base_url = base_url;
        self
    }
    fn with_model(&mut self, model: String) -> &mut Self {
        self.model = model;
        self
    }

    fn with_max_tokens(&mut self, max_tokens: u32) -> &mut Self {
        self.max_tokens = Some(max_tokens);
        self
    }
    fn with_unlimited_tokens(&mut self) -> &mut Self {
        self.max_tokens = None;
        self
    }

    pub fn add_message(&mut self, role: MessageRole, content: String) -> &mut Self {
        self.messages.push(openai::Message{
            content,
            role: role.into(),
            name: None,
        });
        self
    }
    pub fn add_named_message(&mut self, role: MessageRole, content: String, name: String) -> &mut Self {
        self.messages.push(openai::Message{
            content,
            role: role.into(),
            name: Some(name),
        });
        self
    }

    pub fn prompt(&mut self) -> Result<PromptResponse> {
        // TODO actually prompt

        self.messages.clear();

        Ok(PromptResponse::new(vec![Completion::new("this is a dummy response".to_owned(), MessageRole::Assistant)]))
    }
}