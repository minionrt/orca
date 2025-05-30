#![allow(dead_code)]

use crate::llm::{LLM, MessageRole};
use url::Url;

const SUMMARIZE_INTRO: &str =
    "Build a memory representation of the interaction. Skip all introduction. 
Summarize the most important points from the entire conversation history provided, 
along with the latest two messages—one from the user and one from the assistant. 
Focus on capturing key facts, goals, preferences, and any evolving context.\n";

pub struct Memory {
    history: String, //make option??
    llm: LLM,
    api_key: String,
    base_url: Url,
}

impl Memory {
    ///creates new Memory instance with empty history and predefined model
    pub fn new(api_key: String, base_url: &Url) -> Self {
        Memory {
            history: "".to_string(),
            llm: LLM::full(api_key.clone(), base_url.clone(), "model".to_string()), //TODO replace model with one from model enum
            api_key: api_key,
            base_url: base_url.clone(),
        }
    }
    ///creates a new Memory instance with given history and predefined model
    pub fn new_with_history(api_key: String, base_url: &Url, history: String) -> Self {
        Self::new(api_key, base_url).with_history(&history)
    }
    ///(re-)sets the current history to the given one
    pub fn with_history(mut self, history: &String) -> Self {
        self.history = history.clone();
        self
    }
    ///(re-)sets the current model to the given one
    pub fn with_model(mut self, model: String) -> Self {
        //TODO change Model to model type
        self.llm = LLM::full(self.api_key.clone(), self.base_url.clone(), model); //add model.into() here
        self
    }

    ///returns the whole history as String
    pub fn read(&self) -> &String {
        &self.history
    }
    ///add new interaction to history
    pub fn add(mut self, user_input: String, llm_answer: String) {
        let content = format!(
            "{}\n History: {} \n User Input: {}\n The ansnwer of the LLM: {}",
            SUMMARIZE_INTRO, self.history, user_input, llm_answer
        );
        self.history = match self.llm.prompt_unwrapped(content, MessageRole::User) {
            Ok(r) => r.completions[0].content.clone(),
            Err(e) => panic!("Something went wrong with summarizing the memory."),
        };
    }
    ///delete whole history
    pub fn delete_history(mut self) -> Self {
        self.history = "".to_string();
        self
    }
}
