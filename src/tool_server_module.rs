use rmcp::{handler::server::{ServerHandler, tool::ToolRouter}, tool_router};
use serde::Deserialize;
use std::sync::Arc; // Import Arc
use crate::level2_tool_module::{TimeTool, TimeInput}; // Import TimeTool and TimeInput

#[derive(Clone)]
pub struct WeatherTool {
    tool_router: ToolRouter<Self>,
    pub time_tool: Arc<TimeTool>, // Add pub
}

#[derive(Deserialize)]
pub struct WeatherInput {
    pub location: String,
}

#[tool_router]
impl WeatherTool {
    pub async fn get_weather(&self, input: WeatherInput) -> String {
        // Here, the weather tool can decide to call the time tool based on some logic
        // For demonstration, let's say if the location is "TimeCity", it calls the time tool.
        if input.location == "TimeCity" {
            let time_input = TimeInput {
                location: input.location.clone(),
            };
            let time_result = self.time_tool.get_time_in_location(time_input).await;
            format!("Weather in TimeCity is sunny, and {}", time_result)
        } else {
            format!("The weather in {} is sunny.", input.location)
        }
    }
}

impl ServerHandler for WeatherTool {}

// Modify new_weather_tool to accept TimeTool
pub fn new_weather_tool(time_tool: Arc<TimeTool>) -> WeatherTool {
    WeatherTool {
        tool_router: ToolRouter::new(),
        time_tool,
    }
}
