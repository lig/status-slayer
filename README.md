# Status Slayer

Status Slayer is a configurable implementation of the `status` command for [Sway WM](https://swaywm.org/) using the Swaybar Protocol. Written in Rust, it provides a fast and highly customizable way to display status information in your Swaybar.

## Features
- **Flexible Configuration**: Configure commands and intervals using a simple [TOML](https://toml.io/) format.
- **Efficient Updates**: Sends updates to Swaybar only when a command's output changes, ensuring minimal latency.
- **Custom Intervals**: Supports interval-based commands or one-shot execution for static values.

### Planned Features
- Configurable click actions for sections.
- Color configuration per section.
- Pango markup support for rich text formatting.
- Built-in modules (e.g., `hostname`, `date`, `memory`, `cpu`, `network`) that eliminate the need for external commands.

## Installation

Install using Cargo:
```bash
cargo install --locked stlayer
```

## Usage

Add the following to your Sway configuration file (`~/.config/sway/config`):

```bash
status_command stslayer --config <path to config>
```

Replace `<path to config>` with the path to your configuration file.

### Configuration

Status Slayer uses a TOML-based configuration file. Below is an example:

```toml
[[section]]
name = "kernel name"
command = "uname -s"
interval = "oneshot"

[[section]]
name = "date and time"
command = 'date "+%Y-%m-%d %H:%M:%S"'
```

### Configuration Options
- **`name`**: A label for the section.
- **`command`**: The shell command to execute for the section.
- **`interval`**: The interval in seconds to execute the command (default: `1`). Use "oneshot" for commands that run only once.

### How It Works
- Each command runs at the defined interval, and the last known output is displayed in the status bar.
- Status Slayer sends updates to Swaybar immediately whenever a command's output changes.

## Contributing

Contributions are welcome! If you find a bug, have a feature request, or want to contribute code, feel free to:
- Open an issue: [https://codeberg.org/lig/status-slayer/issues](https://codeberg.org/lig/status-slayer/issues)
- Fork the repository and submit a pull request.

## License

This project is licensed under the MIT License. See the [LICENSE](./LICENSE) file for details.

## Links
- **Repository**: [https://codeberg.org/lig/status-slayer](https://codeberg.org/lig/status-slayer)
- **Issues**: [https://codeberg.org/lig/status-slayer/issues](https://codeberg.org/lig/status-slayer/issues)
