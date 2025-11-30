use axum::{
    routing::{get, post},
    Router,
    response::{Html, IntoResponse},
    Form,
};
use std::net::SocketAddr;
use tracing_subscriber;
use tracing;
use tera::{Tera, Context};
use serde::{Deserialize, Serialize};
use lazy_static::lazy_static; // Required for lazy_static macro
use tokio::io::{AsyncWriteExt, AsyncBufReadExt, BufReader};
use serde_json::{json, Value};
use std::env;
use std::fs;
use std::path::PathBuf;

// --- Lock File Management ---

#[derive(Serialize, Deserialize, Debug)]
struct LockData {
    pid: u32,
    port: u16,
}

fn get_lock_file_path() -> Result<PathBuf, std::io::Error> {
    let mut temp_dir = env::temp_dir();
    temp_dir.push("copilot_mcp_tool.lock");
    Ok(temp_dir)
}

fn read_lock_file() -> Result<LockData, anyhow::Error> {
    let path = get_lock_file_path()?;
    let content = fs::read_to_string(path)?;
    let data: LockData = serde_json::from_str(&content)?;
    Ok(data)
}

// --- End Lock File Management ---

// Address of the MCP server. This should ideally be configurable (e.g., via environment variable).
// For now, hardcode it to a common local address.
// const MCP_SERVER_ADDR: &str = "127.0.0.1:21230"; // Using port from previous Python run

lazy_static! {
    pub static ref TERA: Tera = {
        let mut tera = match Tera::new("templates/**/*.html") {
            Ok(t) => t,
            Err(e) => {
                println!("Parsing error(s): {}", e);
                ::std::process::exit(1);
            }
        };
        tera.autoescape_on(vec![ ".html"]);
        tera
    };
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt().init();

    tracing::info!("Starting MCP Web Client.");

    // build our application with routes
    let app = Router::new()
        .route("/", get(index_handler))
        .route("/process", post(process_handler));

    // run it with hyper on localhost:3000
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn index_handler() -> impl IntoResponse {
    let context = Context::new();
    let rendered = TERA.render("index.html", &context).unwrap();
    Html(rendered)
}

#[derive(Debug, Deserialize)]
struct ProcessForm {
    content: String,
}

async fn process_handler(Form(form): Form<ProcessForm>) -> impl IntoResponse {
    tracing::info!("Received content: {}", form.content);

    let mcp_response_str = match send_mcp_request("echo_message", json!({"message": form.content})).await {
        Ok(response) => {
            tracing::info!("MCP Server Response: {:?}", response);
            // Assuming the MCP server's echo_message returns a simple string in its 'result' field
            // The result field itself contains a JSON string, so we need to parse it again.
            // MCP often returns results as JSON strings that need an extra deserialization step.
            if let Some(result_json_str) = response["result"].as_str() {
                match serde_json::from_str::<String>(result_json_str) {
                    Ok(s) => s,
                    Err(_) => result_json_str.to_string(), // Fallback if it's not a simple string
                }
            } else {
                // If result is not a string, just convert the whole result object to a string
                response["result"].to_string()
            }
        },
        Err(e) => {
            tracing::error!("Error communicating with MCP server: {:?}", e);
            format!("Error communicating with MCP server: {}", e)
        }
    };

    let mut context = Context::new();
    context.insert("output", &mcp_response_str);
    let rendered = TERA.render("index.html", &context).unwrap();
    Html(rendered)
}

// Function to send JSON-RPC requests to the MCP server
async fn send_mcp_request(method: &str, params: Value) -> Result<Value, anyhow::Error> {
    let lock_data = read_lock_file().map_err(|e| anyhow::anyhow!("Failed to read MCP server lock file. Is the server running? Error: {}", e))?;
    let mcp_server_addr = format!("127.0.0.1:{}", lock_data.port);

    tracing::info!("Connecting to MCP server at {}", mcp_server_addr);
    let stream = tokio::net::TcpStream::connect(&mcp_server_addr).await?;
    let (reader, mut writer) = stream.into_split();
    let mut buf_reader = BufReader::new(reader);
    
    tracing::info!("Connected to MCP server.");

    // 1. Send initialize request
    let init_params = json!({
        "protocolVersion": "2025-03-26",
        "capabilities": { "roots": { "listChanged": true } },
        "clientInfo": { "name": "mcp_web_client", "version": "0.1.0" }
    });
    let init_request = json!({
        "jsonrpc": "2.0",
        "id": "init",
        "method": "initialize",
        "params": init_params,
    });
    let init_request_str = serde_json::to_string(&init_request)?;
    writer.write_all(init_request_str.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    
    let mut line = String::new();
    buf_reader.read_line(&mut line).await?; // Read init response
    tracing::info!("Received init response: {}", line.trim());

    // 2. Send initialized notification
    let initialized_notif = json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized"
    });
    let initialized_notif_str = serde_json::to_string(&initialized_notif)?;
    writer.write_all(initialized_notif_str.as_bytes()).await?;
    writer.write_all(b"\n").await?;

    // 3. Send the actual tool request
    let request_id = 1; // Simple request ID for the main call
    let request = json!({
        "jsonrpc": "2.0",
        "id": request_id,
        "method": method,
        "params": params,
    });

    let request_str = serde_json::to_string(&request)?;
    tracing::info!("Sending MCP request: {}", request_str);
    writer.write_all(request_str.as_bytes()).await?;
    writer.write_all(b"\n").await?;

    line.clear();
    buf_reader.read_line(&mut line).await?;
    tracing::info!("Received raw MCP response: {}", line.trim());

    let response: Value = serde_json::from_str(&line)?;
    
    if let Some(error) = response.get("error") {
        return Err(anyhow::anyhow!("MCP Server Error: {:?}", error));
    }

    Ok(response)
}