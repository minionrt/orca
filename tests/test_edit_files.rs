use std::fs;
use std::io::Write;
use tempfile::NamedTempFile;
use teamprojekt_agents::agent_actions::edit_files::{
    edit_file, edit_file_from_to, edit_file_line_col_range,
};

fn read_file(path: &std::path::Path) -> String {
    fs::read_to_string(path).expect("File should be readable")
}

#[test]
fn test_edit_file_creates_and_writes() {
    let tmp = NamedTempFile::new().unwrap();
    let content = "Hello, world!";
    edit_file(tmp.path().to_string_lossy().to_string(), content.to_string()).unwrap();
    let read = read_file(tmp.path());
    assert_eq!(read, content, "File content mismatch after create/write");
}

#[test]
fn test_edit_file_overwrites() {
    let mut tmp = NamedTempFile::new().unwrap();
    write!(tmp, "Old content").unwrap();
    let new_content = "New content";
    edit_file(tmp.path().to_string_lossy().to_string(), new_content.to_string()).unwrap();
    let read = read_file(tmp.path());
    assert_eq!(read, new_content, "File content mismatch after overwrite");
}

#[test]
fn test_edit_file_from_to_middle() {
    let mut tmp = NamedTempFile::new().unwrap();
    write!(tmp, "abcdefg").unwrap();
    edit_file_from_to(tmp.path().to_string_lossy().to_string(), "XY".to_string(), 2, 4).unwrap();
    let read = read_file(tmp.path());
    assert_eq!(read, "abXYefg", "File content mismatch after from_to_middle");
}

#[test]
fn test_edit_file_from_to_out_of_bounds() {
    let mut tmp = NamedTempFile::new().unwrap();
    write!(tmp, "abc").unwrap();
    edit_file_from_to(tmp.path().to_string_lossy().to_string(), "XYZ".to_string(), 1, 10).unwrap();
    let read = read_file(tmp.path());
    assert_eq!(read, "aXYZ", "File content mismatch after from_to_out_of_bounds");
}

#[test]
fn test_edit_file_from_to_on_new_file() {
    let tmp = NamedTempFile::new().unwrap();
    edit_file_from_to(tmp.path().to_string_lossy().to_string(), "Hello".to_string(), 0, 0).unwrap();
    let read = read_file(tmp.path());
    assert_eq!(read, "Hello", "File content mismatch after from_to_on_new_file");
}

#[test]
fn test_edit_file_line_col_range_middle() {
    let mut tmp = NamedTempFile::new().unwrap();
    write!(tmp, "abc\ndef\nghi\n").unwrap();
    // Replace "d" in "def" (line 1, col 0) with "XYZ"
    edit_file_line_col_range(
        tmp.path().to_string_lossy().to_string(),
        "XYZ".to_string(),
        1, 0, 1, 1,
    )
    .unwrap();
    let read = read_file(tmp.path());
    assert_eq!(read, "abc\nXYZef\nghi\n", "File content mismatch after line_col_range");
}

#[test]
fn test_edit_file_line_col_range_multiline() {
    let mut tmp = NamedTempFile::new().unwrap();
    write!(tmp, "abc\ndef\nghi\n").unwrap();
    edit_file_line_col_range(
        tmp.path().to_string_lossy().to_string(),
        "123".to_string(),
        1, 1, 2, 2,
    )
    .unwrap();
    let read = read_file(tmp.path());
    assert_eq!(read, "abc\nd123i\n", "File content mismatch after multiline line_col_range");
}

#[test]
fn test_edit_file_line_col_range_on_new_file() {
    let tmp = NamedTempFile::new().unwrap();
    edit_file_line_col_range(
        tmp.path().to_string_lossy().to_string(),
        "Hello\nWorld".to_string(),
        0, 0, 0, 0,
    )
    .unwrap();
    let read = read_file(tmp.path());
    assert_eq!(read, "Hello\nWorld", "File content mismatch after line_col_range on new file");
}