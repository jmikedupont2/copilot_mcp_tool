use rmcp::{handler::server::{ServerHandler, tool::ToolRouter}, tool_router};
use serde::Deserialize;

#[derive(Clone)]
pub struct EchoTool {
    tool_router: ToolRouter<Self>,
}

#[derive(Deserialize)]
pub struct EchoInput {
    pub message: String,
}

#[tool_router]
impl EchoTool {
    pub async fn echo(&self, input: EchoInput) -> String {
        format!("Echo: {}", input.message)
    }
}

impl ServerHandler for EchoTool {}

pub fn new_echo_tool() -> EchoTool {
    EchoTool {
        tool_router: ToolRouter::new(),
    }
}
