#[cfg(test)]
mod tests {
    use serde_json::{Value, json};
    use teamprojekt_agents::agent_actions::bash::BashTool;
    use teamprojekt_agents::tools_interface::ToolInstance;

    #[test]
    fn test_bash_tool_run_echo() {
        let tool = BashTool::new();
        let output = tool.run(json!({ "code": "echo hello" })).unwrap();
        let output = match output {
            Value::String(s) => s,
            _ => "an error! The return value wasn't a Value::String".to_string(),
        };
        assert!(
            output.contains("hello"),
            "Expected output to contain 'hello', got: {}",
            output
        );
    }

    #[test]
    fn test_bash_tool_run_stderr() {
        let tool = BashTool::new();
        let output = tool.run(json!({"code": "ls /nonexistent_path"})).unwrap();
        let output = match output {
            Value::String(s) => s,
            _ => "an error! The return value wasn't a Value::String".to_string(),
        };
        assert!(
            output.contains("No such file") || output.contains("cannot access"),
            "Expected error message in output, got: {}",
            output
        );
    }

    #[test]
    fn test_bash_tool_return_choice() {
        let tool = BashTool::return_choice();
        assert_eq!(tool.function.name, "bash");
        assert_eq!(tool.tool_type, "function");
        assert!(tool.function.description.contains("bash code"));
        assert!(tool.function.parameters["properties"]["code"]["type"] == "string");
    }
}
