use tokio::io::{self, AsyncBufReadExt, BufReader};
use tokio::sync::mpsc;

use crate::protocol::Event;

pub(super) struct EventListener {
    sender: mpsc::Sender<Event>,
}

impl EventListener {
    pub(super) fn new(sender: mpsc::Sender<Event>) -> Self {
        Self { sender }
    }

    pub(super) async fn run(&mut self) {
        let reader = BufReader::new(io::stdin());
        let mut lines = reader.lines();

        assert!(lines.next_line().await.unwrap() == Some("[".to_string()));

        while let Some(line) = lines.next_line().await.unwrap() {
            let event: Event =
                serde_json::from_str(line.trim_start_matches(',')).unwrap();
            self.sender.send(event).await.unwrap();
        }
    }
}
