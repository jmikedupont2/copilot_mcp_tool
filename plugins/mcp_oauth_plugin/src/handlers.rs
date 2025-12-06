use axum::{
    extract::{Path, Query, State},
    response::{Html, IntoResponse, Redirect, Response},
};
use oauth2_reqwest::async_http_client; // Updated import
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, RedirectUrl, Scope,
    TokenResponse, TokenUrl,
};
use serde::Deserialize;
use std::sync::Arc;
use tracing::{error, info};
use url::Url;
use chrono::Utc;
use anyhow::Result; // Use anyhow::Result for error handling in the plugin handlers

use crate::oauth::{OAuthClient, OAuthConfig};
use crate::token_store::{OAuthToken, TokenStore};

// Placeholder for AppState relevant parts for the plugin
// This will be replaced by an actual struct passed from the main app
// Or the necessary parts passed as arguments
#[derive(Debug, Clone)]
pub struct PluginAppState {
    pub token_store: TokenStore,
    pub oauth_configs: Arc<tokio::sync::RwLock<std::collections::HashMap<String, OAuthConfig>>>,
    pub public_url: Url,
}


// Define state query parameter
#[derive(Debug, Deserialize)]
pub struct AuthState {
    pub state: String,
}

#[derive(Debug, Deserialize)]
pub struct AuthCode {
    code: String,
    state: String,
}

// OAuth start handler
pub async fn oauth_start(
    Path(provider): Path<String>,
    State(plugin_app_state): State<Arc<PluginAppState>>,
) -> Result<Redirect, anyhow::Error> { // Use anyhow::Error
    info!("OAuth start initiated for provider: {}", provider);

    let config = plugin_app_state
        .oauth_configs
        .read()
        .await
        .get(&provider)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("OAuth provider {} not found", provider))?; // Use anyhow

    let client = create_oauth_client(config, plugin_app_state.public_url.clone(), plugin_app_state.token_store.clone()).await?;

    let (authorize_url, csrf_state) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("openid".to_string()))
        .add_scope(Scope::new("email".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .add_extra_param("access_type", "offline")
        .add_extra_param("prompt", "consent")
        .add_extra_param("provider", &provider)
        .build() // Added .build()
        .url();

    plugin_app_state
        .token_store
        .save_csrf_token(provider, csrf_state.secret().to_string());

    Ok(Redirect::to(authorize_url.as_str()))
}

// OAuth callback handler
pub async fn oauth_callback(
    Query(AuthCode { code, state }): Query<AuthCode>,
    Query(AuthState { state: _provider_state }): Query<AuthState>, // Extract provider_state
    State(plugin_app_state): State<Arc<PluginAppState>>,
) -> Result<Html<String>, anyhow::Error> { // Use anyhow::Error
    info!("OAuth callback received");

    let provider = plugin_app_state
        .token_store
        .get_csrf_token_provider(&state)
        .ok_or_else(|| anyhow::anyhow!("Invalid or expired CSRF token".to_string()))?; // Use anyhow

    let config = plugin_app_state
        .oauth_configs
        .read()
        .await
        .get(&provider)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("OAuth provider {} not found", provider))?; // Use anyhow

    let csrf_token = plugin_app_state
        .token_store
        .retrieve_csrf_token(&state)
        .ok_or_else(|| anyhow::anyhow!("CSRF token not found or expired".to_string()))?; // Use anyhow

    if csrf_token != state {
        return Err(anyhow::anyhow!("CSRF token mismatch".to_string())); // Use anyhow
    }

    let client = create_oauth_client(config, plugin_app_state.public_url.clone(), plugin_app_state.token_store.clone()).await?;

    let token_result = client
        .exchange_code(AuthorizationCode::new(code))
        .request_async(async_http_client)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to exchange code for token: {}", e))?; // Use anyhow

    info!("Successfully authenticated, access token: {}", token_result.access_token().secret());

    let oauth_token = OAuthToken {
        provider_id: provider.clone(),
        access_token: token_result.access_token().secret().to_string(),
        refresh_token: token_result.refresh_token().map_or_else(|| "".to_string(), |t| t.secret().to_string()),
        expires_at: Utc::now() + chrono::Duration::seconds(token_result.expires_in().map_or(3600, |d| d.as_secs() as i64)),
        enterprise_url: None,
        project_id: None,
    };
    plugin_app_state.token_store.save(oauth_token)?;

    Ok(Html("<h1>Successfully logged in!</h1>".to_string()))
}

// Generic login page (if needed)
pub async fn oauth_login() -> Html<String> {
    Html("<h1>Login Page</h1><p>Please select an OAuth provider.</p>".to_string())
}

// Generic logout handler
pub async fn oauth_logout(State(plugin_app_state): State<Arc<PluginAppState>>) -> Result<Redirect, anyhow::Error> { // Use anyhow::Error
    plugin_app_state.token_store.remove_all_tokens()?;
    Ok(Redirect::to("/admin"))
}

// Helper to create OAuth client - now takes public_url and token_store
async fn create_oauth_client(config: OAuthConfig, public_url: Url, token_store: TokenStore) -> Result<BasicClient, anyhow::Error> {
    let client_id = ClientId::new(config.client_id);
    let client_secret = config.client_secret.map(ClientSecret::new);
    let auth_url = AuthUrl::new(config.auth_url)
        .map_err(|e| anyhow::anyhow!("Invalid AuthUrl: {}", e))?; // Use anyhow
    let token_url = TokenUrl::new(config.token_url)
        .map_err(|e| anyhow::anyhow!("Invalid TokenUrl: {}", e))?; // Use anyhow

    let redirect_url = public_url // Use the passed public_url
        .join("/oauth/callback")
        .map_err(|e| anyhow::anyhow!("Invalid redirect URL: {}", e))?; // Use anyhow
    let redirect_url = RedirectUrl::new(redirect_url.to_string())
        .map_err(|e| anyhow::anyhow!("Invalid RedirectUrl: {}", e))?; // Use anyhow

    let client = BasicClient::new(client_id) // Updated BasicClient::new
        .set_client_secret(client_secret)
        .set_auth_uri(auth_url)
        .set_token_url(Some(token_url)) // token_url is not optional in oauth2 v5
        .set_redirect_uri(redirect_url);

    Ok(client)
}