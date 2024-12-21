use std::time::Duration;
use std::{process::Command, thread};

use serde::Serialize;

use crate::{
    config::Config,
    protocol::{Block, Header, Status},
};

pub struct StatusGenerator {
    interval: Duration,
    header_sent: bool,
    config: Config,
    pretty: bool,
}

impl StatusGenerator {
    pub fn new(config: Config) -> Self {
        StatusGenerator {
            interval: Duration::from_secs(1),
            header_sent: false,
            config,
            pretty: false,
        }
    }

    fn to_json<T: Serialize>(&self, value: T) -> String {
        match self.pretty {
            true => serde_json::to_string_pretty(&value).unwrap(),
            false => serde_json::to_string(&value).unwrap(),
        }
    }
}

impl Iterator for StatusGenerator {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.header_sent {
            self.header_sent = true;
            return Some(format!("{}\n[", self.to_json(Header::new())));
        }

        thread::sleep(self.interval);

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

            blocks.push(Block::new(&stdout, &stdout, "command", &section.name));
        }

        Some(self.to_json(&Status { blocks }))
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use crate::config::{Config, Section};

    use super::StatusGenerator;

    #[rstest]
    fn should_have_new_line_after_header() {
        let mut status_generator = StatusGenerator::new(Config { sections: vec![] });

        let header = status_generator.next().unwrap();
        assert_eq!(
            header,
            r#"{"version":1,"click_events":false,"cont_signal":18,"stop_signal":19}
["#
        );
    }

    #[rstest]
    fn should_generate_status() {
        let mut status_generator = StatusGenerator::new(Config {
            sections: vec![
                Section {
                    name: "first section".to_string(),
                    command: "uname -r".to_string(),
                },
                Section {
                    name: "second section".to_string(),
                    command: r#"date -d "2024-12-21 21:44:54" "+%Y-%m-%d %H:%M:%S""#.to_string(),
                },
            ],
        });
        status_generator.pretty = true;

        let header = status_generator.next().unwrap();
        assert_eq!(
            header,
            r#"{
  "version": 1,
  "click_events": false,
  "cont_signal": 18,
  "stop_signal": 19
}
["#
        );

        let status1: String = status_generator.next().unwrap();
        assert_eq!(
            status1,
            r##"[
  {
    "full_text": "6.12.5-200.fc41.x86_64",
    "short_text": "6.12.5-200.fc41.x86_64",
    "color": "#000000",
    "background": "#ffffff",
    "border": "#000000",
    "border_top": 1,
    "border_bottom": 1,
    "border_left": 1,
    "border_right": 1,
    "min_width": "6.12.5-200.fc41.x86_64",
    "align": "left",
    "name": "command",
    "instance": "first section",
    "urgent": false,
    "separator": true,
    "separator_block_width": 9,
    "markup": "none"
  },
  {
    "full_text": "2024-12-21 21:44:54",
    "short_text": "2024-12-21 21:44:54",
    "color": "#000000",
    "background": "#ffffff",
    "border": "#000000",
    "border_top": 1,
    "border_bottom": 1,
    "border_left": 1,
    "border_right": 1,
    "min_width": "2024-12-21 21:44:54",
    "align": "left",
    "name": "command",
    "instance": "second section",
    "urgent": false,
    "separator": true,
    "separator_block_width": 9,
    "markup": "none"
  }
]"##,
        );
    }
}
