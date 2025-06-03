use std::fs::{self, OpenOptions};
use std::io::{self, Read, Write, Seek, SeekFrom};

// replaces a whole file with new content
pub fn edit_file(path: String, content: String) {
    // Overwrite or create the file with the new content
    if let Err(e) = fs::write(&path, content) {
        eprintln!("Failed to write file '{}': {}", path, e);
    }
}

// only replaces a part of the file from - to
pub fn edit_file_from_to(path: String, content: String, from: i32, to: i32) {
    // Read the existing file content, or create a new file if it doesn't exist
    let mut file_content = match fs::read_to_string(&path) {
        Ok(data) => data,
        Err(_) => String::new(),
    };

    let from_usize = from.max(0) as usize;
    let to_usize = to.max(from) as usize;

    // Ensure indices are within bounds
    let file_len = file_content.len();
    let from_idx = from_usize.min(file_len);
    let to_idx = to_usize.min(file_len);

    // Replace the specified range with the new content
    file_content.replace_range(from_idx..to_idx, &content);

    // Write the modified content back to the file
    if let Err(e) = fs::write(&path, file_content) {
        eprintln!("Failed to write file '{}': {}", path, e);
    }
}