mod world;
mod entity;
mod actions;
mod descriptions;
mod persistence;
mod mcp;

use std::path::PathBuf;
use anyhow::Result;
use tracing_subscriber::EnvFilter;

fn main() -> Result<()> {
    // Initialize logging to stderr (so it doesn't interfere with MCP protocol on stdout)
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .with_writer(std::io::stderr)
        .with_target(false)
        .init();

    tracing::info!("Rubber Duck MCP Server v0.1.0");
    tracing::info!("A text-based healing nature simulation");

    // Determine state file path
    let state_path = get_state_path();
    tracing::info!("State file: {:?}", state_path);

    // Ensure data directory exists
    if let Some(parent) = state_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Create and run the MCP server
    let mut server = mcp::McpServer::new(state_path);
    server.run()?;

    Ok(())
}

fn get_state_path() -> PathBuf {
    // Check for RUBBER_DUCK_STATE environment variable
    if let Ok(path) = std::env::var("RUBBER_DUCK_STATE") {
        return PathBuf::from(path);
    }

    // Default to current directory
    let mut path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    path.push("data");
    path.push("world_state.json");
    path
}
