use axum::{
    extract::{Query, State},
    response::Html,
    routing::get,
    Router,
};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;

use anyhow::{Context, Result};

/// OAuth callback state
#[derive(Debug, Clone)]
pub(super) struct CallbackState {
    pub(super) expected_state: String,
    pub(super) code: Option<String>,
    pub(super) error: Option<String>,
}

/// Start the local callback server
pub(super) async fn start_callback_server(state: Arc<Mutex<CallbackState>>) -> Result<()> {
    let app = Router::new()
        .route("/callback", get(handle_callback))
        .with_state(state.clone());

    let addr = SocketAddr::from(([127, 0, 0, 1], super::CALLBACK_PORT));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .context("Failed to bind callback server")?;

    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            // Wait until we have a code or error
            loop {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                let state_lock = state.lock().await;
                if state_lock.code.is_some() || state_lock.error.is_some() {
                    break;
                }
            }
            // Give a moment for the response to be sent
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        })
        .await
        .context("Callback server failed")?;

    Ok(())
}

/// Handle the OAuth callback
async fn handle_callback(
    State(state): State<Arc<Mutex<CallbackState>>>,
    Query(params): Query<HashMap<String, String>>,
) -> Html<&'static str> {
    let mut state_lock = state.lock().await;

    // Check for error
    if let Some(error) = params.get("error") {
        state_lock.error = Some(error.clone());
        return Html(
            "<html><body><h1>Authorization Failed</h1><p>You can close this window.</p></body></html>",
        );
    }

    // Verify state parameter
    if let Some(received_state) = params.get("state") {
        if received_state != &state_lock.expected_state {
            state_lock.error = Some("State mismatch - possible CSRF attack".to_string());
            return Html(
                "<html><body><h1>Error</h1><p>State verification failed.</p></body></html>",
            );
        }
    } else {
        state_lock.error = Some("Missing state parameter".to_string());
        return Html("<html><body><h1>Error</h1><p>Missing state parameter.</p></body></html>");
    }

    // Get authorization code
    if let Some(code) = params.get("code") {
        state_lock.code = Some(code.clone());
        Html("<html><body><h1>Success!</h1><p>You can close this window and return to the terminal.</p></body></html>")
    } else {
        state_lock.error = Some("Missing authorization code".to_string());
        Html("<html><body><h1>Error</h1><p>Missing authorization code.</p></body></html>")
    }
}
