use std::net::{Shutdown, TcpStream};
use std::io::{BufRead, BufReader, Write};
use anyhow::{Result, anyhow};

use rmcp::model::{
    InitializeRequestParam,
    Implementation,
    ClientCapabilities,
    RootsCapabilities,
    ProtocolVersion,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

// region:    --- Types for JSON-RPC
#[derive(Debug, Serialize, Deserialize)]
pub struct RpcRequest<'a> {
    jsonrpc: &'a str,
    id: u64,
    method: &'a str,
    params: Value,
}

#[derive(Debug, Serialize)]
pub struct RpcNotification<'a> {
    jsonrpc: &'a str,
    method: &'a str,
    params: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RpcResponse {
    #[serde(flatten)]
    pub result: RpcResult,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RpcResult {
    Success { result: Value },
    Error { error: RpcError },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RpcError {
    pub code: i64,
    pub message: String,
    pub data: Option<Value>,
}
// endregion: --- Types for JSON-RPC

// region:    --- MCP Client
pub struct McpClient {
    stream: Option<TcpStream>,
}

impl McpClient {
    pub fn new() -> Self {
        McpClient { stream: None }
    }

    pub fn connect(&mut self, port: u16) -> Result<()> {
        let stream = TcpStream::connect(format!("localhost:{}", port))?;
        println!("Client connected to localhost:{}", port);
        self.stream = Some(stream);
        Ok(())
    }

    fn send_request(&mut self, method: &str, params: Value) -> Result<()> {
        let stream = self.stream.as_mut().ok_or_else(|| anyhow!("Not connected"))?;
        let request = RpcRequest {
            jsonrpc: "2.0",
            id: 1, // Using a fixed ID for simplicity in this example client
            method,
            params,
        };
        let mut json_req = serde_json::to_string(&request)?;
        json_req.push('\n');
        stream.write_all(json_req.as_bytes())?;
        Ok(())
    }

    pub fn receive_response(&mut self) -> Result<RpcResponse> {
        let stream = self.stream.as_mut().ok_or_else(|| anyhow!("Not connected"))?;
        let mut reader = BufReader::new(stream);
        let mut response_str = String::new();
        reader.read_line(&mut response_str)?;
        let response: RpcResponse = serde_json::from_str(&response_str)?;
        Ok(response)
    }

    pub fn initialize(&mut self) -> Result<RpcResponse> {
        let params = InitializeRequestParam {
            protocol_version: ProtocolVersion::LATEST,
            capabilities: ClientCapabilities {
                roots: Some(RootsCapabilities { list_changed: Some(true) }),
                ..Default::default()
            },
            client_info: Implementation {
                name: "copilot_mcp_tool_client".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                ..Default::default()
            },
        };
        self.send_request("initialize", serde_json::to_value(params)?)
            ?;
        self.receive_response()
    }

    pub fn initialized_notification(&mut self) -> Result<()> {
        let stream = self.stream.as_mut().ok_or_else(|| anyhow!("Not connected"))?;
        let notification = RpcNotification {
            jsonrpc: "2.0",
            method: "notifications/initialized",
            params: None,
        };
        let mut json_req = serde_json::to_string(&notification)?;
        json_req.push('\n');
        stream.write_all(json_req.as_bytes())?;
        Ok(())
    }

    pub fn list_tools(&mut self) -> Result<RpcResponse> {
        self.send_request("tools/list", serde_json::json!({}))?;
        self.receive_response()
    }

    pub fn call_tool(&mut self, tool_name: &str, tool_params: Value) -> Result<RpcResponse> {
        let params = serde_json::json!({
            "name": tool_name,
            "arguments": tool_params,
        });
        self.send_request("tools/call", params)?;
        self.receive_response()
    }
}

impl Drop for McpClient {
    fn drop(&mut self) {
        if let Some(stream) = self.stream.as_ref() {
            let _ = stream.shutdown(Shutdown::Both);
        }
    }
}
// endregion: --- MCP Client
