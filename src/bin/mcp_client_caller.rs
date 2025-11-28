use rmcp::service::{serve_client, ServiceExt};
use rmcp::transport::child_process::TokioChildProcess;
use rmcp::model::CallToolRequestParam;
use tokio::process::Command;
use serde_json::{json, Value};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok(); // Load .env file into the current process's environment

    // 1. Launch copilot_mcp_tool as a child process
    let mut child = Command::new("cargo");
    child.arg("run")
        .arg("--manifest-path")
        .arg("C:\\Users\\gentd\\OneDrive\\Documents\\GitHub\\copilot\\copilot_mcp_tool\\Cargo.toml") // Full path to the main Cargo.toml
        .arg("--bin") // Specify that we want to run a specific binary
        .arg("copilot_mcp_tool") // Specify the binary to run from the workspace
        .arg("--"); // Separate cargo args from child process args.




    // 2. Create a client to connect to copilot_mcp_tool
    let copilot_mcp_client = serve_client((), TokioChildProcess::new(child).unwrap())
        .await
        .unwrap();

    // 3. Call the copilot_suggest tool with a sample prompt
    let prompt = "What is the weather in London?"; // Sample prompt for Copilot
    let request_params = json!({
        "prompt": prompt
    });

    let request = CallToolRequestParam {
        name: "copilot_suggest".into(),
        arguments: Some(request_params.as_object().cloned().unwrap()),
    };

    println!("Calling copilot_suggest with prompt: \"{}\"", prompt);
    let result = copilot_mcp_client.peer().call_tool(request).await?;

    // 4. Print the result
    println!("Result from copilot_mcp_tool: {}", serde_json::to_string_pretty(&result)?);

    Ok(())
}