use async_trait::async_trait;
// pub use copilot_client::{
//     Agent, ChatResponse, CopilotError, Embedding, Message, Model,
// }; // Will redefine/map types
use rmcp::handler::client::ClientHandler;
use std::fmt::Display;
use async_openai::config::OpenAIConfig;

// --- Common/Abstracted Types for LLM Interaction ---
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatChoice {
    pub message: Message,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatResponse {
    pub choices: Vec<ChatChoice>,
}

// --- Error Handling ---
#[derive(Debug)]
pub enum CopilotError {
    OpenAIError(async_openai::error::OpenAIError),
    GitHubCopilotError(Box<dyn std::error::Error + Send + Sync>), // For errors from copilot_client crate
    Other(String),
}

impl Display for CopilotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CopilotError::OpenAIError(e) => write!(f, "OpenAI Error: {}", e),
            CopilotError::GitHubCopilotError(e) => write!(f, "GitHub Copilot Error: {}", e),
            CopilotError::Other(msg) => write!(f, "Other Error: {}", msg),
        }
    }
}

impl std::error::Error for CopilotError {}


// --- The Copilot Trait (LLM Abstraction) ---
#[async_trait]
pub trait Copilot: Send + Sync {
    async fn chat_completion(
        &self,
        messages: Vec<Message>,
        model_id: String,
    ) -> Result<ChatResponse, CopilotError>;
}


// --- OpenAI Client Implementation ---
use async_openai::{
    Client as OpenAIClient,
    types::{
        ChatCompletionRequestMessage,
        ChatCompletionRequestUserMessageArgs,
        ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestAssistantMessageArgs,
        CreateChatCompletionRequestArgs,
    },
};

pub struct OpenAICopliotClient {
    client: OpenAIClient<OpenAIConfig>,
}

impl OpenAICopliotClient {
    pub fn new() -> Self {
        Self {
            client: OpenAIClient::new(),
        }
    }
}

#[async_trait]
impl Copilot for OpenAICopliotClient {
    async fn chat_completion(
        &self,
        messages: Vec<Message>,
        model_id: String,
    ) -> Result<ChatResponse, CopilotError> {
        let openai_messages: Vec<ChatCompletionRequestMessage> = messages
            .into_iter()
            .map(|msg| {
                match msg.role.as_str() {
                    "user" => ChatCompletionRequestUserMessageArgs::default()
                                .content(msg.content)
                                .build().unwrap().into(),
                    "system" => ChatCompletionRequestSystemMessageArgs::default()
                                .content(msg.content)
                                .build().unwrap().into(),
                    "assistant" => ChatCompletionRequestAssistantMessageArgs::default()
                                .content(msg.content)
                                .build().unwrap().into(),
                    _ => ChatCompletionRequestUserMessageArgs::default() // Default to user message
                                .content(msg.content)
                                .build().unwrap().into(),
                }
            })
            .collect();

        let request = CreateChatCompletionRequestArgs::default()
            .model(model_id)
            .messages(openai_messages)
            .build()
            .map_err(|e| CopilotError::Other(e.to_string()))?;

        let response = self.client.chat().create(request).await.map_err(CopilotError::OpenAIError)?;

        let choices: Vec<ChatChoice> = response.choices.into_iter().map(|choice| {
            ChatChoice {
                message: Message {
                    role: choice.message.role.to_string(),
                    content: choice.message.content.unwrap_or_default(),
                },
                finish_reason: choice.finish_reason.map(|r| format!("{:?}", r)), // Fixed here
            }
        }).collect();

        Ok(ChatResponse { choices })
    }
}


// --- GitHub Copilot Client Implementation ---
use copilot_client::CopilotClient as GitHubCopilotClientRaw; // Use a different name to avoid conflict

pub struct GitHubCopilotClient {
    client: GitHubCopilotClientRaw,
}

impl GitHubCopilotClient {
    pub async fn new(editor_version: String) -> Result<Self, CopilotError> {
        // This is where the problematic token retrieval happens.
        // For now, we will initialize with a dummy token to allow compilation,
        // but this part will need active debugging.
        let github_token = "dummy_token".to_string(); // Placeholder
        let client = GitHubCopilotClientRaw::new_with_models(
            github_token,
            editor_version,
        ).await.map_err(|e| CopilotError::GitHubCopilotError(e.into()))?;

        Ok(Self { client })
    }
}

#[async_trait]
impl Copilot for GitHubCopilotClient {
    async fn chat_completion(
        &self,
        messages: Vec<Message>,
        model_id: String,
    ) -> Result<ChatResponse, CopilotError> {
        // Map our generic messages to copilot_client's Message type if necessary.
        let gh_messages: Vec<copilot_client::Message> = messages.into_iter().map(|msg| {
            copilot_client::Message {
                role: msg.role,
                content: msg.content,
            }
        }).collect();

        let response = self.client.chat_completion(gh_messages, model_id).await
            .map_err(|e| CopilotError::GitHubCopilotError(e.into()))?;

        // Map copilot_client's ChatResponse to our generic ChatResponse
        let choices: Vec<ChatChoice> = response.choices.into_iter().map(|choice| {
            ChatChoice {
                message: Message {
                    role: choice.message.role,
                    content: choice.message.content,
                },
                finish_reason: choice.finish_reason,
            }
        }).collect();

        Ok(ChatResponse { choices })
    }
}


// --- LLM Driver Enum ---
pub enum LlmDriver {
    OpenAI(OpenAICopliotClient),
    GitHub(GitHubCopilotClient),
}

#[async_trait]
impl Copilot for LlmDriver {
    async fn chat_completion(
        &self,
        messages: Vec<Message>,
        model_id: String,
    ) -> Result<ChatResponse, CopilotError> {
        match self {
            LlmDriver::OpenAI(client) => client.chat_completion(messages, model_id).await,
            LlmDriver::GitHub(client) => client.chat_completion(messages, model_id).await,
        }
    }
}


pub struct MyClientHandler;
impl ClientHandler for MyClientHandler {}
