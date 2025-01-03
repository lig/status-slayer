use std::process::Command;
use std::time::Instant;

use fork::{daemon, Fork};
use tokio::{sync::mpsc, time::sleep};

use crate::protocol::Event;
use crate::{
    config::{Interval, Section},
    protocol::Block,
};

pub(super) struct SectionController {
    config: Section,
    sender: mpsc::Sender<Block>,
}

impl SectionController {
    pub(super) fn new(config: Section, sender: mpsc::Sender<Block>) -> Self {
        Self { config, sender }
    }

    pub(super) async fn run(&self) {
        loop {
            let tick = Instant::now();
            let output = Command::new("sh")
                .args(["-c", &self.config.command])
                .output()
                .unwrap_or_else(|_| {
                    panic!("Failed to execute command `{}`", &self.config.command)
                });
            if !output.status.success() {
                panic!(
                    "Command `{}` failed with error:\n{}",
                    &self.config.command,
                    String::from_utf8_lossy(&output.stderr)
                );
            }

            let stdout = String::from_utf8_lossy(output.stdout.trim_ascii_end());

            self.sender
                .send(Block::new("command", &self.config.name, &stdout))
                .await
                .unwrap();

            match self.config.interval {
                Interval::Oneshot => break,
                Interval::Seconds(duration) => sleep(duration - tick.elapsed()).await,
            }
        }
    }

    pub(super) async fn on_click(&self, _event: Event) {
        if let Some(on_click) = &self.config.on_click {
            if let Ok(Fork::Child) = daemon(false, true) {
                Command::new("sh")
                    .args(["-c", on_click])
                    .output()
                    .unwrap_or_else(|_| {
                        panic!("Failed to execute command `{}`", &self.config.command)
                    });
            }
        }
    }
}
