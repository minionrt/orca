#[cfg(test)]
mod tests {
    use teamprojekt_agents::agent_actions::bash::run_bash;

    #[test]
    fn test_run_bash_echo() {
        let output = run_bash("echo hello").unwrap();
        assert!(output.contains("hello"));
    }

    #[test]
    fn test_run_bash_stderr() {
        let output = run_bash("ls /nonexistent_path").unwrap();
        assert!(output.contains("No such file") || output.contains("cannot access"));
    }
}