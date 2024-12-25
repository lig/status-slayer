use assert_cmd::prelude::*;

use pretty_assertions::assert_eq;
use regex::Regex;
use rstest::rstest;
use std::{
    io::Read,
    process::{Command, Stdio},
    time::Duration,
};
use wait_timeout::ChildExt;

#[rstest]
fn test_config_example() {
    let mut cmd = Command::cargo_bin("stslayer").unwrap();

    let mut child = cmd
        .args(["--config", "./tests/config-example.toml"])
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child.wait_timeout(Duration::from_millis(1500)).unwrap();
    child.kill().unwrap();
    child.wait().unwrap();

    let mut stdout = String::new();
    child.stdout.unwrap().read_to_string(&mut stdout).unwrap();
    println!("{}", stdout);

    let mut stdout_lines = stdout.split('\n');
    assert_eq!(
        stdout_lines.next().unwrap(),
        r#"{"version":1,"click_events":false,"cont_signal":18,"stop_signal":19}"#
    );
    assert_eq!(stdout_lines.next().unwrap(), "[");
    for line in stdout_lines.by_ref().take(2) {
        assert!(
            Regex::new(
                r#"^\[\{"name":"command","instance":"kernel\ release","full_text":"Linux","min_width":"Linux","urgent":false,"separator":true,"markup":"none"\},\{"name":"command","instance":"date\ and\ time","full_text":"\d{4}\-\d{2}\-\d{2}\ \d{1,2}:\d{2}:\d{2}","min_width":"\d{4}\-\d{2}\-\d{2}\ \d{1,2}:\d{2}:\d{2}","urgent":false,"separator":true,"markup":"none"\}\],$"#
            ).unwrap().is_match(line),
            "{}",
            line
        );
    }
    assert_eq!(stdout_lines.next(), Some(""));
    assert_eq!(stdout_lines.next(), None, "Extra stdout");
}
