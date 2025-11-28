use rmcp::{handler::server::{ServerHandler, tool::ToolRouter}, service::ServiceExt, tool_router};
use serde::Deserialize;

#[derive(Clone)]
pub struct CopilotTool {
    tool_router: ToolRouter<Self>,
}

#[derive(Deserialize)]
pub struct SuggestionInput {
    code: String,
}

#[tool_router]
impl CopilotTool {
    async fn copilot_suggest(&self, input: SuggestionInput) -> String {
        format!("Copilot suggests improving this code:\n{}", input.code)
    }
}

impl ServerHandler for CopilotTool {}

#[tokio::main]
async fn main() {
    let server = CopilotTool {
        tool_router: ToolRouter::new(),
    };
    server.serve(rmcp::transport::stdio()).await.unwrap();
}