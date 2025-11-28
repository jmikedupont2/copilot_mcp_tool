use rmcp::{tool_router, handler::server::tool::ToolRouter};
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

#[tokio::main]
async fn main() {
    let server = CopilotTool {
        tool_router: ToolRouter::new(),
    };
 //   rmcp::transport::run_server(server).await.unwrap();
    rmcp::transport::stdio::run_server(server).await.unwrap();
 //
}
