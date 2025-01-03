use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use itertools::Itertools;
use serde::Serialize;
use tokio::sync::mpsc::Receiver;
use tokio::{sync::mpsc, time::sleep};

use super::{EventListener, SectionController};
use crate::protocol::Event;
use crate::{
    config::Config,
    protocol::{Block, Header, Status},
};

pub struct StatusController {
    config: Config,
    status_sender: mpsc::Sender<String>,
    block_receiver: Receiver<Block>,
    pretty: bool,
    section_registry: HashMap<SectionId, SectionRecord>,
    status: Status,
}

struct SectionRecord {
    order: usize,
    controller: Arc<SectionController>,
}

#[derive(PartialEq, Eq, Hash)]
struct SectionId {
    module: String,
    name: String,
}

impl StatusController {
    pub fn new(config: Config, sender: mpsc::Sender<String>) -> Self {
        assert!(
            !config.sections.is_empty(),
            "At least one section must be defined in config"
        );
        let (block_sender, block_receiver) = mpsc::channel::<Block>(1);
        let section_registry: HashMap<SectionId, SectionRecord> = config
            .sections
            .iter()
            .enumerate()
            .map(|(n, section)| {
                (
                    SectionId::new("command", &section.name),
                    SectionRecord {
                        order: n,
                        controller: Arc::new(SectionController::new(
                            section.to_owned(),
                            block_sender.clone(),
                        )),
                    },
                )
            })
            .collect();
        let num_sections = section_registry.len();
        StatusController {
            config,
            status_sender: sender,
            block_receiver,
            pretty: false,
            section_registry,
            status: Status {
                blocks: Vec::with_capacity(num_sections),
            },
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        self.spawn_section_controllers();
        let mut event_receiver = self.spawn_event_listener();

        self.send_header().await;
        self.initialize_status().await;

        let mut dirty = false;
        loop {
            tokio::select! {
                block = self.block_receiver.recv() => {
                    let block = block.unwrap();

                    let section_record = &self.section_registry[&SectionId::new(&block.name, &block.instance)];

                    if self.status.blocks[section_record.order] == block {
                        continue;
                    }

                    self.status.blocks[section_record.order] = block;
                    dirty = true;
                }
                event = event_receiver.recv() => {
                    let event = event.unwrap();

                    let section_controller = Arc::clone(
                        &self.section_registry[
                                &SectionId::new(&event.name, &event.instance)].controller);

                    section_controller.on_click(event).await;
                }
                _ = sleep(self.config.min_interval) => {
                    if !dirty {
                        continue
                    }
                    self.status_sender.send(self.get_status()).await.unwrap();
                    dirty = false;
                }
            }
        }
    }

    async fn send_header(&mut self) {
        self.status_sender.send(self.get_header()).await.unwrap();
    }

    fn spawn_section_controllers(&self) {
        for section_controller in self
            .section_registry
            .values()
            .map(|record| Arc::clone(&record.controller))
        {
            tokio::spawn(async move {
                section_controller.run().await;
            });
        }
    }

    fn spawn_event_listener(&self) -> mpsc::Receiver<Event> {
        let (event_sender, event_receiver) = mpsc::channel::<Event>(1);

        tokio::spawn(async {
            let mut event_listener = EventListener::new(event_sender);
            event_listener.run().await;
        });

        event_receiver
    }

    async fn initialize_status(&mut self) {
        let mut initial_data: HashMap<SectionId, Block> = HashMap::new();

        while initial_data.len() < self.section_registry.len() {
            let block = self.block_receiver.recv().await.unwrap();
            initial_data
                .entry(SectionId::new(&block.name, &block.instance))
                .insert_entry(block);
        }

        self.status.blocks.extend(
            initial_data
                .into_iter()
                .map(|(section_id, block)| {
                    (&self.section_registry[&section_id].order, block)
                })
                .sorted_by_key(|v| v.0)
                .map(|(_, block)| block),
        );

        self.status_sender.send(self.get_status()).await.unwrap();
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

impl SectionId {
    fn new(module: &str, name: &str) -> Self {
        Self {
            module: module.to_string(),
            name: name.to_string(),
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
                    on_click: None,
                }],
            },
            tx,
        );

        let header = status_controller.get_header();
        assert_eq!(
            header,
            r#"{"version":1,"click_events":true,"cont_signal":18,"stop_signal":19}
["#
        );
    }
}
