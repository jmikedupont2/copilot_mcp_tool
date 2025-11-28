use rmcp::handler::server::{ServerHandler, tool::ToolRouter};
use rmcp::service::ServiceExt;
use rmcp::tool_router;
use rmcp::model::CallToolRequestParam;

use serde::Deserialize;
use crate::copilot::Copilot;
use std::sync::Arc;
use copilot_client::CopilotClient;
use serde_json::Value;
use tokio::process::Command; // Changed to tokio::process::Command
use std::process::Stdio; // Imported Stdio directly from std::process
use std::io::{self, Write};

mod copilot;
mod tool_server_module;
use tool_server_module::{WeatherTool, WeatherInput};
mod level2_tool_module;
use level2_tool_module::{TimeTool, TimeInput};
mod level3_tool_module;
use level3_tool_module::{EchoTool, EchoInput};

// Helper function to get token from gh CLI (copied from mcp_client_caller)
async fn get_github_token_from_gh_cli() -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("gh") // Now tokio::process::Command
        .arg("auth")
        .arg("token")
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()?
        .wait_with_output()
        .await?; // Await here is now correct

    if !output.status.success() {
        eprintln!("Error: 'gh auth token' failed with status: {}", output.status);
        io::stdout().write_all(&output.stderr)?;
        return Err("Failed to get GitHub token from gh CLI".into());
    }

    let token = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if token.is_empty() {
        return Err("No token found in 'gh auth token' output.".into());
    }
    Ok(token)
}


#[derive(Clone)]
pub struct CopilotTool {
    tool_router: ToolRouter<Self>,
    copilot_client: Arc<dyn Copilot + Send + Sync>,
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
                content: "You are a helpful assistant that suggests tool calls in JSON format.".to_string(),
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

                    if tool_name == "get_weather" {
                        if let Some(location) = arguments_map.get("location").and_then(|v| v.as_str()) {
                            let weather_input = WeatherInput {
                                location: location.to_string(),
                            };
                            let result = self.weather_tool.get_weather(weather_input).await;
                            return serde_json::to_string(&result).unwrap();
                        }
                    }
                    if tool_name == "get_time_in_location" {
                         if let Some(location) = arguments_map.get("location").and_then(|v| v.as_str()) {
                            let time_input = TimeInput {
                                location: location.to_string(),
                            };
                            let result = self.weather_tool.time_tool.get_time_in_location(time_input).await;
                            return serde_json::to_string(&result).unwrap();
                        }
                    }
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
async fn main() -> Result<(), Box<dyn std::error::Error>> { // Make main return Result
    dotenv::dotenv().ok(); // Load .env file

    let github_token = get_github_token_from_gh_cli().await?; // Get token here

    let copilot_client = CopilotClient::new_with_models( // Use new_with_models
        github_token,
        "copilot_mcp_tool".to_string()
    ).await?; // Await and ?

    let echo_tool = Arc::new(level3_tool_module::new_echo_tool());
    let time_tool = Arc::new(level2_tool_module::new_time_tool(echo_tool.clone()));
    let weather_tool = tool_server_module::new_weather_tool(time_tool.clone());

    let server = CopilotTool {
        tool_router: ToolRouter::new(),
        copilot_client: Arc::new(copilot_client),
        weather_tool: Arc::new(weather_tool),
    };
    server.serve(rmcp::transport::stdio()).await.unwrap();

    Ok(()) // Add Ok(()) at the end
}
