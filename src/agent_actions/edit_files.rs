use crate::openai;
use crate::tools_interface::ToolInstance;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::io;

/// Tool for editing files with different methods: full replacement,
/// byte-range replacement, or line-column range replacement
pub struct EditFilesTool;

impl EditFilesTool {
    /// Replaces the entire contents of a file with the given content
    pub fn edit_file(path: &str, content: &str) -> io::Result<()> {
        fs::write(path, content)
    }

    /// Replaces a part of the file between byte indices `from` and `to`
    /// with the provided content. If the file does not exist, it is created
    pub fn edit_file_from_to(path: &str, content: &str, from: usize, to: usize) -> io::Result<()> {
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

#[derive(Deserialize)]
pub struct EditFilesToolArgs {
    path: String,
    content: String,

    #[serde(default)]
    from: Option<usize>,

    #[serde(default)]
    to: Option<usize>,

    #[serde(default)]
    start_line: Option<usize>,

    #[serde(default)]
    end_line: Option<usize>,

    #[serde(default)]
    start_col: Option<usize>,

    #[serde(default)]
    end_col: Option<usize>,
}

/// Runs the tool based on JSON parameters. Supports:
/// - Full file replacement,
/// - Byte-range replacement,
/// - Line-column range replacement.
impl ToolInstance for EditFilesTool {
    type Args = EditFilesToolArgs;
    type Out = ();
    fn run(args: EditFilesToolArgs) -> Result<(), Box<dyn std::error::Error>> {
        let mut path = args.path.clone();
        let content = &args.content;

        if let (Some(from), Some(to)) = (args.from, args.to) {
            Self::edit_file_from_to(&path, content, from, to)?;
        } else if let (Some(start_line), Some(start_col), Some(end_line), Some(end_col)) =
            (args.start_line, args.start_col, args.end_line, args.end_col)
        {
            Self::edit_file_line_col_range(
                &path, content, start_line, start_col, end_line, end_col,
            )?;
        } else {
            Self::edit_file(&path, content)?;
        }

        Ok(())
    }

    /// Returns the tool definition with a JSON schema for LLM integration
    fn return_choice() -> openai::Tool {
        openai::Tool::function(
            "edit_files".to_owned(),
            "Edits the contents of a file, optionally by range".to_owned(),
            HashMap::from([
                (
                    "path".to_owned(),
                    openai::FunctionParameter::new("string", "Path to the file."),
                ),
                (
                    "content".to_owned(),
                    openai::FunctionParameter::new("string", "Content to write"),
                ),
            ]),
            HashMap::from([
                (
                    "from".to_owned(),
                    openai::FunctionParameter::new("integer", "Optional start byte index"),
                ),
                (
                    "to".to_owned(),
                    openai::FunctionParameter::new("integer", "Optional end byte index"),
                ),
                (
                    "start_line".to_owned(),
                    openai::FunctionParameter::new(
                        "integer",
                        "Optional start line index (0-based)",
                    ),
                ),
                (
                    "end_line".to_owned(),
                    openai::FunctionParameter::new("integer", "Optional end line index (0-based)"),
                ),
                (
                    "start_col".to_owned(),
                    openai::FunctionParameter::new(
                        "string",
                        "Optional start column index (0-based)",
                    ),
                ),
                (
                    "end_col".to_owned(),
                    openai::FunctionParameter::new(
                        "integer",
                        "Optional end column index (0-based)",
                    ),
                ),
            ]),
        )
    }
}
