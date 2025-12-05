use std::fmt::Debug; // Removed Display
use std::error::Error;
// Removed Deserialize, Serialize - as Meme struct is now in core.
use std::any::Any;
use serde_json; // Added serde_json

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

// Trait for an abstract meme. Concrete Meme structs (e.g., in solfunmeme-core) will implement this.
pub trait AnyMeme: Debug + Send + Sync + 'static {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn category_name(&self) -> String; // Changed to String
    fn category_emoji(&self) -> String; // Changed to String
    fn emoji(&self) -> String; // Changed to String
    fn content(&self) -> String; // Changed to String
    fn tags(&self) -> &[String]; // Remains &[String]
    // Add other methods for common meme properties that the UI needs to display.
    // This allows the loader to deal with memes polymorphically without knowing their concrete type.
    fn as_any(&self) -> &dyn Any;
    fn box_clone(&self) -> Box<dyn AnyMeme>; // Added for Clone
    fn equals(&self, other: &dyn AnyMeme) -> bool; // Added for PartialEq
}


// Trait for a source that can provide Meme implementations.
// This could be a static list, a loaded module, or a remote service.
pub trait MemeSource: Send + Sync {
    fn get_all_memes(&self) -> Result<Vec<Box<dyn AnyMeme>>>;
    fn get_memes_by_category(&self, category: &str) -> Result<Vec<Box<dyn AnyMeme>>>;
    // Add other methods for querying memes.
}

// Trait for handling encryption/decryption of meme-related state.
// This could be used for meme content, internal configuration, etc.
pub trait EncryptedState: Send + Sync {
    fn encrypt(&self, data: &[u8], key: &[u8]) -> Result<Vec<u8>>;
    fn decrypt(&self, data: &[u8], key: &[u8]) -> Result<Vec<u8>>;
}

// Trait for a loader that discovers and loads MemeSource implementations.
pub trait MemeLoader: Send + Sync {
    fn load_source(&self, source_id: &str) -> Result<Box<dyn MemeSource>>;
    // Add methods for dynamic loading (e.g., from .so files, network).
}

// Custom error types for the loader.
#[derive(Debug, thiserror::Error)]
pub enum MemeLoaderError {
    #[error("Meme not found: {0}")]
    MemeNotFound(String),
    #[error("Category not found: {0}")]
    CategoryNotFound(String),
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),
    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error("Other error: {0}")]
    Other(String),
}

impl From<String> for MemeLoaderError {
    fn from(s: String) -> Self {
        MemeLoaderError::Other(s)
    }
}