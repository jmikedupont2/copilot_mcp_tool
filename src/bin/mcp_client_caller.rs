use rmcp::service::serve_client;
use rmcp::model::CallToolRequestParam;
use tokio::process::Command;
use serde_json::{json, Value};
use tokio::net::TcpStream;
use tokio::io::{BufReader, AsyncBufReadExt};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    // 1. Launch copilot_mcp_tool as a child process, capturing stderr
    let mut child_cmd = Command::new("target/debug/copilot_mcp_tool.exe");
    child_cmd.stderr(std::process::Stdio::piped());
    let mut child = child_cmd.spawn()?; // child is mut to allow kill() later

    let child_stderr = child.stderr.take().ok_or("Child process stderr not captured")?;
    let mut reader = BufReader::new(child_stderr);

    let mut port_line = String::new();
    let port = loop {
        port_line.clear();
        reader.read_line(&mut port_line).await?;
        if port_line.starts_with("SERVER_PORT: ") {
            let port_str = port_line.trim_start_matches("SERVER_PORT: ").trim();
            break port_str.parse::<u16>()?;
        }
        if port_line.is_empty() {
            return Err("Child process exited without printing server port".into());
        }
    };
    eprintln!("Client: Captured server port: {}", port);

    // Read and print the rest of the child's stderr in a background task
    tokio::spawn(async move {
        let mut line = String::new();
        while reader.read_line(&mut line).await.unwrap_or(0) > 0 {
            eprintln!("Server stderr: {}", line.trim_end());
            line.clear();
        }
    });

    let addr = format!("127.0.0.1:{}", port);
    eprintln!("Client: Attempting to connect to server at {}", addr);

    // 2. Connect to the child process via TCP
    let stream = TcpStream::connect(&addr).await?;
    eprintln!("Client: Connected to server at {}.", addr);

    // 3. Create an rmcp client to communicate over TCP
    let copilot_mcp_client = serve_client((), stream)
        .await
        .unwrap(); // This is the unwrap that previously failed.

    eprintln!("Client: rmcp client created. Calling echo_message...");
    // 4. Call the echo_message tool with a sample message
    let message = "Hello from client!";
    let request_params = json!({
        "message": message
    });

    let request = CallToolRequestParam {
        name: "echo_message".into(),
        arguments: Some(request_params.as_object().cloned().unwrap()),
    };

    println!("Calling echo_message with message: \"{}\"", message);
    let result = copilot_mcp_client.peer().call_tool(request).await?;

    // 5. Print the result
    println!("Result from copilot_mcp_tool: {}", serde_json::to_string_pretty(&result)?);

    // Clean up child process
    child.kill().await?;

    Ok(())
}