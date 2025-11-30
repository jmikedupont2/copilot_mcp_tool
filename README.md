# copilot_mcp_tool

**[Development Plan and Status](plan.md)**

`copilot_mcp_tool` is a Rust-based Model Context Protocol (MCP) server designed to act as a bridge between a Large Language Model (LLM) like Gemini (via `copilot-client`) and a chain of nested tools. This project demonstrates how to create a multi-level nested tool calling system using the MCP.

## Project Structure

This project has been refactored into a client-daemon architecture, managed by a single binary: `copilot_mcp_tool`.

*   **`copilot_mcp_tool` (main binary):** This is the core application, acting as both a server and a client.
    *   **Server Mode:** When started with `start`, it runs as a detached background process, listening for MCP connections.
    *   **Client Mode:** When used with commands like `list` or `call`, it connects to the running server instance to execute commands.
*   **`mcp_web_client` (binary):** A simple web-based GUI that connects to the `copilot_mcp_tool` server to provide a user interface for calling tools.
*   **Nested Tool Modules:** The project still contains the `WeatherTool`, `TimeTool`, and `EchoTool` to demonstrate multi-level nested tool calling.

## Prerequisites

Before you begin, ensure you have the following installed:

*   **Rust and Cargo:** [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)
*   **GitHub CLI (`gh`):** [https://cli.github.com/](https://cli.github.com/) (Ensure you are logged in using `gh auth login`).
*   **A GitHub Token:** A Personal Access Token (PAT) with `copilot` scope (or sufficient scopes for Copilot access) is required.

## Installation and Setup

1.  **Clone the repository:**

    ```bash
    git clone https://github.com/jmikedupont2/copilot_mcp_tool.git
    cd copilot_mcp_tool
    ```

2.  **Set your GitHub Token:**
    The project relies on a `GITHUB_TOKEN` environment variable for authentication with the Copilot API.

    To set this up, you can extract your GitHub CLI token (which usually has the necessary scopes) and save it to a `.env` file.

    ```bash
    # Extract your GitHub token using gh CLI and save to .env
    # Note: This requires gh CLI to be logged in.
    # On Windows PowerShell:
    (gh auth token).Trim() | Set-Content -Path .\.env -Value { "GITHUB_TOKEN=" + $_ }

    # On Linux/macOS or Git Bash:
    # gh auth token > .env
    # echo "GITHUB_TOKEN=$(cat .env)" > .env
    # Note: The above two lines are a quick hack. A safer way is:
    # echo "GITHUB_TOKEN=$(gh auth token)" > .env
    ```
    **Important:** For the `GITHUB_TOKEN` to be picked up by `cargo run`, you *must* ensure it's loaded into your shell's environment before executing commands.
    *   **Manually (Windows Command Prompt):** `set GITHUB_TOKEN=YOUR_TOKEN_HERE`
    *   **Manually (Windows PowerShell):** `$env:GITHUB_TOKEN="YOUR_TOKEN_HERE"`
    *   **Manually (Linux/macOS/Git Bash):** `export GITHUB_TOKEN="YOUR_TOKEN_HERE"`
    *   **Using `direnv` (recommended for development):** Install `direnv` and add `eval "$(direnv hook bash)"` (or your shell equivalent) to your shell config. Then, simply put `export GITHUB_TOKEN=YOUR_TOKEN_HERE` in your `.envrc` file, or if you prefer to reuse the `.env` file created above, put `source .env` in your `.envrc`.

3.  **Build the project:**

    ```bash
    cargo build
    ```

## Usage

The `copilot_mcp_tool` is a command-line tool to manage the MCP server and interact with it.

### Managing the Server

First, build the project:
```bash
cargo build
```

**Check the Server Status:**
By default, running the tool with no arguments shows the server status.
```bash
cargo run --bin copilot_mcp_tool
# Or, more explicitly:
cargo run --bin copilot_mcp_tool -- status
# Output: Server is STOPPED.
```

**Start the Server:**
This launches the server as a background process.
```bash
cargo run --bin copilot_mcp_tool -- start
# Output:
# Server starting in background...
# Server is RUNNING on port 58361 (PID: 12345).
```

**Stop the Server:**
```bash
cargo run --bin copilot_mcp_tool -- stop
# Output:
# INFO copilot_mcp_tool: Stopping server (PID: 12345)...
# INFO copilot_mcp_tool: Server stopped.
```

### Interacting with the Server

Once the server is running, you can use the client commands.

**List Available Tools:**
```bash
cargo run --bin copilot_mcp_tool -- list
```
This will connect to the running server and print a JSON list of the available tools (`echo_message` and `kill_process`).

**Call a Tool:**
The `call` command uses a `tool_name` followed by key-value pairs for parameters.
```bash
# Example with a simple message
cargo run --bin copilot_mcp_tool -- call echo_message message=hello

# Example with spaces in the value (use quotes)
cargo run --bin copilot_mcp_tool -- call echo_message message="hello world"
```
The tool will connect to the server, execute the command, and print the JSON result.

### Using the Web GUI

The project also includes a simple web client.

1.  **Start the MCP server:**
    ```bash
    cargo run --bin copilot_mcp_tool -- start
    ```

2.  **Run the web client:**
    In a separate terminal:
    ```bash
    cargo run --bin mcp_web_client
    ```
    The web client will start and listen on `http://localhost:3000`.

3.  **Open your browser** and navigate to `http://localhost:3000` to interact with the server through a web interface.

## Contributing

Feel free to open issues or pull requests to improve this demonstration of nested MCP tool calling.
