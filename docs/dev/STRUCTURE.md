# Project Structure

## Overview

This agent fetches tasks from the minion API, clones target repositories, and executes requested changes using LLM-powered tools.

## Core Modules

### Entry Point
- `main.rs` - Initializes logging, fetches tasks, handles repository setup

### Task Processing
- `task_handler.rs` - Coordinates task execution
- `fetch_task.rs` - Retrieves tasks from minion API
- `models.rs` - Data structures for API communication

### Repository Management
- `repo_clone.rs` - Handles git operations and workspace setup
- `memory.rs` - Manages context and state during execution

### LLM Integration
- `llm.rs` - Core LLM interaction logic
- `openai.rs` - OpenAI API implementation

### Tool System
- `tools/` - Tool definitions and collections
- `tools_interface.rs` - Tool execution interface
- `agent_actions/` - Specific agent capabilities:
  - `bash.rs` - Shell command execution
  - `edit_files.rs` - File modification operations
  - `read_files.rs` - File reading operations
  - `dir_tree.rs` - Directory structure analysis
  - `submit_code.rs` - Code submission handling

### Infrastructure
- `logging.rs` - Tracing setup and configuration
- `report.rs` - Success/failure reporting to minion API

## Flow

1. Agent starts, initializes logging
2. Fetches task from minion API
3. Clones target repository with auth
4. Creates task context with working directory
5. Executes task using available tools
6. Reports outcome back to API

## Testing

Tests are in `tests/` directory, covering individual modules and integration scenarios.
