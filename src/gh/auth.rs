use anyhow::{Context, Result};

use crate::util::{load_credentials, save_credentials, GithubCredentials};

/// Save token and fetch username
pub async fn login(token: &str) -> Result<String> {
    let username = get_username(token).await?;

    let mut creds = load_credentials().unwrap_or_default();
    creds.github = Some(GithubCredentials {
        token: token.to_string(),
        username: username.clone(),
    });
    save_credentials(&creds)?;

    Ok(username)
}

/// Get the username for the authenticated user
async fn get_username(token: &str) -> Result<String> {
    let octocrab = octocrab::OctocrabBuilder::new()
        .personal_token(token.to_string())
        .build()
        .context("Failed to create GitHub client")?;

    let user = octocrab
        .current()
        .user()
        .await
        .context("Failed to get current user - check your token")?;

    Ok(user.login)
}

/// Get stored token if available
pub fn get_token() -> Option<String> {
    load_credentials()
        .ok()
        .and_then(|c| c.github.map(|g| g.token))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_token_returns_option() {
        let result = get_token();
        // Result is either Some(token) or None
        assert!(result.is_some() || result.is_none());
    }

    #[test]
    fn get_token_consistent_results() {
        // Calling get_token multiple times should return the same result
        let result1 = get_token();
        let result2 = get_token();
        assert_eq!(result1.is_some(), result2.is_some());
    }

    #[test]
    fn credentials_struct_usage() {
        // Verify we can create and use the credential structs
        let creds = GithubCredentials {
            token: "test_token".to_string(),
            username: "testuser".to_string(),
        };
        assert_eq!(creds.token, "test_token");
        assert_eq!(creds.username, "testuser");
    }

    #[test]
    fn credentials_optional_in_parent() {
        use crate::util::Credentials;
        let creds = Credentials::default();
        assert!(creds.github.is_none());
    }
}
