mod client;
mod server;

use client::SearchBackend;
use rmcp::{ServiceExt, transport::stdio};
use server::SearchServer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_env_filter(tracing_subscriber::EnvFilter::from_default_env()).init();
    let backend = SearchBackend::from_env()?;
    let service = SearchServer { backend }.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
