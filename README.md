# mcps

Small Rust CLI to enable/disable Codex MCP servers in your Codex config.

## What it does

`mcps` edits the `enabled` flag under `[mcp_servers.<name>]` in a TOML config file.

- Default config path: `~/.codex/config.toml`
- Override config path: `--config /path/to/config.toml`

## Features

- Enable a configured MCP server
- Disable a configured MCP server
- List all configured MCP servers with status
- Writes updates safely via temp file + replace

## Requirements

- Rust toolchain (`cargo`)
- A valid Codex config containing `[mcp_servers]`

## Installation

### Option 1: Build + install script

```bash
./build.sh
```

This script:

- Builds release binary (`cargo build --release`)
- Installs `mcps` to `~/.local/bin` by default
- Prints whether your `PATH` already includes the install directory

Set a custom install directory:

```bash
MCPS_INSTALL_DIR="$HOME/bin" ./build.sh
```

### Option 2: Cargo install from local path

```bash
cargo install --path .
```

## Usage

```bash
mcps list
mcps enable <name>
mcps disable <name>
mcps --config /path/to/config.toml list
```

Run help:

```bash
mcps --help
```

## Config expectations

Example config:

```toml
[mcp_servers.xcode]
command = "xcrun"
args = ["mcpbridge"]
enabled = true
```

Notes:

- If `enabled` is missing for a server, `list` treats it as enabled.
- `enable`/`disable` requires the server to already exist in `[mcp_servers]`.

## Output examples

`mcps list`:

```text
Name   Status
xcode  enabled
```

`mcps disable xcode`:

```text
xcode disabled
```

`mcps enable xcode`:

```text
xcode enabled
```

## Error cases

You can get errors when:

- `[mcp_servers]` section is missing
- Requested server name is not found
- `enabled` exists but is not boolean
- Config file cannot be read, parsed, or replaced

## Development

Build:

```bash
cargo build
```

Run tests:

```bash
cargo test
```

Install release build:

```bash
./build.sh
```

## Troubleshooting

- `mcps: command not found`: add install directory (for example `~/.local/bin`) to `PATH`.
- Permission errors writing config: verify file ownership and directory permissions.
- TOML parse errors: validate your config syntax and table structure.

## License

No license file is currently included in this repository.
