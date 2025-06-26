use crate::openai::{Function, Tool};
use crate::tools_interface::ToolInstance;
use serde_json::json;
use std::fs;

/// Tool to read the contents of a file from disk
pub struct ReadFilesTool;

impl ReadFilesTool {
    pub fn read_file(&self, path: &str) -> std::io::Result<String> {
        fs::read_to_string(path)
    }
}


impl ToolInstance for ReadFilesTool {
    fn run(&self, params: serde_json::Value) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let path = params.get("path")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'path' parameter")?;

        let content = self.read_file(path)?; // ← hier wird sie verwendet

        Ok(serde_json::json!({ "content": content }))
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

        let tool = ReadFilesTool;

        // Übergib ein Objekt mit dem "path"-Feld
        let input = serde_json::json!({ "path": path });

        let output = tool.run(input)
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;

        // Kein from_str mehr nötig – output ist bereits ein serde_json::Value
        Ok(output)
    }
}


