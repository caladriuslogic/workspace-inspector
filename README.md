# terminal-inspector

A CLI tool to inspect running terminal emulators and multiplexer sessions on macOS. Outputs JSON by default for easy integration with other tools.

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
cargo install --path .
```

## Usage

```sh
# Show everything (JSON)
terminal-inspector

# Show everything (human-readable)
terminal-inspector --pretty

# Only terminal emulators
terminal-inspector terminals

# Only tmux sessions
terminal-inspector tmux

# Only shelldon instances
terminal-inspector shelldon

# Only zellij sessions
terminal-inspector zellij

# Print canonical URI for current terminal location
terminal-inspector where
# => terminal://iterm2/window:1229/tab:3/tmux:main/window:1/pane:0
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

## Requirements

- macOS (uses AppleScript and macOS-specific process inspection)
- Automation permissions may be required for iTerm2 and Terminal.app inspection

## Notes

- **Ghostty** and **Alacritty** are detected as running but don't expose APIs for tab/window introspection yet
- **Shelldon** instances are discovered via `/tmp/shelldon-{pid}.json` files and queried over their MCP HTTP API
- **WezTerm** requires the `wezterm` CLI to be on your PATH
- **Zellij** requires `zellij` to be on your PATH

## License

MIT
