use serde_json::{json, Value};
use std::fs;
use std::io;

use crate::tools_interface::ToolInstance;
use crate::openai::{Tool, Function};

/// Overwrites the entire contents of a file with the provided content
///
/// If the file does not exist, it will be created. If the file exists, its contents
/// will be replaced with the new content
pub fn edit_file(path: String, content: String) -> io::Result<()> {
    fs::write(&path, content)
}

/// Replaces a specified byte range in a file with new content
///
/// The range is specified as [from, to), where from is inclusive and to is exclusive
pub fn edit_file_from_to(path: String, content: String, from: usize, to: usize) -> io::Result<()> {
    let mut file_content = match fs::read_to_string(&path) {
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

    file_content.replace_range(from_idx..to_idx, &content);

    fs::write(&path, file_content)
}

/// Replaces a specified range in a file with new content, using line and column indices
pub fn edit_file_line_col_range(
    path: String,
    content: String,
    start_line: usize,
    start_col: usize,
    end_line: usize,
    end_col: usize,
) -> std::io::Result<()> {
    let mut file_content = match fs::read_to_string(&path) {
        Ok(data) => data,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(e),
    };

    let lines: Vec<&str> = file_content.lines().collect();

    let start_line = start_line.min(lines.len());
    let end_line = end_line.min(lines.len());

    let mut start_byte = 0;
    let mut end_byte = 0;

    for (i, line) in lines.iter().enumerate() {
        if i < start_line {
            start_byte += line.len() + 1; // +1 for '\n'
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
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "start/end indices are not on valid UTF-8 character boundaries",
        ));
    }

    file_content.replace_range(start_byte..end_byte, &content);

    fs::write(&path, file_content)
}

/// Returns the JSON Schema for the parameters of the `edit_files` tool
pub fn parameters_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "path": {
                "type": "string",
                "description": "Path to the file to edit or create."
            },
            "content": {
                "type": "string",
                "description": "New content to write to the file."
            }
        },
        "required": ["path", "content"]
    })
}

/// Struct implementing the `edit_files` tool for ToolInstance interface
pub struct EditFilesTool;

impl ToolInstance for EditFilesTool {
    fn run(&self, params: Vec<String>) -> Result<String, Box<dyn std::error::Error>> {
        if params.len() != 2 {
            return Err("Expected exactly 2 parameters: path and content.".into());
        }

        let path = &params[0];
        let content = &params[1];

        edit_file(path.clone(), content.clone())?;

        let result = json!({ "status": "success", "path": path }).to_string();
        Ok(result)
    }

    fn return_choice() -> Tool {
        Tool {
            function: Function {
                name: "edit_files".to_string(),
                description: "Overwrites or creates a file with the provided content.".to_string(),
                parameters: parameters_schema(),
            },
            tool_type: "function".to_string(),
        }
    }
}
