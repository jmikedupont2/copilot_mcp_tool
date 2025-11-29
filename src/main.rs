use rmcp::{
    handler::server::{ServerHandler, tool::ToolRouter},
    model::{CallToolResult, ErrorData}, // Removed unused Tool import
    service::{ServiceExt, RequestContext, RoleServer}, // Corrected imports for RequestContext and RoleServer
    // tool, // Removed to avoid confusion
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream}; // Corrected import for TcpListener and TcpStream
use std::env;
use schemars::JsonSchema;
use async_trait::async_trait;

#[derive(Clone)]
pub struct EchoServerTool {
    tool_router: ToolRouter<Self>,
}

#[derive(Deserialize, Serialize, JsonSchema)]
pub struct EchoInput {
    message: String,
}

impl EchoServerTool {
    pub fn new() -> Self {
        let mut tool_router = ToolRouter::new();
        // Commenting out tool! macro for now
        /*
        tool_router.with_route(
            tool!(
                name = "echo_message",
                description = "Echoes a message back",
                input = EchoInput,
                output = String,
                callback = EchoServerTool::echo_message,
            )
        );
        */
        EchoServerTool { tool_router }
    }

    async fn echo_message(&self, input: EchoInput) -> String {
        eprintln!("EchoServerTool: Received message: {}", input.message); // Corrected from input.input.message
        format!("Echoing: {}", input.message)
    }
}

#[async_trait]
impl ServerHandler for EchoServerTool {
    async fn call_tool(
        &self,
        request_param: rmcp::model::CallToolRequestParam,
        _request_context: RequestContext<RoleServer>, // Re-added third parameter
    ) -> Result<CallToolResult, ErrorData> {
        let tool_name = request_param.name.as_ref();
        let arguments = request_param.arguments.unwrap_or_default();
        
        // ToolRouter::call_tool is assumed async based on previous error hinting.
        match self.tool_router.call_tool(tool_name, arguments).await {
            Ok(result) => Ok(result),
            Err(e) => Err(ErrorData { code: rmcp::model::ErrorCode::ServerError, message: format!("ToolRouter error: {}", e).into(), data: None }), // Corrected ErrorCode variant
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("Main: copilot_mcp_tool started.");
    let args: Vec<String> = env::args().collect();
    let port_string_from_env: String;
    let port_str = if args.len() > 1 {
        &args[1]
    } else {
        port_string_from_env = env::var("PORT").unwrap_or_else(|_| "0".to_string());
        port_string_from_env.as_str()
    };
    let port: u16 = port_str.parse()?;
    let addr = format!("127.0.0.1:{}", port);
    eprintln!("Main: Attempting to bind to address: {}", addr);

    let server_tool = EchoServerTool::new();
    let server_arc = Arc::new(server_tool);
    eprintln!("Main: EchoServerTool instance created.");

    let listener = TcpListener::bind(&addr).await?;
    let local_addr = listener.local_addr()?;
    eprintln!("SERVER_PORT: {}", local_addr.port());
    eprintln!("EchoServerTool listening on {}", local_addr);

    loop {
        eprintln!("Main: Waiting for incoming connection...");
        let (stream, peer_addr) = listener.accept().await?;
        eprintln!("Main: Accepted connection from: {}", peer_addr);

        let server_clone = server_arc.clone();
        tokio::spawn(async move {
            let server_for_conn = (*server_clone).clone();
            eprintln!("Handler: Serving client {}: Starting rmcp::service::serve", peer_addr);
            
            match server_for_conn.serve(stream).await {
                Ok(running_service) => {
                    eprintln!("Handler: Serving client {}: Server loop started.", peer_addr);
                    if let Err(e) = running_service.waiting().await {
                        eprintln!("Handler: Error waiting for client {}: {}", peer_addr, e);
                    }
                },
                Err(e) => {
                    eprintln!("Handler: Error serving client {}: {}", peer_addr, e);
                }
            }
            eprintln!("Handler: Client {} disconnected. Finished serving.", peer_addr);
        });
    }
}