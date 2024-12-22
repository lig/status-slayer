use std::{fs, path::PathBuf};

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

fn main() {
    let args = Args::parse();

    let config_path = args.config;
    if !config_path.is_file() {
        panic!("Config file doesn't exist or is not a regular file")
    }

    let config: Config =
        toml::from_str(&fs::read_to_string(config_path).expect("Cannot read config file"))
            .expect("Config file format error");

    for status in StatusGenerator::new(config) {
        println!("{}", status);
    }
}
