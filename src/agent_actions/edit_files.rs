use crate::openai::{Function, Tool};
use crate::tools_interface::ToolInstance;
use serde_json::json;
use std::fs;
use std::io;

/// Tool for editing files with different methods: full replacement,
/// byte-range replacement, or line-column range replacement
pub struct EditFilesTool;

impl EditFilesTool {
    /// Replaces the entire contents of a file with the given content
    pub fn edit_file(&self, path: &str, content: &str) -> io::Result<()> {
        fs::write(path, content)
    }

    /// Replaces a part of the file between byte indices `from` and `to`
    /// with the provided content. If the file does not exist, it is created
    pub fn edit_file_from_to(
        &self,
        path: &str,
        content: &str,
        from: usize,
        to: usize,
    ) -> io::Result<()> {
        // Read existing file contents, or start with empty content if the file doesn't exist
        let mut file_content = match fs::read_to_string(path) {
            Ok(data) => data,
            Err(e) if e.kind() == io::ErrorKind::NotFound => String::new(),
            Err(e) => return Err(e),
        };

        // Clamp the indices to valid bounds within the file
        let file_len = file_content.len();
        let from_idx = from.clamp(0, file_len);
        let to_idx = to.clamp(from_idx, file_len);

        // Ensure indices are at valid UTF-8 character boundaries
        if !file_content.is_char_boundary(from_idx) || !file_content.is_char_boundary(to_idx) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "from/to indices are not on valid UTF-8 character boundaries",
            ));
        }

        // Replace the byte range with the new content
        file_content.replace_range(from_idx..to_idx, content);

        fs::write(path, file_content)
    }

    /// Replaces a part of the file determined by a line-column range with the given content
    /// Line and column numbers are 0-based
    pub fn edit_file_line_col_range(
        &self,
        path: &str,
        content: &str,
        start_line: usize,
        start_col: usize,
        end_line: usize,
        end_col: usize,
    ) -> io::Result<()> {
        // Read existing file contents, or start with empty content if the file doesn't exist
        let mut file_content = match fs::read_to_string(path) {
            Ok(data) => data,
            Err(e) if e.kind() == io::ErrorKind::NotFound => String::new(),
            Err(e) => return Err(e),
        };

        // Split the file into lines
        let lines: Vec<&str> = file_content.lines().collect();

        let start_line = start_line.min(lines.len());
        let end_line = end_line.min(lines.len());

        // Calculate byte offsets for the start and end positions
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

        // Add columns within the start and end lines
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

        // Ensure the byte offsets are valid UTF-8 character boundaries
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

/// Runs the tool based on JSON parameters. Supports:
/// - Full file replacement,
/// - Byte-range replacement,
/// - Line-column range replacement.
impl ToolInstance for EditFilesTool {
    fn run(
        &self,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let path = params
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or("Missing or invalid 'path' parameter")?;

        let content = params
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or("Missing or invalid 'content' parameter")?;

        let from = params
            .get("from")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize);
        let to = params
            .get("to")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize);

        let start_line = params
            .get("start_line")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize);
        let start_col = params
            .get("start_col")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize);
        let end_line = params
            .get("end_line")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize);
        let end_col = params
            .get("end_col")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize);

        if let (Some(from), Some(to)) = (from, to) {
            self.edit_file_from_to(path, content, from, to)?;
        } else if let (Some(start_line), Some(start_col), Some(end_line), Some(end_col)) =
            (start_line, start_col, end_line, end_col)
        {
            self.edit_file_line_col_range(path, content, start_line, start_col, end_line, end_col)?;
        } else {
            self.edit_file(path, content)?;
        }

        Ok(json!({ "status": "success" }))
    }

    /// Returns the tool definition with a JSON schema for LLM integration
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