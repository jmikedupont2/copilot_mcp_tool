use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc; // Kept std::sync::Arc
use tokio::sync::RwLock; // Changed from std::sync::RwLock
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};

/// OAuth token information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthToken {
    /// Provider ID (e.g., "claude-max", "anthropic-oauth")
    pub provider_id: String,
    /// OAuth access token
    pub access_token: String,
    /// OAuth refresh token
    pub refresh_token: String,
    /// Token expiration time (UTC)
    pub expires_at: DateTime<Utc>,
    /// Optional enterprise URL for GitHub Copilot Enterprise
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enterprise_url: Option<String>,
    /// Optional Google Cloud project ID for Gemini Code Assist API
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
}

impl OAuthToken {
    /// Check if token is expired
    pub fn is_expired(&self) -> bool {
        Utc::now() >= self.expires_at
    }

    /// Check if token will expire soon (within 5 minutes)
    pub fn needs_refresh(&self) -> bool {
        let now = Utc::now();
        let buffer = chrono::Duration::minutes(5);
        now + buffer >= self.expires_at
    }
}

/// Token storage - persists to JSON file
#[derive(Debug, Clone)]
pub struct TokenStore {
    /// Path to token storage file
    file_path: PathBuf,
    /// In-memory cache of tokens
    tokens: Arc<RwLock<HashMap<String, OAuthToken>>>,
    /// In-memory cache of CSRF states (PKCE verifier) to provider_id mappings
    csrf_states: Arc<RwLock<HashMap<String, String>>>,
}

impl TokenStore {
    /// Create a new token store
    /// Loads existing tokens from file if it exists
    pub fn new(file_path: PathBuf) -> Result<Self> {
        let tokens = if file_path.exists() {
            let content = fs::read_to_string(&file_path)
                .context("Failed to read token file")?;
            serde_json::from_str(&content)
                .context("Failed to parse token file")?
        } else {
            HashMap::new()
        };

        Ok(Self {
            file_path,
            tokens: Arc::new(RwLock::new(tokens)),
            csrf_states: Arc::new(RwLock::new(HashMap::new())), // Initialize csrf_states
        })
    }

    /// Get default token store path
    /// ~/.claude-code-mux/oauth_tokens.json
    pub fn default_path() -> Result<PathBuf> {
        let home = dirs::home_dir()
            .context("Failed to get home directory")?;
        let config_dir = home.join(".claude-code-mux");
        fs::create_dir_all(&config_dir)
            .context("Failed to create config directory")?;
        Ok(config_dir.join("oauth_tokens.json"))
    }

    /// Create a token store at the default location
    pub fn default() -> Result<Self> {
        let path = Self::default_path()?;
        Self::new(path)
    }

    /// Save token for a provider
    pub fn save(&self, token: OAuthToken) -> Result<()> {
        let provider_id = token.provider_id.clone();

        // Update in-memory cache
        tokio::runtime::Handle::current().block_on(async { // block_on for async RwLock
            let mut tokens = self.tokens.write().await;
            tokens.insert(provider_id, token);
        });

        // Persist to file
        self.persist()?;

        Ok(())
    }

    /// Get token for a provider
    pub fn get(&self, provider_id: &str) -> Option<OAuthToken> {
        tokio::runtime::Handle::current().block_on(async { // block_on for async RwLock
            let tokens = self.tokens.read().await;
            tokens.get(provider_id).cloned()
        })
    }

    /// Remove token for a provider
    pub fn remove(&self, provider_id: &str) -> Result<()> {
        tokio::runtime::Handle::current().block_on(async { // block_on for async RwLock
            let mut tokens = self.tokens.write().await;
            tokens.remove(provider_id);
            Ok(()) as anyhow::Result<()> // Explicitly cast to anyhow::Result<()>
        })?;

        // Persist to file
        self.persist()?;

        Ok(())
    }

    /// Remove all tokens
    pub fn remove_all_tokens(&self) -> Result<()> {
        tokio::runtime::Handle::current().block_on(async { // block_on for async RwLock
            let mut tokens = self.tokens.write().await;
            tokens.clear();
            Ok(()) as anyhow::Result<()> // Explicitly cast to anyhow::Result<()>
        })?;

        // Persist to file
        self.persist()?;

        Ok(())
    }

    /// List all provider IDs that have tokens
    pub fn list_providers(&self) -> Vec<String> {
        tokio::runtime::Handle::current().block_on(async { // block_on for async RwLock
            let tokens = self.tokens.read().await;
            tokens.keys().cloned().collect()
        })
    }

    /// Get all tokens
    pub fn all(&self) -> HashMap<String, OAuthToken> {
        tokio::runtime::Handle::current().block_on(async { // block_on for async RwLock
            let tokens = self.tokens.read().await;
            tokens.clone()
        })
    }

    /// Persist tokens to file
    fn persist(&self) -> Result<()> {
        tokio::runtime::Handle::current().block_on(async { // block_on for async RwLock
            let tokens = self.tokens.read().await;
            let json = serde_json::to_string_pretty(&*tokens)
                .context("Failed to serialize tokens")?;

            fs::write(&self.file_path, json)
                .context("Failed to write token file")?;

            // Set file permissions to 0600 (owner read/write only)
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = fs::metadata(&self.file_path)?.permissions();
                perms.set_mode(0o600);
                fs::set_permissions(&self.file_path, perms)?;
            }
            Ok(())
        })
    }

    /// Save a CSRF token (PKCE state) associated with a provider_id
    pub fn save_csrf_token(&self, provider_id: String, state: String) {
        tokio::runtime::Handle::current().block_on(async { // block_on for async RwLock
            let mut csrf_states = self.csrf_states.write().await;
            csrf_states.insert(state, provider_id);
        })
    }

    /// Get the provider_id associated with a CSRF token (PKCE state)
    pub fn get_csrf_token_provider(&self, state: &str) -> Option<String> {
        tokio::runtime::Handle::current().block_on(async { // block_on for async RwLock
            let csrf_states = self.csrf_states.read().await;
            csrf_states.get(state).cloned()
        })
    }

    /// Retrieve and remove a CSRF token (PKCE state)
    pub fn retrieve_csrf_token(&self, state: &str) -> Option<String> {
        tokio::runtime::Handle::current().block_on(async { // block_on for async RwLock
            let mut csrf_states = self.csrf_states.write().await;
            csrf_states.remove(state)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_token_store() {
        let rt = tokio::runtime::Runtime::new().unwrap(); // Create a Tokio runtime
        rt.block_on(async { // Block on the async test
            let temp_dir = TempDir::new().unwrap();
            let token_path = temp_dir.path().join("tokens.json");
            let store = TokenStore::new(token_path).unwrap();

            let token = OAuthToken {
                provider_id: "test-provider".to_string(),
                access_token: "access-123".to_string(),
                refresh_token: "refresh-456".to_string(),
                expires_at: Utc::now() + chrono::Duration::hours(1),
                enterprise_url: None,
                project_id: None,
            };

            store.save(token.clone()).unwrap();

            let retrieved = store.get("test-provider").unwrap();
            assert_eq!(retrieved.access_token, "access-123");
            assert_eq!(retrieved.refresh_token, "refresh-456");

            store.remove("test-provider").unwrap();
            assert!(store.get("test-provider").is_none());
        });
    }

    #[test]
    fn test_token_expiration() {
        let expired_token = OAuthToken {
            provider_id: "test".to_string(),
            access_token: "token".to_string(),
            refresh_token: "refresh".to_string(),
            expires_at: Utc::now() - chrono::Duration::hours(1),
            enterprise_url: None,
            project_id: None,
        };

        assert!(expired_token.is_expired());
        assert!(expired_token.needs_refresh());

        let valid_token = OAuthToken {
            provider_id: "test".to_string(),
            access_token: "token".to_string(),
            refresh_token: "refresh".to_string(),
            expires_at: Utc::now() + chrono::Duration::hours(1),
            enterprise_url: None,
            project_id: None,
        };

        assert!(!valid_token.is_expired());
        assert!(!valid_token.needs_refresh());
    }
}