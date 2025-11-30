use rmcp::model::{Request, Content, Tool as RmcpTool, CallToolResult, ListToolsResult, PaginatedRequestParam, ServerResult, ClientInfo, CallToolRequestMethod, Implementation, InitializeResult};
use rmcp::service::{Service, RoleServer, RequestContext, NotificationContext, RunningService};
use rmcp::model::{ClientRequest, ClientResult, ClientNotification, ServerInfo, ErrorData as McpError};
use rmcp::handler::server::tool::{ToolRouter, CallToolHandler, ToolCallContext};
use rmcp::handler::server::router::tool::{CallToolHandlerExt, IntoToolRoute};
use rmcp::transport::io;

use serde_json::Value;
use tokio::runtime::Runtime;
use anyhow::Result;
use async_trait::async_trait;
use log::{info, error};
use futures::{FutureExt, future::BoxFuture};

use std::sync::Arc;
use std::future::Future;

// Placeholder for RustDesk integration
mod rustdesk_integration;

// Define your MCP commands
pub struct ConnectToPeer;

// =========================================================================
// Custom ConnectToPeer Handler
// This struct will implement CallToolHandler
// =========================================================================
#[derive(Clone)]
struct ConnectToPeerCallHandler;

#[async_trait]
impl CallToolHandler<RustdeskMcpService, ()> for ConnectToPeerCallHandler {
    fn call(
        self,
        context: ToolCallContext<'_, RustdeskMcpService>,
    ) -> BoxFuture<'_, Result<CallToolResult, McpError>> {
        async move {
            info!("Executing connect_to_peer command with request: {:?}", context.arguments);

            let args = context.arguments.unwrap(); // Unwrap once

            let peer_id = args.get("peer_id").unwrap()
                .as_str()
                .ok_or_else(|| McpError::invalid_params("peer_id is required", None))?
                .to_string();
            let password = args.get("password").unwrap()
                .as_str()
                .map(|s| s.to_string());
            let conn_type = args.get("conn_type").unwrap()
                .as_str()
                .unwrap_or("Default")
                .to_string();

            info!("Attempting to connect to peer: {} with conn_type: {}", peer_id, conn_type);

            let session_id = uuid::Uuid::new_v4().to_string();
            info!("Successfully 'connected' to peer: {}, session_id: {}", peer_id, session_id);

            Ok(CallToolResult::success(vec![Content::text(session_id)]))
        }.boxed()
    }
}

// =========================================================================
// Refactored Service Implementation
// =========================================================================

struct RustdeskMcpService {
    tool_router: ToolRouter<Self>,
    // Other state for the service, if any
}

impl RustdeskMcpService {
    fn new() -> Self {
        let mut tool_router = ToolRouter::new();
        
        let connect_to_peer_attr = RmcpTool {
            name: "connect_to_peer".into(),
            title: None,
            description: Some("Initiates a connection to a specified RustDesk peer.".into()),
            input_schema: Arc::new(serde_json::json!({
                "type": "object",
                "properties": {
                    "peer_id": {
                        "type": "string",
                        "description": "The ID of the RustDesk peer to connect to."
                    },
                    "password": {
                        "type": "string",
                        "description": "The password for the remote peer (optional)."
                    },
                    "conn_type": {
                        "type": "string",
                        "description": "The connection type (e.g., 'Default', 'FileTransfer', 'Terminal')."
                    }
                },
                "required": ["peer_id"]
            }).as_object().unwrap().clone()), // Convert to Arc<JsonObject>
            output_schema: None,
            annotations: None,
            icons: None,
            meta: None,
        };

        let mut builder = ConnectToPeerCallHandler.name("connect_to_peer");
        builder.attr = connect_to_peer_attr;
        tool_router.add_route(builder.into_tool_route());

        Self {
            tool_router,
        }
    }
}

// Dummy service implementation for now, will refine based on `ConnectToPeer`
impl Service<RoleServer> for RustdeskMcpService {
    fn handle_request(
        &self,
        request: ClientRequest, // R::PeerReq
        context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ServerResult, McpError>> + Send + '_ {
        async move {
            match request {
                ClientRequest::CallToolRequest(req) => {
                    info!("Received CallToolRequest: {:?}", req);
                    // Use the tool_router to dispatch the call
                    let tool_call_context = ToolCallContext::new(
                        self,
                        req,
                        context
                    );
                    self.tool_router.call(tool_call_context).await
                        .map(ServerResult::CallToolResult)
                }
                ClientRequest::ListToolsRequest(_req) => {
                    info!("Received ListToolsRequest");
                    let tools = self.tool_router.list_all();
                    Ok(ServerResult::ListToolsResult(ListToolsResult { tools, next_cursor: None }))
                }
                _
                => {
                    error!("Unhandled ClientRequest: {:?}", request);
                    Err(McpError::method_not_found::<CallToolRequestMethod>())
                }
            }
        }
    }

    fn handle_notification(
        &self,
        notification: ClientNotification, // R::PeerNot
        context: NotificationContext<RoleServer>,
    ) -> impl Future<Output = Result<(), McpError>> + Send + '_ {
        async move {
            info!("Received ClientNotification: {:?}", notification);
            Ok(())
        }
    }

    fn get_info(&self) -> InitializeResult { // R::Info which is ServerInfo, which is InitializeResult
        InitializeResult {
            protocol_version: Default::default(),
            capabilities: Default::default(),
            server_info: Implementation { // Uses Implementation struct
                name: "Rustdesk MCP Service".to_string(),
                version: "0.1.0".to_string(),
                title: Some("mcpdesk Server".to_string()),
                icons: None,
                website_url: None,
            },
            instructions: None,
        }
    }
}

fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();
    info!("RustDesk MCP Service starting...");

    let runtime = Runtime::new()?;
    runtime.block_on(async {
        let service = RustdeskMcpService::new();

        // Start the MCP server using rmcp::service::serve_server
        // For standard IO (Unix pipes or Windows named pipes), use rmcp::transport::io::stdio()
        // The server_path mentioned before was likely for a specific transport implementation.
        // For now, we'll use stdio (stdin/stdout) as the transport.
        info!("Starting MCP server using standard I/O...");
        if let Err(e) = rmcp::service::serve_server(service, io::stdio()).await {
            error!("Failed to start MCP server: {:?}", e);
            return Err(e.into());
        }
        Ok(())
    })
}
