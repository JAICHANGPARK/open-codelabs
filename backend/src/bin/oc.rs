use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    backend::cli::app::entry().await
}
