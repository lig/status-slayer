use std::path::PathBuf;

use clap::Parser;
use stslayer::{config::Config, controller::StatusController};
use tokio::sync::mpsc;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
/** Status Slayer is a configurable implementation of status command for Sway WM using
Swaybar Protocol
*/
struct Args {
    #[arg(short, long)]
    config: PathBuf,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let config = Config::from_file(&args.config).unwrap();

    let (tx, mut rx) = mpsc::channel(1);

    let echo_task = tokio::spawn(async move {
        while let Some(status) = rx.recv().await {
            println!("{}", status);
        }
    });

    let st_task = tokio::spawn(async move {
        let mut status_controller = StatusController::new(config, tx);
        status_controller.run().await.unwrap();
    });

    echo_task.await.unwrap();
    st_task.await.unwrap();
}
