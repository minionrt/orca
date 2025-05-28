use create::llm::{LLM, MessageRole};

const SUMMARIZE_INTRO: &str = "You are building your own memory of an interaction. 
    Please summarize the most important points of the following interaction history
    and the new two messages, one by the user and one by the LLM:\n";

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
            llm: LLM::full(api_key, base_url, "model"), //TODO change model
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
    pub fn with_model(&self, model: String) -> Self{ //TODO change Model to model type and implement global enum for models
        self.llm = LLM::full(self.api_key, self.base_url, model);
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