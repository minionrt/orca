# Logging Setup

This project uses `tracing` for console logging.

## Usage

```rust
use tracing::{info, warn, error, debug};

fn example() {
    debug!("Debug information");
    info!("General information");
    warn!("Warning message");
    error!("Error occurred: {}", error_msg);
}
```

## Configuration

Set the log level using the `RUST_LOG` environment variable:

```bash
# Show all logs
export RUST_LOG=debug

# Show only info and above (default)
export RUST_LOG=info

# Show only warnings and errors
export RUST_LOG=warn
```

## Log Levels

- `error!`: For errors that need immediate attention
- `warn!`: For problems that don't stop execution
- `info!`: For general application flow (what the agent is doing)
- `debug!`: For detailed debugging (JSON requests/responses, etc.)
