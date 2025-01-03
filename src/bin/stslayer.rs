use std::{path::PathBuf, process, str::FromStr};

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
    /// Path to Status Slayer config file.
    /// Default: `$XDG_CONFIG_HOME/stslayer/config.toml`
    config: Option<PathBuf>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let config_path = &args.config.to_owned().unwrap_or_else(defaul_config_path);
    let config = Config::from_file(&config_path).unwrap_or_else(|err| {
        eprintln!("Error: {}", err);
        process::exit(1);
    });

    let (tx, mut rx) = mpsc::channel(1);

    let st_task = tokio::spawn(async move {
        let mut status_controller = StatusController::new(config, tx);
        status_controller.run().await.unwrap();
    });

    let echo_task = tokio::spawn(async move {
        while let Some(status) = rx.recv().await {
            println!("{}", status);
        }
    });

    st_task.await.unwrap();
    echo_task.await.unwrap();
}

fn defaul_config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or(PathBuf::from_str(".config").unwrap())
        .join("stslayer/config.toml")
}
