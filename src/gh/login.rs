use anyhow::Result;

use super::auth;
use super::cli::LoginArgs;

/// Handle the `hu gh login` command
pub async fn run(args: LoginArgs) -> Result<()> {
    let username = auth::login(&args.token).await?;
    println!("✓ Logged in as {}", username);
    Ok(())
}
