#![allow(dead_code)]

use crate::openai;

pub trait ToolInstance {
    // Runs the function with its parameters given as Vector of Strings
    fn run(
        &self,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error>>;
    // Returns the definition and description of the function as defined in openai::Tool
    fn return_choice() -> openai::Tool;
}
