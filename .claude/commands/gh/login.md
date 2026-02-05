Authenticate with GitHub using a Personal Access Token.

Usage: `hu gh login -t <TOKEN>`

Options:
- `-t, --token` - Personal Access Token (required)

Setup:
1. Create a token at https://github.com/settings/tokens
2. Required scopes: `repo`, `read:org`, `workflow`
3. Run `hu gh login -t ghp_...`

Token is stored in `~/.config/hu/credentials.toml`.
