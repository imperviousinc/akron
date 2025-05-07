use tokio::sync::broadcast;

// Yuki service main
pub async fn main(args: Vec<String>, shutdown: broadcast::Sender<()>) -> anyhow::Result<()> {
    yuki::app::run(args, shutdown).await?;
    Ok(())
}
