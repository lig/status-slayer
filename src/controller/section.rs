use std::process::Stdio;
use std::time::Instant;

use crate::protocol::Event;
use crate::{
    config::{Interval, Section},
    protocol::Block,
};
use tokio::{process::Command, sync::mpsc, time::sleep};

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
                .await
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
        if let Some(cmd) = &self.config.on_click {
            self.handle_click(cmd).await;
        }
    }

    pub(super) async fn on_secondary_click(&self, _event: Event) {
        if let Some(cmd) = &self.config.on_secondary_click {
            self.handle_click(cmd).await;
        }
    }

    async fn handle_click(&self, cmd: &str) {
        Command::new("sh")
            .args(["-c", cmd])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap_or_else(|_| {
                panic!("Failed to execute command `{}`", &self.config.command)
            })
            .wait()
            .await
            .unwrap();
    }
}
