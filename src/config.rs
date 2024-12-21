use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
pub struct Config {
    #[serde(alias = "section")]
    pub sections: Vec<Section>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Section {
    pub name: String,
    pub command: String,
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::{Config, Section};

    #[rstest]
    fn should_parse_config() {
        let config_data = r#"
            [[section]]
            name = "first section"
            command = "uname -r"

            [[section]]
            name = "second section"
            command = 'date "+%Y-%m-%d %H:%M:%S"'
        "#;

        let config: Config = toml::from_str(config_data).unwrap();

        assert_eq!(
            config,
            Config {
                sections: vec![
                    Section {
                        name: "first section".to_string(),
                        command: "uname -r".to_string(),
                    },
                    Section {
                        name: "second section".to_string(),
                        command: r#"date "+%Y-%m-%d %H:%M:%S""#.to_string(),
                    },
                ]
            }
        );
    }
}
