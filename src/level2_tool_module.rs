use rmcp::{handler::server::{ServerHandler, tool::ToolRouter}, tool_router};
use serde::Deserialize;
use std::sync::Arc;
use crate::level3_tool_module::{EchoTool, EchoInput}; // Import EchoTool and EchoInput

#[derive(Clone)]
pub struct TimeTool {
    tool_router: ToolRouter<Self>,
    pub echo_tool: Arc<EchoTool>, // Add EchoTool as a client
}

#[derive(Deserialize)]
pub struct TimeInput {
    pub location: String,
}

#[tool_router]
impl TimeTool {
    pub async fn get_time_in_location(&self, input: TimeInput) -> String {
        // In a real application, you would integrate with a time API
        // For demonstration, let's say if the location is "EchoCity", it calls the echo tool.
        if input.location == "EchoCity" {
            let echo_input = EchoInput {
                message: format!("Time for {}", input.location),
            };
            let echo_result = self.echo_tool.echo(echo_input).await;
            format!("The current time in {} is 12:00 PM. {}", input.location, echo_result)
        } else {
            format!("The current time in {} is 12:00 PM.", input.location)
        }
    }
}

impl ServerHandler for TimeTool {}

// Modify new_time_tool to accept EchoTool
pub fn new_time_tool(echo_tool: Arc<EchoTool>) -> TimeTool {
    TimeTool {
        tool_router: ToolRouter::new(),
        echo_tool,
    }
}