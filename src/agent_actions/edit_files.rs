use crate::openai::{Function, Tool};
use crate::tools_interface::ToolInstance;
use serde_json::json;
use std::fs;
use std::io;

pub struct EditFilesTool;

impl EditFilesTool {
    fn edit_file(&self, path: &str, content: &str) -> io::Result<()> {
        fs::write(path, content)
    }

    fn edit_file_from_to(&self, path: &str, content: &str, from: usize, to: usize) -> io::Result<()> {
        let mut file_content = match fs::read_to_string(path) {
            Ok(data) => data,
            Err(e) if e.kind() == io::ErrorKind::NotFound => String::new(),
            Err(e) => return Err(e),
        };

        let file_len = file_content.len();
        let from_idx = from.clamp(0, file_len);
        let to_idx = to.clamp(from_idx, file_len);

        if !file_content.is_char_boundary(from_idx) || !file_content.is_char_boundary(to_idx) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "from/to indices are not on valid UTF-8 character boundaries",
            ));
        }

        file_content.replace_range(from_idx..to_idx, content);

        fs::write(path, file_content)
    }

    fn edit_file_line_col_range(
        &self,
        path: &str,
        content: &str,
        start_line: usize,
        start_col: usize,
        end_line: usize,
        end_col: usize,
    ) -> io::Result<()> {
        let mut file_content = match fs::read_to_string(path) {
            Ok(data) => data,
            Err(e) if e.kind() == io::ErrorKind::NotFound => String::new(),
            Err(e) => return Err(e),
        };

        let lines: Vec<&str> = file_content.lines().collect();

        let start_line = start_line.min(lines.len());
        let end_line = end_line.min(lines.len());

        let mut start_byte = 0;
        let mut end_byte = 0;

        for (i, line) in lines.iter().enumerate() {
            if i < start_line {
                start_byte += line.len() + 1; // +1 für '\n'
            }
            if i < end_line {
                end_byte += line.len() + 1;
            }
        }

        if start_line < lines.len() {
            let line = lines[start_line];
            let col = start_col.min(line.chars().count());
            start_byte += line.chars().take(col).map(|c| c.len_utf8()).sum::<usize>();
        }
        if end_line < lines.len() {
            let line = lines[end_line];
            let col = end_col.min(line.chars().count());
            end_byte += line.chars().take(col).map(|c| c.len_utf8()).sum::<usize>();
        }

        if !file_content.is_char_boundary(start_byte) || !file_content.is_char_boundary(end_byte) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "start/end indices are not on valid UTF-8 character boundaries",
            ));
        }

        file_content.replace_range(start_byte..end_byte, content);

        fs::write(path, file_content)
    }
}

impl ToolInstance for EditFilesTool {
    fn run(&self, params: Vec<String>) -> Result<String, Box<dyn std::error::Error>> {
        // Expects path and content as must-have
        let path = params.get(0).ok_or("Missing 'path' parameter")?;
        let content = params.get(1).ok_or("Missing 'content' parameter")?;

        // optional: parameter for from/to or line/col range parsen
        // expectation: optional more parameters can be added/followed, e.g.
        // params[2] = from (usize) or start_line
        // params[3] = to (usize) or start_col
        // params[4] = end_line
        // params[5] = end_col

        // call different methods depending on the number and type of parameters

        match params.len() {
            2 => {
                // only path + content => overwrites whole file
                self.edit_file(path, content)?;
            }
            4 => {
                // path, content, from, to => replace byte range 
                let from = params[2].parse::<usize>()?;
                let to = params[3].parse::<usize>()?;
                self.edit_file_from_to(path, content, from, to)?;
            }
            6 => {
                // path, content, start_line, start_col, end_line, end_col => replace line/col range 
                let start_line = params[2].parse::<usize>()?;
                let start_col = params[3].parse::<usize>()?;
                let end_line = params[4].parse::<usize>()?;
                let end_col = params[5].parse::<usize>()?;
                self.edit_file_line_col_range(path, content, start_line, start_col, end_line, end_col)?;
            }
            _ => {
                return Err("Invalid number of parameters for edit_files".into());
            }
        }

        let result = json!({ "status": "success" });
        Ok(result.to_string())
    }

    fn return_choice() -> Tool {
        Tool {
            tool_type: "function".to_string(),
            function: Function {
                name: "edit_files".to_string(),
                description: "Edits the contents of a file, optionally by range.".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "path": { "type": "string", "description": "Path to the file." },
                        "content": { "type": "string", "description": "Content to write." },
                        "from": { "type": "integer", "description": "Optional start byte index." },
                        "to": { "type": "integer", "description": "Optional end byte index." },
                        "start_line": { "type": "integer", "description": "Optional start line index (0-based)." },
                        "start_col": { "type": "integer", "description": "Optional start column index (0-based)." },
                        "end_line": { "type": "integer", "description": "Optional end line index (0-based)." },
                        "end_col": { "type": "integer", "description": "Optional end column index (0-based)." }
                    },
                    "required": ["path", "content"]
                }),
            },
        }
    }
}

impl EditFilesTool {
    pub fn run_from_value(args: serde_json::Value) -> Result<serde_json::Value, anyhow::Error> {
        let path = args.get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing or invalid 'path' parameter"))?;

        let content = args.get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing or invalid 'content' parameter"))?;

        // Versuche optionale Parameter auszulesen
        let from = args.get("from").and_then(|v| v.as_u64()).map(|v| v as usize);
        let to = args.get("to").and_then(|v| v.as_u64()).map(|v| v as usize);

        let start_line = args.get("start_line").and_then(|v| v.as_u64()).map(|v| v as usize);
        let start_col = args.get("start_col").and_then(|v| v.as_u64()).map(|v| v as usize);
        let end_line = args.get("end_line").and_then(|v| v.as_u64()).map(|v| v as usize);
        let end_col = args.get("end_col").and_then(|v| v.as_u64()).map(|v| v as usize);

        let tool = EditFilesTool;

        // call different run methods according to parameters
        let output = if let (Some(from), Some(to)) = (from, to) {
            tool.edit_file_from_to(path, content, from, to)?;
            json!({ "status": "success" }).to_string()
        } else if let (Some(start_line), Some(start_col), Some(end_line), Some(end_col)) = (start_line, start_col, end_line, end_col) {
            tool.edit_file_line_col_range(path, content, start_line, start_col, end_line, end_col)?;
            json!({ "status": "success" }).to_string()
        } else {
            tool.edit_file(path, content)?;
            json!({ "status": "success" }).to_string()
        };

        let result: serde_json::Value = serde_json::from_str(&output)?;
        Ok(result)
    }
}
