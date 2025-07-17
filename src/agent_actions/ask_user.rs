use crate::openai;
use crate::tools_interface::ToolInstance;
use url::Url;
use std::env;

pub struct AskUserTool;

impl AskUserTool {
    pub fn new() -> Self {
        AskUserTool
    }

    pub fn ask_user(&self, inquiry: &str) -> String {
        let client = reqwest::blocking::Client::new();
        let minion_api: Url = env::var("MINION_API_BASE_URL").unwrap().parse().unwrap();
        let minion_token = env::var("MINION_API_TOKEN").unwrap();
        let url = minion_api.join("agent/inquiry").unwrap();
       

        let response = client
            .post(url)
            .bearer_auth(minion_token.clone())
            .json(&serde_json::json!({"inquiry": inquiry}) )
            .send();

        match response {
            Ok(resp) => match resp.text() {
                Ok(text) => text,
                Err(_) => "[ERROR] Could not read response body".to_string(),
            },
            Err(_) => "[ERROR] Could not contact CLI endpoint".to_string(),
        }
    }
}

impl Default for AskUserTool {
    fn default() -> Self {
        AskUserTool::new()
    }
}

impl ToolInstance for AskUserTool {
    fn run(
        &self,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let inquiry = match params.get("inquiry") {
            None => {
                return Err("The parameter \"inquiry\" doesn't exist in the given tool call".into());
            }
            Some(serde_json::Value::String(s)) => s,
            Some(_) => return Err("The parameter \"inquiry\" isn't given as string.".into()),
        };
        let output = self.ask_user(inquiry);
        Ok(serde_json::Value::String(output))
    }

    fn return_choice() -> openai::Tool {
        openai::Tool {
            function: openai::Function {
                name: "ask_user".to_string(),
                description: "a tool for clarification inquiries".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "inquiry": {
                            "type": "string",
                            "description": "the clarification request"
                        }
                    },
                    "required": ["inquiry"]
                }),
            },
            tool_type: "function".to_string()
        }
    }
}
