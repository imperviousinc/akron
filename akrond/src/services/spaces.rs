use env_logger::Env;
use spaces_client::app::App;
use tokio::sync::broadcast;

// Spaces service main
pub async fn main(args: Vec<String>, shutdown: broadcast::Sender<()>) -> anyhow::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let mut app = App::new(shutdown.clone());
    app.run(args).await
}
