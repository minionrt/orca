#[cfg(test)]
mod tests {
    use teamprojekt_agents::agent_actions::bash::{BashTool, BashToolArgs};
    use teamprojekt_agents::tools_interface::ToolInstance;

    #[test]
    fn test_bash_tool_run_echo() {
        let output = BashTool::run(BashToolArgs {
            code: "echo hello".to_owned(),
            working_dir: ".".to_owned(),
        })
        .unwrap();
        assert!(
            output.contains("hello"),
            "Expected output to contain 'hello', got: {output}"
        );
    }

    #[test]
    fn test_bash_tool_run_stderr() {
        let output = BashTool::run(BashToolArgs {
            code: "ls /nonexistent_path".to_owned(),
            working_dir: ".".to_owned(),
        })
        .unwrap();
        assert!(
            output.contains("No such file") || output.contains("cannot access"),
            "Expected error message in output, got: {output}"
        );
    }

    #[test]
    fn test_bash_tool_return_choice() {
        let tool = BashTool::return_choice();
        assert_eq!(tool.function.name, "bash");
        assert_eq!(tool.tool_type, "function");
        assert!(tool.function.description.contains("bash code"));
        assert_eq!(
            tool.function.parameters.properties["code"].param_type,
            "string"
        );
    }
}
