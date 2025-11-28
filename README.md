# copilot_mcp_tool

`copilot_mcp_tool` is a Rust-based Model Context Protocol (MCP) server designed to act as a bridge between a Large Language Model (LLM) like Gemini (via `copilot-client`) and a chain of nested tools. This project demonstrates how to create a multi-level nested tool calling system using the MCP.

## Project Structure

This project consists of:

*   **`copilot_mcp_tool` (main binary):** The core MCP server. It exposes a `copilot_suggest` tool. When this tool is invoked, it communicates with the GitHub Copilot API (via the `copilot-client` crate). Copilot is prompted to suggest a tool call in JSON format. This server then parses Copilot's suggestion and executes the corresponding tool in a nested fashion.
*   **`tool_server_module`:** A module within `copilot_mcp_tool` that provides a `WeatherTool` with a `get_weather` function.
*   **`level2_tool_module`:** A module within `copilot_mcp_tool` that provides a `TimeTool` with a `get_time_in_location` function. This tool is called by `WeatherTool`.
*   **`level3_tool_module`:** A module within `copilot_mcp_tool` that provides an `EchoTool` with an `echo` function. This tool is called by `TimeTool`.
*   **`mcp_client_caller` (binary):** A client binary that launches `copilot_mcp_tool` as a child process and calls its `copilot_suggest` tool with a sample prompt, demonstrating the end-to-end functionality.

## Features

*   **Multi-level Nested Tool Calls:** Demonstrates how one tool can invoke another, creating a chain of execution.
*   **GitHub Copilot Integration:** Utilizes the `copilot-client` crate to interact with the GitHub Copilot API for tool suggestion.
*   **Model Context Protocol (MCP) Server:** Implements the MCP for structured communication.

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

## Usage with Gemini CLI

The `copilot_mcp_tool` acts as an MCP server. To integrate it with Gemini CLI, you would typically run it and then configure Gemini CLI to connect to it.

1.  **Run the `copilot_mcp_tool` server:**

    ```bash
    cargo run
    ```
    This will start the MCP server, listening for incoming requests, and internally setting up the nested `WeatherTool`, `TimeTool`, and `EchoTool`.

2.  **Configure Gemini CLI (Conceptual):**
    *   Once the `copilot_mcp_tool` is running, it exposes a MCP endpoint (likely over standard I/O if run directly).
    *   Your Gemini CLI setup would need to be configured to interact with this running MCP server. This typically involves specifying the transport mechanism (e.g., stdio, TCP) and the address.
    *   You would then prompt Gemini, asking it to use the `copilot_suggest` tool with an appropriate prompt.

### Example Interaction Flow

1.  **Start `copilot_mcp_tool`:**
    ```bash
    cargo run
    ```
    The server will start and wait for input.

2.  **Using `mcp_client_caller` to test the server:**
    In a *separate terminal*, with `GITHUB_TOKEN` correctly set in its environment, run the client caller:
    ```bash
    cargo run --bin mcp_client_caller
    ```
    The `mcp_client_caller` will:
    *   Launch another instance of `copilot_mcp_tool` (this time acting as the client to *your running server* from step 1, using child process transport).
    *   Send a prompt to Copilot via the `copilot_suggest` tool (e.g., "What is the weather in London?").
    *   Copilot will respond with a JSON tool call (e.g., `{"tool": "get_weather", "arguments": {"location": "London"}}`).
    *   Your running `copilot_mcp_tool` server will parse this, call its internal `WeatherTool`, which in turn might call `TimeTool`, and so on.
    *   The final result will be returned to the `mcp_client_caller` and printed.

    **Note:** The example above uses `mcp_client_caller` to directly test the server. For a real Gemini CLI integration, the Gemini CLI would be the one issuing the `copilot_suggest` call.

### Demonstrating Nested Calls

*   **`copilot_suggest` -> `get_weather`:** If the prompt is "What is the weather in London?", Copilot will likely suggest `get_weather("London")`.
*   **`copilot_suggest` -> `get_weather` -> `get_time_in_location`:** If the prompt is "What is the weather and time in TimeCity?", Copilot might suggest `get_weather("TimeCity")`. The `WeatherTool` for "TimeCity" is configured to then call `get_time_in_location("TimeCity")`.
*   **`copilot_suggest` -> `get_weather` -> `get_time_in_location` -> `echo`:** The `TimeTool` for "EchoCity" is configured to then call `echo("Time for EchoCity")`.

## Contributing

Feel free to open issues or pull requests to improve this demonstration of nested MCP tool calling.
