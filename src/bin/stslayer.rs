use std::path::PathBuf;

use clap::Parser;
use stslayer::{config::Config, generator::StatusGenerator};

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

    let config_path = args.config;

    let config = Config::from_file(&config_path).unwrap();

    for status in StatusGenerator::new(config) {
        println!("{}", status);
    }
}
