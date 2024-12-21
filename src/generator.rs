use std::thread;
use std::time::Duration;

use crate::{
    config::Config,
    protocol::{Block, Header, Status},
};

pub struct StatusGenerator {
    interval: Duration,
    header_sent: bool,
    config: Config,
}

impl StatusGenerator {
    pub fn new(config: Config) -> Self {
        StatusGenerator {
            interval: Duration::from_secs(1),
            header_sent: false,
            config,
        }
    }
}

impl Iterator for StatusGenerator {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.header_sent {
            self.header_sent = true;
            return Some(format!(
                "{}\n[",
                serde_json::to_string(&Header::new()).unwrap()
            ));
        }

        thread::sleep(self.interval);

        let mut blocks: Vec<Block> = vec![];
        for section in &self.config.sections {
            blocks.push(Block::new(
                &section.command,
                &section.command,
                "command",
                &section.name,
            ));
        }

        Some(serde_json::to_string(&Status { blocks }).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use crate::config::{Config, Section};

    use super::StatusGenerator;

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
                    command: r#"date "+%Y-%m-%d %H:%M:%S""#.to_string(),
                },
            ],
        });

        let header = status_generator.next().unwrap();
        assert_eq!(
            header,
            r#"{"version":1,"click_events":false,"cont_signal":18,"stop_signal":19}
["#
        );

        let statuses: Vec<String> = status_generator.take(3).collect();
        assert_eq!(
            statuses,
            vec![
                r##"[{"full_text":"uname -r","short_text":"uname -r","color":"#000000","background":"#ffffff","border":"#000000","border_top":1,"border_bottom":1,"border_left":1,"border_right":1,"min_width":"uname -r","align":"left","name":"command","instance":"first section","urgent":false,"separator":true,"separator_block_width":9,"markup":"none"},{"full_text":"date \"+%Y-%m-%d %H:%M:%S\"","short_text":"date \"+%Y-%m-%d %H:%M:%S\"","color":"#000000","background":"#ffffff","border":"#000000","border_top":1,"border_bottom":1,"border_left":1,"border_right":1,"min_width":"date \"+%Y-%m-%d %H:%M:%S\"","align":"left","name":"command","instance":"second section","urgent":false,"separator":true,"separator_block_width":9,"markup":"none"}]"##,
                r##"[{"full_text":"uname -r","short_text":"uname -r","color":"#000000","background":"#ffffff","border":"#000000","border_top":1,"border_bottom":1,"border_left":1,"border_right":1,"min_width":"uname -r","align":"left","name":"command","instance":"first section","urgent":false,"separator":true,"separator_block_width":9,"markup":"none"},{"full_text":"date \"+%Y-%m-%d %H:%M:%S\"","short_text":"date \"+%Y-%m-%d %H:%M:%S\"","color":"#000000","background":"#ffffff","border":"#000000","border_top":1,"border_bottom":1,"border_left":1,"border_right":1,"min_width":"date \"+%Y-%m-%d %H:%M:%S\"","align":"left","name":"command","instance":"second section","urgent":false,"separator":true,"separator_block_width":9,"markup":"none"}]"##,
                r##"[{"full_text":"uname -r","short_text":"uname -r","color":"#000000","background":"#ffffff","border":"#000000","border_top":1,"border_bottom":1,"border_left":1,"border_right":1,"min_width":"uname -r","align":"left","name":"command","instance":"first section","urgent":false,"separator":true,"separator_block_width":9,"markup":"none"},{"full_text":"date \"+%Y-%m-%d %H:%M:%S\"","short_text":"date \"+%Y-%m-%d %H:%M:%S\"","color":"#000000","background":"#ffffff","border":"#000000","border_top":1,"border_bottom":1,"border_left":1,"border_right":1,"min_width":"date \"+%Y-%m-%d %H:%M:%S\"","align":"left","name":"command","instance":"second section","urgent":false,"separator":true,"separator_block_width":9,"markup":"none"}]"##,
            ]
        );
    }
}
