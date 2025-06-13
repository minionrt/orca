use std::io::Write;
use teamprojekt_agents::agent_actions::read_files::read_file;
use tempfile::NamedTempFile;

#[test]
fn test_read_file_existing() {
    let mut tmpfile = NamedTempFile::new().expect("Failed to create temp file");
    let content = "Test content for reading.";
    write!(tmpfile, "{}", content).expect("Failed to write to temp file");

    let result = read_file(tmpfile.path().to_str().unwrap().to_string());
    assert_eq!(result, Some(content.to_string()), "Content mismatch");
}

#[test]
fn test_read_file_nonexistent() {
    // tempfile creates real files -> for nonexistent I simulate it with a fake-Pfad
    let nonexistent_path = "/tmp/clearly_nonexistent_file_123456789.txt";
    let result = read_file(nonexistent_path.to_string());
    assert_eq!(result, None, "Expected None for nonexistent file");
}

#[test]
fn test_read_file_empty() {
    let tmpfile = NamedTempFile::new().expect("Failed to create temp file");
    // Empty file -> directly created, no write needed

    let result = read_file(tmpfile.path().to_str().unwrap().to_string());
    assert_eq!(result, Some(String::new()), "Expected empty string");
}
