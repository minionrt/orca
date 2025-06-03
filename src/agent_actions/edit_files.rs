use std::fs::{self};

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
/// If the file cannot be written, an error message will be printed to stderr.
pub fn edit_file(path: String, content: String) {
    if let Err(e) = fs::write(&path, content) {
        eprintln!("Failed to write file '{}': {}", path, e);
    }
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
/// If the file cannot be written, an error message will be printed to stderr.
pub fn edit_file_from_to(path: String, content: String, from: usize, to: usize) {
    let mut file_content = fs::read_to_string(&path).unwrap_or_default();

    let from_usize = from.max(0);
    let to_usize = to.max(from);

    let file_len = file_content.len();
    let from_idx = from_usize.min(file_len);
    let to_idx = to_usize.min(file_len);

    file_content.replace_range(from_idx..to_idx, &content);

    if let Err(e) = fs::write(&path, file_content) {
        eprintln!("Failed to write file '{}': {}", path, e);
    }
}
