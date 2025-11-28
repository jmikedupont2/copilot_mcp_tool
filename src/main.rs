use rmcp::handler::server::{ServerHandler, tool::ToolRouter};
use rmcp::service::ServiceExt;
use rmcp::tool_router;
use rmcp::model::CallToolRequestParam; // Re-add CallToolRequestParam here

use serde::Deserialize;
use crate::copilot::Copilot;
use std::sync::Arc;
use copilot_client::CopilotClient;
use serde_json::Value;
// use std::env; // Removed std::env as it's not directly used for non-debugging purposes now.

mod copilot;
mod tool_server_module; // Import the new module
use tool_server_module::{WeatherTool, WeatherInput}; // Import WeatherTool and WeatherInput
mod level2_tool_module; // Import the level2 tool module
use level2_tool_module::{TimeTool, TimeInput}; // Import TimeTool and TimeInput
mod level3_tool_module; // Import the level3 tool module
use level3_tool_module::{EchoTool, EchoInput}; // Import EchoTool and EchoInput

#[derive(Clone)]
pub struct CopilotTool {
    tool_router: ToolRouter<Self>,
    copilot_client: Arc<dyn Copilot + Send + Sync>,
    // Changed this to hold WeatherTool directly
    weather_tool: Arc<WeatherTool>,
}

#[derive(Deserialize)]
pub struct SuggestionInput {
    prompt: String,
}

#[tool_router]
impl CopilotTool {
    async fn copilot_suggest(&self, input: SuggestionInput) -> String {
        let messages = vec![
            crate::copilot::Message {
                role: "system".to_string(),
                content: "You are a helpful assistant that suggests tool calls in JSON format.".to_string(), // Simplified content
            },
            crate::copilot::Message {
                role: "user".to_string(),
                content: input.prompt,
            },
        ];

        let response = self
            .copilot_client
            .chat_completion(messages, "gpt-4".to_string())
            .await
            .unwrap();

        if let Some(choice) = response.choices.first() {
            let content = &choice.message.content;
            if let Ok(tool_call) = serde_json::from_str::<Value>(content) {
                if let (Some(tool), Some(arguments)) = (tool_call.get("tool"), tool_call.get("arguments")) {
                    let tool_name = tool.as_str().unwrap().to_string();
                    let arguments_map = arguments.as_object().cloned().unwrap_or_default();

                    // Directly call the weather_tool
                    if tool_name == "get_weather" {
                        if let Some(location) = arguments_map.get("location").and_then(|v| v.as_str()) {
                            let weather_input = WeatherInput {
                                location: location.to_string(),
                            };
                            let result = self.weather_tool.get_weather(weather_input).await;
                            return serde_json::to_string(&result).unwrap();
                        }
                    }
                    // Handle calls to the TimeTool if Copilot suggests it directly
                    // This is for demonstration of nested calls. The prompt for Copilot
                    // should be carefully crafted to chain calls if needed.
                    if tool_name == "get_time_in_location" {
                         if let Some(location) = arguments_map.get("location").and_then(|v| v.as_str()) {
                            let time_input = TimeInput {
                                location: location.to_string(),
                            };
                            let result = self.weather_tool.time_tool.get_time_in_location(time_input).await;
                            return serde_json::to_string(&result).unwrap();
                        }
                    }
                    // Handle calls to the EchoTool if Copilot suggests it directly
                    if tool_name == "echo" {
                        if let Some(message) = arguments_map.get("message").and_then(|v| v.as_str()) {
                            let echo_input = EchoInput {
                                message: message.to_string(),
                            };
                            let result = self.weather_tool.time_tool.echo_tool.echo(echo_input).await;
                            return serde_json::to_string(&result).unwrap();
                        }
                    }
                }
            }
            content.clone()
        } else {
            "No suggestion found".to_string()
        }
    }
}

impl ServerHandler for CopilotTool {}


#[tokio::main]
async fn main() {
    dotenv::dotenv().ok(); // Load .env file

    // Removed the println! statement for GITHUB_TOKEN

    let copilot_client = CopilotClient::from_env_with_models("copilot_mcp_tool".to_string())
        .await
        .unwrap();

    let echo_tool = Arc::new(level3_tool_module::new_echo_tool()); // Instantiate EchoTool
    let time_tool = Arc::new(level2_tool_module::new_time_tool(echo_tool.clone())); // Pass EchoTool to TimeTool
    let weather_tool = tool_server_module::new_weather_tool(time_tool.clone()); // Pass TimeTool to WeatherTool

    let server = CopilotTool {
        tool_router: ToolRouter::new(),
        copilot_client: Arc::new(copilot_client),
        weather_tool: Arc::new(weather_tool), // Pass the weather_tool
    };
    server.serve(rmcp::transport::stdio()).await.unwrap();
}