use rmcp::service::{serve_client, ServiceExt};
use rmcp::transport::child_process::TokioChildProcess;
use rmcp::model::CallToolRequestParam;
use tokio::process::Command;
use serde_json::{json, Value};
// use std::env; // Remove std::env import as it's no longer needed

// Removed get_github_token_from_gh_cli function

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok(); // Load .env file into the current process's environment

    // No need to get GitHub Token from gh CLI here or pass it to child
    // as copilot_mcp_tool will get its own token.

    // 1. Launch copilot_mcp_tool (now an echo server) as a child process
    let mut child = Command::new("target/debug/copilot_mcp_tool.exe");

    // 2. Create a client to connect to copilot_mcp_tool
    let copilot_mcp_client = serve_client((), TokioChildProcess::new(child).unwrap())
        .await
        .unwrap();

    // 3. Call the echo_message tool with a sample prompt
    let message = "Hello from client!";
    let request_params = json!({
        "message": message
    });

    let request = CallToolRequestParam {
        name: "echo_message".into(), // Changed to echo_message
        arguments: Some(request_params.as_object().cloned().unwrap()),
    };

    println!("Calling echo_message with message: \"{}\"", message);
    let result = copilot_mcp_client.peer().call_tool(request).await?;

    // 4. Print the result
    println!("Result from copilot_mcp_tool: {}", serde_json::to_string_pretty(&result)?);

    Ok(())
}