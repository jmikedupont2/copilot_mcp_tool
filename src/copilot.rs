use async_trait::async_trait;
pub use copilot_client::{
    Agent, ChatResponse, CopilotError, Embedding, Message, Model,
};
use rmcp::handler::client::ClientHandler;

#[async_trait]
pub trait Copilot {
    async fn get_agents(&self) -> Result<Vec<Agent>, CopilotError>;
    async fn get_models(&self) -> Result<Vec<Model>, CopilotError>;
    async fn chat_completion(
        &self,
        messages: Vec<Message>,
        model_id: String,
    ) -> Result<ChatResponse, CopilotError>;
    async fn get_embeddings(
        &self,
        inputs: Vec<String>,
    ) -> Result<Vec<Embedding>, CopilotError>;
}

use copilot_client::CopilotClient;

#[async_trait]
impl Copilot for CopilotClient {
    async fn get_agents(&self) -> Result<Vec<Agent>, CopilotError> {
        self.get_agents().await
    }

    async fn get_models(&self) -> Result<Vec<Model>, CopilotError> {
        self.get_models().await
    }

    async fn chat_completion(
        &self,
        messages: Vec<Message>,
        model_id: String,
    ) -> Result<ChatResponse, CopilotError> {
        self.chat_completion(messages, model_id).await
    }

    async fn get_embeddings(
        &self,
        inputs: Vec<String>,
    ) -> Result<Vec<Embedding>, CopilotError> {
        self.get_embeddings(inputs).await
    }
}

pub struct MyClientHandler;
impl ClientHandler for MyClientHandler {}