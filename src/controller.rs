use std::process::Command;
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use anyhow::Result;
use itertools::Itertools;
use serde::Serialize;
use tokio::{
    sync::mpsc::{self, Sender},
    time::sleep,
};

use crate::{
    config::{Config, Interval, Section},
    protocol::{Block, Header, Status},
};

pub struct StatusController {
    config: Config,
    status_sender: Sender<String>,
    pretty: bool,
    min_interval: Duration,
    section_index: HashMap<SectionId, usize>,
    status: Status,
}

impl StatusController {
    pub fn new(config: Config, sender: Sender<String>) -> Self {
        assert!(
            !config.sections.is_empty(),
            "At least one section must be defined in config"
        );
        let section_index: HashMap<SectionId, usize> = config
            .sections
            .iter()
            .enumerate()
            .map(|(n, section)| (SectionId::new("command", &section.name), n))
            .collect();
        let num_sections = section_index.len();
        StatusController {
            config,
            status_sender: sender,
            pretty: false,
            min_interval: Duration::from_secs(1),
            section_index,
            status: Status {
                blocks: Vec::with_capacity(num_sections),
            },
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        self.status_sender.send(self.get_header()).await.unwrap();

        let (block_sender, mut block_receiver) = mpsc::channel::<Block>(1);
        for section in self.config.sections.clone() {
            let block_sender = block_sender.clone();
            tokio::spawn(async {
                let mut section_controller = SectionController::new(section, block_sender);
                section_controller.run().await;
            });
        }

        let mut initial_data: HashMap<SectionId, Block> = HashMap::new();
        while initial_data.len() < self.section_index.len() {
            let block = tokio::select! {
                block = block_receiver.recv() => block.unwrap(),
            };
            initial_data
                .entry(SectionId::new(&block.name, &block.instance))
                .insert_entry(block);
        }
        self.status.blocks.extend(
            initial_data
                .into_iter()
                .map(|(section_id, block)| (self.section_index[&section_id], block))
                .sorted_by_key(|v| v.0)
                .map(|(_, block)| block),
        );

        self.status_sender.send(self.get_status()).await.unwrap();
        let mut last_sent = Instant::now();

        loop {
            let block = tokio::select! {
                block = block_receiver.recv() => block.unwrap(),
            };
            let section_num = self.section_index[&SectionId::new(&block.name, &block.instance)];
            self.status.blocks[section_num] = block;

            if last_sent.elapsed() > self.min_interval {
                self.status_sender.send(self.get_status()).await.unwrap();
                last_sent = Instant::now();
            }
        }
    }

    fn get_header(&self) -> String {
        format!("{}\n[", self.to_json(self.get_header_data()))
    }

    fn get_status(&self) -> String {
        format!("{},", self.to_json(&self.status))
    }

    fn get_header_data(&self) -> Header {
        Header::new()
    }

    fn to_json<T: Serialize>(&self, value: T) -> String {
        match self.pretty {
            true => serde_json::to_string_pretty(&value).unwrap(),
            false => serde_json::to_string(&value).unwrap(),
        }
    }
}

#[derive(PartialEq, Eq, Hash)]
struct SectionId {
    module: String,
    name: String,
}

impl SectionId {
    fn new(module: &str, name: &str) -> Self {
        Self {
            module: module.to_string(),
            name: name.to_string(),
        }
    }
}

struct SectionController {
    config: Section,
    sender: Sender<Block>,
    cache: Option<String>,
}

impl SectionController {
    fn new(config: Section, sender: Sender<Block>) -> Self {
        Self {
            config,
            sender,
            cache: None,
        }
    }

    async fn run(&mut self) {
        loop {
            let tick = Instant::now();
            let output = Command::new("sh")
                .args(["-c", &self.config.command])
                .output()
                .unwrap_or_else(|_| panic!("Failed to execute command `{}`", &self.config.command));

            if !output.status.success() {
                panic!(
                    "Command `{}` failed with error:\n{}",
                    &self.config.command,
                    String::from_utf8_lossy(&output.stderr)
                );
            }

            let stdout = String::from_utf8_lossy(output.stdout.trim_ascii_end());

            if self.cache.as_ref().is_none_or(|v| v != &stdout) {
                let stdout = self.cache.insert(stdout.to_string());
                self.sender
                    .send(Block::new("command", &self.config.name, stdout))
                    .await
                    .unwrap();
            }

            match self.config.interval {
                Interval::Oneshot => break,
                Interval::Seconds(duration) => sleep(duration - tick.elapsed()).await,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;
    use tokio::sync::mpsc;

    use crate::config::{Config, Interval, Section};

    use super::StatusController;

    #[rstest]
    fn should_produce_header() {
        let (tx, _rx) = mpsc::channel(1);

        let status_controller = StatusController::new(
            Config {
                min_interval: Config::default_min_interval(),
                sections: vec![Section {
                    name: "foo".to_string(),
                    command: "test".to_string(),
                    interval: Interval::Oneshot,
                }],
            },
            tx,
        );

        let header = status_controller.get_header();
        assert_eq!(
            header,
            r#"{"version":1,"click_events":false,"cont_signal":18,"stop_signal":19}
["#
        );
    }
}
