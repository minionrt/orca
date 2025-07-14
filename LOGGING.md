# Logging Setup

This project uses the `tracing` ecosystem for structured logging with the following features:

## Features

- **Console Logging**: Pretty-printed logs to stdout/stderr with colors
- **File Logging**: Daily rotating log files in the `logs/` directory
- **Structured Logging**: JSON-compatible structured logging with spans and events
- **Environment Configuration**: Log levels can be controlled via environment variables

## Usage

### Basic Logging

```rust
use tracing::{info, warn, error, debug};

fn example() {
    debug!("Debug information");
    info!("General information");
    warn!("Warning message");
    error!("Error occurred: {}", error_msg);
}
```

### Structured Logging with Fields

```rust
use tracing::{info, info_span};

fn process_task(task_id: &str) {
    let _span = info_span!("process_task", task_id = task_id).entered();
    
    info!(status = "started", "Processing task");
    // ... do work ...
    info!(status = "completed", duration_ms = 1234, "Task completed");
}
```

## Configuration

### Environment Variables

Set the log level using the `RUST_LOG` environment variable:

```bash
# Show all logs
export RUST_LOG=debug

# Show only info and above
export RUST_LOG=info

# Show only warnings and errors
export RUST_LOG=warn

# Module-specific logging
export RUST_LOG=teamprojekt_agents::agent_actions=debug,info
```

### Log Files

- Log files are stored in `logs/`
- Files rotate daily (e.g., `agent.log.2025-01-14`)
- Old log files are automatically kept for reference

## Initialization

The logging system is initialized in `main.rs`:

```rust
fn main() {
    // Initialize logging first thing
    logging::init_logging();
    
    // ... rest of application
}
```

For tests, use the simpler console-only version:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_something() {
        crate::logging::init_simple_logging();
        // ... test code with logging
    }
}
```

## Best Practices

1. **Use appropriate log levels**:
   - `error!`: For errors that need immediate attention
   - `warn!`: For problems that don't stop execution
   - `info!`: For general application flow
   - `debug!`: For detailed debugging information

2. **Include context**:
   ```rust
   info!(user_id = user.id, action = "login", "User logged in");
   ```

3. **Use spans for operations**:
   ```rust
   let _span = info_span!("database_query", table = "users").entered();
   ```

4. **Don't log sensitive information**:
   ```rust
   // Bad
   debug!("User password: {}", password);
   
   // Good
   debug!(user_id = user.id, "User authentication attempt");
   ```
