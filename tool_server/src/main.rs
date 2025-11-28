use rmcp::{handler::server::{ServerHandler, tool::ToolRouter}, service::ServiceExt, tool_router};
use serde::Deserialize;

#[derive(Clone)]
pub struct WeatherTool {
    tool_router: ToolRouter<Self>,
}

#[derive(Deserialize)]
pub struct WeatherInput {
    location: String,
}

#[tool_router]
impl WeatherTool {
    async fn get_weather(&self, input: WeatherInput) -> String {
        format!("The weather in {} is sunny.", input.location)
    }
}

impl ServerHandler for WeatherTool {}

#[tokio::main]
async fn main() {
    let server = WeatherTool {
        tool_router: ToolRouter::new(),
    };
    server.serve(rmcp::transport::stdio()).await.unwrap();
}