//! Converge Knowledge MCP Server for Claude Desktop

use converge_knowledge::core::{KnowledgeBase, KnowledgeBaseConfig};
use converge_knowledge::mcp::McpServer;

use clap::Parser;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Converge Knowledge MCP Server for Claude Desktop
#[derive(Parser)]
#[command(name = "converge-knowledge-mcp")]
#[command(author, version, about = "MCP server for Claude Desktop integration")]
struct Args {
    /// Transport mode: stdio or http
    #[arg(short, long, default_value = "stdio")]
    transport: String,

    /// HTTP address (only used with http transport)
    #[arg(short, long, default_value = "127.0.0.1:8080")]
    address: String,

    /// Path to knowledge base storage
    #[arg(short, long, default_value = "./knowledge.db")]
    storage: String,

    /// Embedding dimensions
    #[arg(short, long, default_value = "384")]
    dimensions: usize,

    /// Disable self-learning
    #[arg(long)]
    no_learning: bool,

    /// Enable verbose logging (to stderr)
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Initialize logging (to stderr to not interfere with stdio transport)
    let filter = if args.verbose { "debug" } else { "warn" };
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| filter.into()),
        )
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
        .init();

    info!(
        "Starting Converge Knowledge MCP Server v{}",
        env!("CARGO_PKG_VERSION")
    );

    // Create knowledge base configuration
    let config = KnowledgeBaseConfig {
        dimensions: args.dimensions,
        storage_path: args.storage.clone(),
        learning_enabled: !args.no_learning,
        ..Default::default()
    };

    // Initialize knowledge base
    let kb = KnowledgeBase::with_config(config).await?;
    let kb = Arc::new(RwLock::new(kb));

    // Create MCP server
    let server = McpServer::new(kb);

    // Run with selected transport
    match args.transport.as_str() {
        "stdio" => {
            info!("Running MCP server with stdio transport");
            server.run_stdio().await?;
        }
        "http" => {
            info!("Running MCP server with HTTP transport on {}", args.address);
            server.run_http(&args.address).await?;
        }
        _ => {
            anyhow::bail!(
                "Unknown transport: {}. Use 'stdio' or 'http'.",
                args.transport
            );
        }
    }

    Ok(())
}
