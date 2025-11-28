use rmcp::{handler::server::{ServerHandler, tool::ToolRouter}, service::ServiceExt, tool_router};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::io;

// Define a simple Echo tool
#[derive(Clone)]
pub struct EchoServerTool {
    tool_router: ToolRouter<Self>,
}

#[derive(Deserialize, Serialize)]
pub struct EchoInput {
    message: String,
}

#[tool_router]
impl EchoServerTool {
    async fn echo_message(&self, input: EchoInput) -> String {
        format!("Echoing: {}", input.message)
    }
}

impl ServerHandler for EchoServerTool {}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server = EchoServerTool {
        tool_router: ToolRouter::new(),
    };

    server.serve((io::stdin(), io::stdout())).await?;

    Ok(())
}