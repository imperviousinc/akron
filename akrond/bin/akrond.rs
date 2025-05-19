use std::{env, fs};

use akrond::runner::{ServiceKind, ServiceRunner};
use akrond::Akron;
use directories::ProjectDirs;
use spaces_client::config::{safe_exit, ExtendedNetwork};
use tokio::sync::broadcast;

fn main() {
    let args: Vec<String> = env::args().collect();
    if let Some(service) = ServiceRunner::parse(&args) {
        if let Err(e) = service.run() {
            eprintln!("{}", e);
            safe_exit(1);
        }
        return;
    }

    let (akrond, shutdown) = Akron::create(false);
    let rt = tokio::runtime::Runtime::new().expect("Failed to build tokio runtime");
    if let Err(e) = rt.block_on(async_main(akrond, shutdown)) {
        eprintln!("{}", e);
        safe_exit(1);
    }
}

async fn async_main(akrond: Akron, shutdown: broadcast::Sender<()>) -> anyhow::Result<()> {
    let sigterm = tokio::signal::ctrl_c();
    let sigterm_shutdown = shutdown.clone();
    tokio::spawn(async move {
        sigterm.await.expect("could not listen for shutdown");
        let _ = sigterm_shutdown.send(());
    });

    let chain = ExtendedNetwork::Mainnet;
    let project_dirs = get_default_dirs();

    fs::create_dir_all(project_dirs.data_dir())?;

    println!(
        "Configuration directory: {}",
        project_dirs.data_dir().to_str().expect("No data dir")
    );
    let yuki_data_dir = project_dirs.data_dir().join("yuki");
    let spaces_data_dir = project_dirs.data_dir().join("spaces");

    let mut yuki_args = vec!["--data-dir", yuki_data_dir.to_str().expect("valid path")];

    let spaces_args = vec![
        "--bitcoin-rpc-url",
        "http://127.0.0.1:8225",
        "--data-dir",
        spaces_data_dir.to_str().expect("valid path"),
        "--bitcoin-rpc-light",
    ];

    let checkpoint_path = spaces_data_dir.join(chain.to_string());

    // Note: this loads the checkpoint and overrides the existing db
    // everytime.
    // TODO: check if the db already exists and store the initial checkpoint somewhere (to pass to yuki)
    let checkpoint = akrond
        .load_checkpoint(
            "https://checkpoint.akron.io/protocol.sdb",
            &checkpoint_path,
            None,
        )
        .await?;

    yuki_args.push("--prune-point");

    let prune_point = format!(
        "{}:{}",
        hex::encode(checkpoint.block.hash),
        checkpoint.block.height
    );
    yuki_args.push(&prune_point);

    akrond
        .start(
            ServiceKind::Yuki,
            yuki_args.iter().map(|s| s.to_string()).collect(),
        )
        .await?;
    akrond
        .start(
            ServiceKind::Spaces,
            spaces_args.iter().map(|s| s.to_string()).collect(),
        )
        .await?;

    println!("Checkpoint loaded at height = {}", checkpoint.block.height);

    let mut shutdown_recv = shutdown.subscribe();
    _ = shutdown_recv.recv().await;
    Ok(())
}

fn get_default_dirs() -> ProjectDirs {
    ProjectDirs::from("", "", "akron").unwrap_or_else(|| {
        eprintln!("error: could not retrieve default project directories from os");
        safe_exit(1);
    })
}
