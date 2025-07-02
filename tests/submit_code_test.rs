/* #[cfg(test)]
mod tests {
    use teamprojekt_agents::agent_actions::submit_code::GitSubmissionTool;
    use teamprojekt_agents::tools_interface::ToolInstance;
    use serde_json::json;

    #[test]
    fn test_git_add_action() {
        let tool = GitSubmissionTool::default();
        let result = tool.run(json!({ "action": "add" }));
        assert!(result.is_ok());
        assert!(result.unwrap().as_str().unwrap().contains("git add"));
    }

    #[test]
    fn test_git_push_action() {
        let tool = GitSubmissionTool::default();
        let result = tool.run(json!({ "action": "push" }));
        assert!(result.is_ok());
        assert!(result.unwrap().as_str().unwrap().contains("git push"));
    }
} */
