use crate::openai::{Function, Tool};
use crate::tools_interface::ToolInstance;
use serde_json::json;
use std::fs;

/// Tool to read the contents of a file from disk
pub struct ReadFilesTool;

impl ReadFilesTool {
    fn read_file(&self, path: &str) -> std::io::Result<String> {
        fs::read_to_string(path)
    }
}


impl ToolInstance for ReadFilesTool {
    fn run(&self, params: Vec<String>) -> Result<String, Box<dyn std::error::Error>> {
        let path = params.get(0)
            .ok_or("Missing 'path' parameter")?;

        let content = self.read_file(path)?;
        let result = json!({ "content": content });
        Ok(result.to_string())
    }

    fn return_choice() -> Tool {
        Tool {
            tool_type: "function".to_string(),
            function: Function {
                name: "read_files".to_string(),
                description: "Reads the content of a file.".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Path to the file to read."
                        }
                    },
                    "required": ["path"]
                }),
            },
        }
    }
}

impl ReadFilesTool {
    pub fn run_from_value(args: serde_json::Value) -> Result<serde_json::Value, anyhow::Error> {
        // Extrahiere "path" aus JSON
        let path = args.get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing or invalid 'path' parameter"))?;

        // Tool-Instanz anlegen
        let tool = ReadFilesTool;

        // run mit Vec<String> aufrufen
        let output = tool.run(vec![path.to_string()])
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;

        // JSON-String in Value parsen
        let result: serde_json::Value = serde_json::from_str(&output)?;
        Ok(result)
    }
}

