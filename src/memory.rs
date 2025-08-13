#![allow(dead_code)]

use crate::llm::{Completion, LLM, MessageRole};
use crate::models::Model;
use std::collections::VecDeque;
use tracing::{debug, error, info};
use url::Url;

const MEMORY_LENGTH: usize = 8;
const SUMMARIZE_INTRO: &str = r#"**System Prompt: Coding Agent Memory Summarizer**

You are an AI specialized in compressing the memory of a coding assistant’s past interactions into concise, factual summaries that preserve all important information needed to continue the task effectively.

**Your role:**
- You will receive two new messages: one from the user and one from the coding agent.
- You will also receive the previous summary of all earlier interactions.
- Your task is to create a new, updated summary that incorporates the two new messages and all necessary information from the previous summary.

**Guidelines:**
1. **Purpose:** The summary is for the agent’s internal memory only, not for the user. It must be optimized for problem-solving and continuity of work.
2. **Content:**
   - Include only factual, task-relevant details: code-related facts, problem statements, constraints, solutions tried, results, and technical context.
   - Keep the facts as they are stated; do not paraphrase technical details.
   - Do not include irrelevant conversation, small talk, or emotional content.
3. **Conciseness:**
   - Be as short as possible while preserving all important key facts from the task, code, and topic.
   - Do not include full code snippets unless a specific fragment is essential for context.
4. **Accuracy:** Ensure all retained facts remain correct and unaltered.
5. **Structure:**
   - Use a clear, structured format that helps the coding agent quickly retrieve information.
   - Suggested sections:
     - **Current Task / Goal**
     - **Key Facts & Context**
     - **Progress & Decisions Made**
     - **Outstanding Questions / Next Steps**

**Output:**
Produce only the updated summary in the structured format above. Do not include explanations of your process."#;

#[derive(Clone, Debug)]
pub struct Interaction {
    // Message of the user, in our case the task or the response of the tools
    pub user: String,
    // Answer of the LLM
    pub assistant: String,
}

/// Memory stores a summarized history plus the most recent `MEMORY_LENGTH` full interactions
///
/// This should act as our "working memory":
/// - `recent` contains the latest `MEMORY_LENGTH` interactions
/// - `Memory` contains the proposed filter, so all the messages older than the limit are summarized by the defined LLM
pub struct Memory {
    history: String,
    recent: VecDeque<Interaction>, // Holds last `MEMORY_LENGTH` interactions
    llm: LLM,
    api_key: String,
    base_url: Url,
    max_pairs: usize,
}

impl Memory {
    /// Create new `Memory` instance with empty history, no recent interactions and predefined model
    pub fn new(api_key: &str, base_url: &Url) -> Self {
        info!("Memory: Creating new Memory instance");
        Memory {
            history: "empty".to_string(),
            recent: VecDeque::new(),
            llm: LLM::full(
                api_key.to_string(),
                base_url.clone(),
                Model::Basic.into(),
                None,
                None,
            ),
            api_key: api_key.to_string(),
            base_url: base_url.clone(),
            max_pairs: MEMORY_LENGTH,
        }
    }

    /// Ceate new `Memory` instance with a given history so we can restore memory or make the agent remember something
    pub fn new_with_history(api_key: String, base_url: &Url, history: String) -> Self {
        info!("Memory: Creating new Memory instance with history");
        Self::new(&api_key, base_url).with_history(&history)
    }

    /// Change the history of the agent. Attention, that only replaces the summarized history part!
    ///
    /// If you also want to delete the latest interactions that are stored as literal Strings, you need to call `delete_recent`
    pub fn with_history(mut self, history: &str) -> Self {
        info!("Memory: Changing history of agent, not `recent`-queue");
        self.history = history.to_owned();
        self
    }

    /// Delete the queue of recent interactions
    ///
    /// Attention! Doesn't clear history of summarized messages. If you want to delete both, please call `delete_history`
    pub fn delete_recent(mut self) -> Self {
        info!("Memory: Deleting recent-queue, not history");
        self.recent.clear();
        self
    }

    /// Clears both the summarized history and the recent interactions queue.
    pub fn delete_history(&mut self) -> &Self {
        info!("Memory: Deleting recent-queue and history");
        self.history.clear();
        self.recent.clear();
        self
    }

    /// Changes the LLM model used for summarization to manually defined `Model` value.
    pub fn with_model(mut self, model: Model) -> Self {
        info!("Memory: Changing the LLM Model for the Memory representation");
        self.llm = LLM::full(
            self.api_key.clone(),
            self.base_url.clone(),
            model.into(),
            None,
            None,
        );
        self
    }

    /// Return string of summarized history followed by all recent interactions.
    ///
    /// This is the full memory context of the agent that can be used for prompting the LLM.
    pub fn read(&self) -> String {
        info!("Memory: reading full memory context");
        let mut combined = String::new();
        combined.push_str(&format!("Summarized history:\n{}\n\n", self.history));
        combined.push_str("Recent interactions:\n");
        for (i, pair) in self.recent.iter().enumerate() {
            combined.push_str(&format!(
                "Pair {}:\nUser: {}\nAssistant: {}\n\n",
                i + 1,
                pair.user,
                pair.assistant
            ));
        }
        debug!("Memory: Full memory context:{}", combined);
        combined
    }

    /// Add new interaction to Memory representation
    ///
    /// The new interaction is appended as it is to the `recent` queue and if the queue hits its border
    /// the oldest interactions are summarized into `history`.
    pub fn add(&mut self, user_input: String, llm_answer: String) {
        info!("Memory: Adding the new interaction pair to recent");
        debug!("User_input: {}, LLM_Input: {}", user_input, llm_answer);
        self.recent.push_back(Interaction {
            user: user_input,
            assistant: llm_answer,
        });

        info!("Memory: Cutting queue to defined length and summarizing the rest");
        if self.recent.len() > self.max_pairs {
            let overflow_count = self.recent.len() - self.max_pairs;
            let mut overflow_text = String::new();

            for _ in 0..overflow_count {
                if let Some(old) = self.recent.pop_front() {
                    overflow_text.push_str(&format!(
                        "User: {}\nAssistant: {}\n\n",
                        old.user, old.assistant
                    ));
                }
            }

            let content = format!(
                "{}\nHistory: {}\nNew Interactions to Summarize:\n{}",
                SUMMARIZE_INTRO, self.history, overflow_text
            );

            self.history = match self.llm.prompt_unwrapped(content, MessageRole::User) {
                Ok(r) => match &r {
                    Completion::Text(content) => content.clone(),
                    Completion::ToolCalls(_) => panic!("Expected text completion, got tool call!"),
                },
                Err(e) => {
                    error!("Memory: LLM request failed: {}", e);
                    panic!("Something went wrong with summarizing the memory.")
                }
            };
            debug!("New history representation: {}", self.history);
        }
    }
}
