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

    pub(super) fn on_click(&self, _event: Event) {
        if let Some(cmd) = &self.config.on_click {
            self.handle_click(cmd);
        }
    }

    pub(super) fn on_secondary_click(&self, _event: Event) {
        if let Some(cmd) = &self.config.on_secondary_click {
            self.handle_click(cmd);
        }
    }

    fn handle_click(&self, cmd: &str) {
        if let Ok(Fork::Child) = daemon(false, true) {
            Command::new("sh")
                .args(["-c", cmd])
                .output()
                .unwrap_or_else(|_| {
                    panic!("Failed to execute command `{}`", &self.config.command)
                });
        }
    }
}
