#[cfg(test)]
mod tests {
    use std::fs;
    use std::io::Write;
    use teamprojekt_agents::agent_actions::edit_files::EditFilesTool;
    use tempfile::NamedTempFile;

    fn read_file(path: &std::path::Path) -> String {
        fs::read_to_string(path).expect("File should be readable")
    }

    #[test]
    fn test_edit_file_creates_and_writes() {
        let tool = EditFilesTool;
        let tmp = NamedTempFile::new().unwrap();
        let content = "Hello, world!";
        tool.edit_file(tmp.path().to_str().unwrap(), content)
            .unwrap();
        let read = read_file(tmp.path());
        assert_eq!(read, content, "File content mismatch after create/write");
    }

    #[test]
    fn test_edit_file_overwrites() {
        let tool = EditFilesTool;
        let mut tmp = NamedTempFile::new().unwrap();
        write!(tmp, "Old content").unwrap();
        let new_content = "New content";
        tool.edit_file(tmp.path().to_str().unwrap(), new_content)
            .unwrap();
        let read = read_file(tmp.path());
        assert_eq!(read, new_content, "File content mismatch after overwrite");
    }

    #[test]
    fn test_edit_file_from_to_middle() {
        let tool = EditFilesTool;
        let mut tmp = NamedTempFile::new().unwrap();
        write!(tmp, "abcdefg").unwrap();
        tool.edit_file_from_to(tmp.path().to_str().unwrap(), "XY", 2, 4)
            .unwrap();
        let read = read_file(tmp.path());
        assert_eq!(
            read, "abXYefg",
            "File content mismatch after from_to_middle"
        );
    }

    #[test]
    fn test_edit_file_from_to_out_of_bounds() {
        let tool = EditFilesTool;
        let mut tmp = NamedTempFile::new().unwrap();
        write!(tmp, "abc").unwrap();
        tool.edit_file_from_to(tmp.path().to_str().unwrap(), "XYZ", 1, 10)
            .unwrap();
        let read = read_file(tmp.path());
        assert_eq!(
            read, "aXYZ",
            "File content mismatch after from_to_out_of_bounds"
        );
    }

    #[test]
    fn test_edit_file_from_to_on_new_file() {
        let tool = EditFilesTool;
        let tmp = NamedTempFile::new().unwrap();
        tool.edit_file_from_to(tmp.path().to_str().unwrap(), "Hello", 0, 0)
            .unwrap();
        let read = read_file(tmp.path());
        assert_eq!(
            read, "Hello",
            "File content mismatch after from_to_on_new_file"
        );
    }

    #[test]
    fn test_edit_file_line_col_range_middle() {
        let tool = EditFilesTool;
        let mut tmp = NamedTempFile::new().unwrap();
        write!(tmp, "abc\ndef\nghi\n").unwrap();
        // Replace "d" in "def" (Line 1, Collumn 0) with "XYZ"
        tool.edit_file_line_col_range(tmp.path().to_str().unwrap(), "XYZ", 1, 0, 1, 1)
            .unwrap();
        let read = read_file(tmp.path());
        assert_eq!(
            read, "abc\nXYZef\nghi\n",
            "File content mismatch after line_col_range"
        );
    }

    #[test]
    fn test_edit_file_line_col_range_multiline() {
        let tool = EditFilesTool;
        let mut tmp = NamedTempFile::new().unwrap();
        write!(tmp, "abc\ndef\nghi\n").unwrap();
        tool.edit_file_line_col_range(tmp.path().to_str().unwrap(), "123", 1, 1, 2, 2)
            .unwrap();
        let read = read_file(tmp.path());
        assert_eq!(
            read, "abc\nd123i\n",
            "File content mismatch after multiline line_col_range"
        );
    }

    #[test]
    fn test_edit_file_line_col_range_on_new_file() {
        let tool = EditFilesTool;
        let tmp = NamedTempFile::new().unwrap();
        tool.edit_file_line_col_range(tmp.path().to_str().unwrap(), "Hello\nWorld", 0, 0, 0, 0)
            .unwrap();
        let read = read_file(tmp.path());
        assert_eq!(
            read, "Hello\nWorld",
            "File content mismatch after line_col_range on new file"
        );
    }
}
