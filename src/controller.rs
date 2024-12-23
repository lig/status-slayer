use std::time::Duration;
use std::{process::Command, thread};

use anyhow::Result;
use serde::Serialize;
use tokio::sync::mpsc::Sender;

use crate::{
    config::Config,
    protocol::{Block, Header, Status},
};

pub struct StatusController {
    interval: Duration,
    config: Config,
    sender: Sender<String>,
    pretty: bool,
}

impl StatusController {
    pub fn new(config: Config, sender: Sender<String>) -> Self {
        StatusController {
            interval: Duration::from_secs(1),
            config,
            sender,
            pretty: false,
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        self.sender.send(self.get_header()).await.unwrap();

        loop {
            let next_line = self.get_status();
            self.sender.send(next_line).await.unwrap();
            thread::sleep(self.interval);
        }
    }

    fn get_header(&self) -> String {
        return format!("{}\n[", self.to_json(Header::new()));
    }


    fn get_status(&self) -> String {
        let mut blocks: Vec<Block> = vec![];

        for section in &self.config.sections {
            let output = Command::new("sh")
                .args(["-c", &section.command])
                .output()
                .unwrap_or_else(|_| panic!("Failed to execute command `{}`", &section.command));

            if !output.status.success() {
                panic!(
                    "Command `{}` failed with error:\n{}",
                    &section.command,
                    String::from_utf8_lossy(&output.stderr)
                );
            }

            let stdout = String::from_utf8_lossy(output.stdout.trim_ascii_end());

            blocks.push(Block::new(&stdout, "command", &section.name));
        }

        format!("{},", self.to_json(&Status { blocks }))
    }

    fn to_json<T: Serialize>(&self, value: T) -> String {
        match self.pretty {
            true => serde_json::to_string_pretty(&value).unwrap(),
            false => serde_json::to_string(&value).unwrap(),
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;
    use tokio::sync::mpsc;

    use crate::config::{Config, Section};

    use super::StatusController;

    #[rstest]
    fn should_generate_header() {
        let (tx, _rx) = mpsc::channel(1);

        let status_controller = StatusController::new(Config { sections: vec![] }, tx);

        let header = status_controller.get_header();
        assert_eq!(
            header,
            r#"{"version":1,"click_events":false,"cont_signal":18,"stop_signal":19}
["#
        );
    }

    #[rstest]
    fn should_generate_status() {
        let (tx, _rx) = mpsc::channel(1);

        let mut status_controller = StatusController::new(
            Config {
                sections: vec![
                    Section {
                        name: "first section".to_string(),
                        command: "uname -s".to_string(),
                    },
                    Section {
                        name: "second section".to_string(),
                        command: r#"date -d "2024-12-21 21:44:54" "+%Y-%m-%d %H:%M:%S""#
                            .to_string(),
                    },
                ],
            },
            tx,
        );
        status_controller.pretty = true;

        let status1: String = status_controller.get_status();
        assert_eq!(
            status1,
            r##"[
  {
    "full_text": "Linux",
    "min_width": "Linux",
    "name": "command",
    "instance": "first section",
    "urgent": false,
    "separator": true,
    "markup": "none"
  },
  {
    "full_text": "2024-12-21 21:44:54",
    "min_width": "2024-12-21 21:44:54",
    "name": "command",
    "instance": "second section",
    "urgent": false,
    "separator": true,
    "markup": "none"
  }
],"##,
        );
    }
}
