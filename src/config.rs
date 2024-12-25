use std::{fs, path::Path, time::Duration};

use anyhow::{bail, Result};
use serde::{de, Deserialize};

#[derive(Debug, Deserialize, PartialEq)]
pub struct Config {
    #[serde(alias = "section")]
    pub sections: Vec<Section>,
}

impl Config {
    pub fn from_file(config_path: &Path) -> Result<Self> {
        if !config_path.is_file() {
            bail!("Config file doesn't exist or is not a regular file")
        }
        Ok(
            toml::from_str(&fs::read_to_string(config_path).expect("Cannot read config file"))
                .expect("Config file format error"),
        )
    }
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct Section {
    pub name: String,
    pub command: String,
    #[serde(default)]
    pub interval: Interval,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Interval {
    Oneshot,
    Seconds(Duration),
}

impl Default for Interval {
    fn default() -> Self {
        Self::Seconds(Duration::from_secs(1))
    }
}

impl<'de> Deserialize<'de> for Interval {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value: serde_json::Value = Deserialize::deserialize(deserializer)?;
        match value {
            serde_json::Value::String(s) if s.to_lowercase() == "oneshot" => Ok(Interval::Oneshot),
            serde_json::Value::Number(n) if n.is_u64() => {
                let secs = n.as_u64().unwrap();
                if secs >= 1 {
                    Ok(Interval::Seconds(Duration::from_secs(secs)))
                } else {
                    Err(de::Error::custom("Interval must be greater or equal `1`"))
                }
            }
            _ => Err(de::Error::custom("Invalid interval value")),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::{Config, Interval, Section};

    #[rstest]
    fn should_parse_config() {
        let config_data = r#"
            [[section]]
            name = "oneshot interval"
            command = "uname -r"
            interval = "oneshot"

            [[section]]
            name = "default interval"
            command = 'date "+%Y-%m-%d %H:%M:%S"'

            [[section]]
            name = "custom interval"
            command = 'date "+%Y-%m-%d %H:%M"'
            interval = 60
        "#;

        let config: Config = toml::from_str(config_data).unwrap();

        assert_eq!(
            config,
            Config {
                sections: vec![
                    Section {
                        name: "oneshot interval".to_string(),
                        command: "uname -r".to_string(),
                        interval: Interval::Oneshot,
                    },
                    Section {
                        name: "default interval".to_string(),
                        command: r#"date "+%Y-%m-%d %H:%M:%S""#.to_string(),
                        interval: Interval::default(),
                    },
                    Section {
                        name: "custom interval".to_string(),
                        command: r#"date "+%Y-%m-%d %H:%M""#.to_string(),
                        interval: Interval::Seconds(Duration::from_secs(60)),
                    },
                ]
            }
        );
    }
}
