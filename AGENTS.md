# Agent Guidelines for hevy-progressive-overloader

## Commands

- **Build**: `cargo build`
- **Run server**: `cargo run` (uses PORT env var, defaults to 3005)
- **Run on different port**: `PORT=8080 cargo run`
- **Test all**: `cargo test`
- **Test single**: `cargo test <test_name>`
- **Lint**: `cargo clippy`
- **Format**: `cargo fmt`
- **Check**: `cargo check` (fast compilation check)

## Code Style

- **Imports**: Group std imports first, then external crates, then local modules
- **Naming**: snake_case for functions/variables, PascalCase for types/structs, SCREAMING_SNAKE_CASE for constants
- **Error handling**: Use `anyhow::Result<T>` with `?` operator; avoid unwrap() except in tests
- **Types**: Prefer explicit types over inference for public APIs
- **Formatting**: Run `cargo fmt` before commits
- **Documentation**: Add doc comments for public functions/structs using `///`
- **Async**: Use `async fn` for async operations, prefer `tokio::spawn` for background tasks

## Project Structure

- Source code in `src/` with modular structure (clients/, config.rs, main.rs)
- Tests in same files as implementation using `#[cfg(test)]` modules
- Webhook-based service for Hevy fitness app integration
- Environment variables: HEVY_API_KEY, WEBHOOK_TOKEN, PORT, BASE_URL

# Always

- Format the file after adding/editing code
- Use `anyhow::Result<T>` for error handling consistency

