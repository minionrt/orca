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