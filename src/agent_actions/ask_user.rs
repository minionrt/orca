use crate::openai;
use crate::tools_interface::ToolInstance;

pub struct AskUserTool;

impl AskUserTool{
    pub fn new() -> Self{
        AskUserTool
    }
}

pub fn ask_user(inquiry: &str) -> String {
    println!("Inquiry request from the AI 🤖: {}", inquiry);
    format!("Inquiry request from the AI 🤖: {}", inquiry).to_string()
}

impl Default for AskUserTool{
    fn default() -> Self {
        AskUserTool::new()
    }
}
impl ToolInstance for AskUserTool{
    fn run(
            &self,
            params: serde_json::Value,
        ) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let inquiry = match params.get("inquiry"){
            None => {
                return Err("The parameter \"inquiry\" doesn't exist in the given tool call".into());
            }
            Some(serde_json::Value::String(s)) => s,
            Some(_) => return Err("The parameter \"inquiry\" isn't given as string.".into()),
        };
        let output = ask_user(inquiry);
        Ok(serde_json::Value::String(output))
    }

    fn return_choice() -> openai::Tool {
        // Provide a valid Tool instance here as required by your application.
        // Replace the following line with actual construction if needed.
        openai::Tool{
            function: openai::Function{
                name: "ask_user".to_string(),
                description: "a tool for clarification inquiries".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "inquiry": {
                            "type": "string",
                            "description": "the clarifcation request"
                        }
                    },
                    "required": ["inquiry"]
                }),
            },
            tool_type: "function".to_string()
        }
    }
}