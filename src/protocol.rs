/**
 * Swaybar Protocol implementation.
 * See: https://man.archlinux.org/man/swaybar-protocol.7.en
 */
use serde::Serialize;
use signal_hook::consts::{SIGCONT, SIGSTOP};

#[derive(Debug, Serialize)]
pub struct Header {
    pub version: u8,
    pub click_events: bool,
    pub cont_signal: i32,
    pub stop_signal: i32,
}

impl Header {
    pub fn new() -> Self {
        Self {
            version: 1,
            // TODO: Handle click events
            click_events: false,
            cont_signal: SIGCONT,
            stop_signal: SIGSTOP,
        }
    }
}

impl Default for Header {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum MinWidth {
    Pixels(u32),
    WidthOf(String),
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Align {
    Left,
    Right,
    Center,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Markup {
    Pango,
    None,
}

#[derive(Debug, Serialize)]
#[serde(transparent)]
pub struct Status {
    pub blocks: Vec<Block>,
}

#[derive(Debug, Serialize)]
pub struct Block {
    // The text that will be displayed. If missing, the block will be skipped
    pub full_text: String,
    // If given and the text needs to be shortened due to space, this will be displayed instead of full_text
    #[serde(skip_serializing_if = "Option::is_none")] 
    pub short_text: Option<String>,
    // The text color to use in #RRGGBBAA or #RRGGBB notation
    #[serde(skip_serializing_if = "Option::is_none")] 
    pub color: Option<String>,
    // The background color for the block in #RRGGBBAA or #RRGGBB notation
    #[serde(skip_serializing_if = "Option::is_none")] 
    pub background: Option<String>,
    // The border color for the block in #RRGGBBAA or #RRGGBB notation
    #[serde(skip_serializing_if = "Option::is_none")] 
    pub border: Option<String>,
    // The height in pixels of the top border. The default is 1
    #[serde(skip_serializing_if = "Option::is_none")] 
    pub border_top: Option<u32>,
    // The height in pixels of the bottom border. The default is 1
    #[serde(skip_serializing_if = "Option::is_none")] 
    pub border_bottom: Option<u32>,
    // The width in pixels of the left border. The default is 1
    #[serde(skip_serializing_if = "Option::is_none")] 
    pub border_left: Option<u32>,
    // The width in pixels of the right border. The default is 1
    #[serde(skip_serializing_if = "Option::is_none")] 
    pub border_right: Option<u32>,
    /* The minimum width to use for the block. This can either be given in pixels or
    a string can be given to allow for it to be calculated based on the width of the
    string. */
    pub min_width: MinWidth,
    /* If the text does not span the full width of the block, this specifies how the
    text should be aligned inside of the block. This can be left (default), right, or
    center. */
    #[serde(skip_serializing_if = "Option::is_none")] 
    pub align: Option<Align>,
    /* A name for the block. This is only used to identify the block for click events.
    If set, each block should have a unique name and instance pair. */
    pub name: String,
    /* The instance of the name for the block. This is only used to identify the block
    for click events. If set, each block should have a unique name and instance pair. */
    pub instance: String,
    /* Whether the block should be displayed as urgent. Currently swaybar utilizes the
    colors set in the sway config for urgent workspace buttons. See sway-bar(5) for more
    information on bar color configuration. */
    pub urgent: bool,
    /* Whether the bar separator should be drawn after the block. See sway-bar(5) for
    more information on how to set the separator text. */
    pub separator: bool,
    /* The amount of pixels to leave blank after the block. The separator text will be
    displayed centered in this gap. The default is 9 pixels. */
    #[serde(skip_serializing_if = "Option::is_none")] 
    pub separator_block_width: Option<u32>,
    /* The type of markup to use when parsing the text for the block. This can either be
    pango or none (default). */
    pub markup: Markup,
}

impl Block {
    pub fn new(full_text: &str, name: &str, instance: &str) -> Self {
        Self {
            full_text: full_text.to_string(),
            short_text: None,
            color: None,
            background: None,
            border: None,
            border_top: None,
            border_bottom: None,
            border_left: None,
            border_right: None,
            min_width: MinWidth::WidthOf(full_text.to_string()),
            align: None,
            name: name.to_string(),
            instance: instance.to_string(),
            urgent: false,
            separator: true,
            separator_block_width: None,
            markup: Markup::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::Block;

    #[rstest]
    fn should_implement_swaybar_protocol() {
        let block1 = Block::new(
            "test full text 1",
            "test name 1",
            "test instance 1",
        );
        let block2 = Block::new(
            "test full text 2",
            "test name 2",
            "test instance 2",
        );

        let status_json = serde_json::to_string_pretty(&vec![block1, block2]).unwrap();

        println!("{}", status_json);
        assert_eq!(
            status_json,
            r##"[
  {
    "full_text": "test full text 1",
    "min_width": "test full text 1",
    "name": "test name 1",
    "instance": "test instance 1",
    "urgent": false,
    "separator": true,
    "markup": "none"
  },
  {
    "full_text": "test full text 2",
    "min_width": "test full text 2",
    "name": "test name 2",
    "instance": "test instance 2",
    "urgent": false,
    "separator": true,
    "markup": "none"
  }
]"##
        );
    }
}
