use std::fs;
use std::io;

/// Overwrites the entire contents of a file with the provided content.
///
/// If the file does not exist, it will be created. If the file exists, its contents
/// will be replaced with the new content.
///
/// # Arguments
///
/// * `path` - The path to the file to edit.
/// * `content` - The new content to write to the file.
///
/// # Errors
///
/// Returns an error if the file cannot be written.
pub fn edit_file(path: String, content: String) -> io::Result<()> {
    fs::write(&path, content)
}

/// Replaces a specified byte range in a file with new content.
///
/// If the file does not exist, it will be created as an empty file before the operation.
/// The range is specified as `[from, to)`, where `from` is inclusive and `to` is exclusive.
/// If the range exceeds the file's length, it will be clamped to the file's bounds.
///
/// # Arguments
///
/// * `path` - The path to the file to edit.
/// * `content` - The content to insert into the specified range.
/// * `from` - The starting byte index (inclusive) of the range to replace.
/// * `to` - The ending byte index (exclusive) of the range to replace.
///
/// # Errors
///
/// Returns an error if the file cannot be read or written.
pub fn edit_file_from_to(path: String, content: String, from: usize, to: usize) -> io::Result<()> {
    let mut file_content = match fs::read_to_string(&path) {
        Ok(data) => data,
        Err(e) if e.kind() == io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(e),
    };

    let file_len = file_content.len();
    let from_idx = from.clamp(0, file_len);
    let to_idx = to.clamp(from_idx, file_len);

    // Ensure indices are on char boundaries
    if !file_content.is_char_boundary(from_idx) || !file_content.is_char_boundary(to_idx) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "from/to indices are not on valid UTF-8 character boundaries",
        ));
    }

    file_content.replace_range(from_idx..to_idx, &content);

    fs::write(&path, file_content)
}

/// Replaces a specified range in a file with new content, using line and column indices.
/// Intended for use with LSP integration
///
/// If the file does not exist, it will be created as an empty file before the operation.
/// The range is specified as (start_line, start_col) to (end_line, end_col), all zero-based and inclusive/exclusive respectively.
/// If the range exceeds the file's bounds, it will be clamped.
///
/// # Arguments
///
/// * `path` - The path to the file to edit.
/// * `content` - The content to insert into the specified range.
/// * `start_line` - The starting line index (zero-based).
/// * `start_col` - The starting column index (zero-based, in chars).
/// * `end_line` - The ending line index (zero-based).
/// * `end_col` - The ending column index (zero-based, in chars).
///
/// # Errors
///
/// Returns an error if the file cannot be read or written, or if indices are invalid.
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

    // Add column offsets (in chars, not bytes)
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

    // Ensure char boundaries
    if !file_content.is_char_boundary(start_byte) || !file_content.is_char_boundary(end_byte) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "start/end indices are not on valid UTF-8 character boundaries",
        ));
    }

    file_content.replace_range(start_byte..end_byte, &content);

    fs::write(&path, file_content)
}
