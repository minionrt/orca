use create::llm::{LLM, MessageRole};

const SUMMARIZE_INTRO: &str = "Build a memory representation of the interaction. 
Summarize the most important points from the entire conversation history provided, 
along with the latest two messages—one from the user and one from the assistant. 
Focus on capturing key facts, goals, preferences, and any evolving context.\n";

pub struct Memory{
    history: String,
    llm: LLM,
    api_key: String,
    base_url: String
}

impl Memory{
    pub fn new(api_key: String, base_url:String) -> Self{
        Memory{
            history: "".to_owned,
            llm: LLM::full(api_key, base_url, "model"), //TODO replace model with one from model enum
            api_key: api_key,
            base_url: base_url
        }
    }
    pub fn new_with_history(api_key:String, base_url: String, history: String) -> Self{
        Self::new(api_key, base_url)
            .with_history(history)
    }
    pub fn with_history(&self, history: String) -> Self{
        self.history = history;
        self
    }
    pub fn with_model(&self, model: String) -> Self{ //TODO change Model to model type
        self.llm = LLM::full(self.api_key, self.base_url, model); //add model.into() here
        self
    }
    
    pub fn read(&self) -> String {
        self.history
    }
    pub fn add(&self, user_input: String, llm_answer: String){
        content = format!("{}User Input: {}\n The ansnwer of the LLM: {}", SUMMARIZE_INTRO, user_input, llm_answer);
        self.history = self.llm.prompt_unwrapped(content, MessageRole::User);
    }
    pub fn delete_history(&self)-> Self{
        self.history = "".to_owned;
        self
    }
}