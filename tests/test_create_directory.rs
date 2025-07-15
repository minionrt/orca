#[cfg(test)]
mod tests {
    use teamprojekt_agents::agent_actions::create_directory::{CreateDirectoryTool, create_directory};
    use teamprojekt_agents::tools_interface::ToolInstance;
    use tempfile::TempDir;

    #[test]
    fn test_create_directory_tool() {
        let temp_dir = TempDir::new().unwrap();
        let test_path = temp_dir.path().join("test_dir");
        
        let tool = CreateDirectoryTool::new();
        let params = serde_json::json!({
            "path": test_path.to_string_lossy()
        });
        
        let result = tool.run(params).unwrap();
        
        assert!(test_path.exists());
        assert!(test_path.is_dir());
        assert!(result.as_str().unwrap().contains("Directory created"));
    }

    #[test]
    fn test_create_directory_nested() {
        let temp_dir = TempDir::new().unwrap();
        let test_path = temp_dir.path().join("parent").join("child").join("grandchild");
        
        let result = create_directory(&test_path.to_string_lossy());
        
        assert!(result.is_ok());
        assert!(test_path.exists());
        assert!(test_path.is_dir());
    }

    #[test]
    fn test_create_directory_already_exists() {
        let temp_dir = TempDir::new().unwrap();
        let test_path = temp_dir.path().join("existing_dir");
        
        // Create directory first
        std::fs::create_dir_all(&test_path).unwrap();
        
        // Try to create again - should not fail
        let result = create_directory(&test_path.to_string_lossy());
        
        assert!(result.is_ok());
        assert!(test_path.exists());
        assert!(test_path.is_dir());
    }

    #[test]
    fn test_create_directory_tool_return_choice() {
        let tool = CreateDirectoryTool::return_choice();
        assert_eq!(tool.function.name, "create_directory");
        assert_eq!(tool.tool_type, "function");
        assert!(tool.function.description.contains("Creates a directory"));
    }
}
