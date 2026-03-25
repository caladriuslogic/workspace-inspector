# workspace-inspector

A Rust library and CLI tool to inspect your workspace — terminal emulators, multiplexer sessions, and more. Outputs JSON by default for easy integration with other tools.

## Supported

### Terminal Emulators

| Terminal | Detection | Tabs/Windows | CWD | Shell | Dimensions |
|----------|-----------|-------------|-----|-------|------------|
| iTerm2 | AppleScript | Yes | Yes | Yes | Yes |
| Terminal.app | AppleScript | Yes | Yes | Yes | Yes |
| kitty | JSON socket API | Yes | Yes | Yes | Yes |
| WezTerm | `wezterm cli` | Yes | Yes | — | Yes |
| Ghostty | Process only | — | — | — | — |
| Alacritty | Process only | — | — | — | — |

### Multiplexers

| Multiplexer | Sessions | Tabs/Windows | Panes | CWD | Command | Dimensions |
|-------------|----------|-------------|-------|-----|---------|------------|
| tmux | Yes | Yes | Yes | Yes | Yes | Yes |
| Zellij | Yes | Yes | Yes | Yes | Yes | Yes |
| Shelldon | Yes (via MCP) | Yes | Yes | — | — | — |

## Install

```sh
cargo install workspace-inspector
```

Or from source:

```sh
cargo install --path .
```

## Usage

```sh
# Show everything (JSON)
workspace-inspector

# Show everything (human-readable)
workspace-inspector --pretty

# Only terminal emulators
workspace-inspector terminals

# Only tmux sessions
workspace-inspector tmux

# Only shelldon instances
workspace-inspector shelldon

# Only zellij sessions
workspace-inspector zellij

# Print canonical URI for current workspace location
workspace-inspector where
# => workspace://iterm2/window:1229/tab:3/tmux:main/window:1/pane:0
```

## JSON Output

Default output is compact JSON. Example:

```json
{
  "terminals": [
    {
      "app": "iTerm2",
      "pid": 697,
      "windows": [
        {
          "id": "1229",
          "tabs": [
            {
              "title": "Default",
              "tty": "/dev/ttys000",
              "shell_pid": 20415,
              "shell": "bash",
              "cwd": "/Users/jmorrow",
              "columns": 249,
              "rows": 102
            }
          ]
        }
      ]
    }
  ],
  "tmux": [
    {
      "name": "main",
      "id": "$0",
      "attached": true,
      "windows": [
        {
          "index": 1,
          "name": "editor",
          "active": true,
          "panes": [
            {
              "index": 1,
              "pid": 12345,
              "command": "nvim",
              "cwd": "/home/user/project",
              "width": 200,
              "height": 50,
              "active": true
            }
          ]
        }
      ]
    }
  ]
}
```

Empty sections are omitted from the output.

## Library Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
workspace-inspector = { git = "https://github.com/caladriuslogic/workspace-inspector" }
```

```rust
use workspace_inspector::{locate, inspect_all, inspect_tmux};

// Get a canonical URI for the current workspace location
let uri = locate()?;
// => "workspace://iterm2/window:1229/tab:3/tmux:main/window:1/pane:0"

// Inspect everything
let output = inspect_all()?;
for term in &output.terminals {
    println!("{} ({:?})", term.app, term.pid);
}

// Or inspect specific subsystems
let sessions = inspect_tmux()?;
```

## Requirements

- macOS: Automation permissions may be required for iTerm2 and Terminal.app inspection
- Linux and Windows support is planned

## Notes

- **Ghostty** and **Alacritty** are detected as running but don't expose APIs for tab/window introspection yet
- **Shelldon** instances are discovered via `/tmp/shelldon-{pid}.json` files and queried over their MCP HTTP API
- **WezTerm** requires the `wezterm` CLI to be on your PATH
- **Zellij** requires `zellij` to be on your PATH

## License

MIT
