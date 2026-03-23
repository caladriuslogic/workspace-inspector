use anyhow::Result;
use std::collections::BTreeMap;
use std::process::Command;

use crate::process;
use crate::types::{TerminalEmulator, TerminalTab, TerminalWindow};

pub fn detect() -> Result<Option<TerminalEmulator>> {
    let pids = process::find_pids_by_name("WezTerm");
    if pids.is_empty() {
        // Also try lowercase for CLI-launched instances
        let pids2 = process::find_pids_by_name("wezterm-gui");
        if pids2.is_empty() {
            return Ok(None);
        }
    }

    let output = Command::new("wezterm")
        .args(["cli", "list", "--format", "json"])
        .output();

    let output = match output {
        Ok(o) if o.status.success() => o,
        _ => {
            return Ok(Some(TerminalEmulator {
                app: "WezTerm".into(),
                pid: pids.first().copied(),
                windows: vec![],
            }));
        }
    };

    let entries: Vec<serde_json::Value> = serde_json::from_slice(&output.stdout)?;

    // Group by window_id, then by tab_id
    let mut windows_map: BTreeMap<String, Vec<TerminalTab>> = BTreeMap::new();

    for entry in &entries {
        let window_id = entry
            .get("window_id")
            .and_then(|v| v.as_u64())
            .map(|v| v.to_string())
            .unwrap_or_default();

        let title = entry
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let cwd = entry
            .get("cwd")
            .and_then(|v| v.as_str())
            .map(|s| {
                // WezTerm returns file:// URLs
                s.strip_prefix("file://localhost")
                    .or_else(|| s.strip_prefix("file://"))
                    .unwrap_or(s)
                    .to_string()
            });

        let (cols, rows) = entry
            .get("size")
            .map(|s| {
                let cols = s.get("cols").and_then(|v| v.as_u64()).map(|v| v as u32);
                let rows = s.get("rows").and_then(|v| v.as_u64()).map(|v| v as u32);
                (cols, rows)
            })
            .unwrap_or((None, None));

        let tab = TerminalTab {
            title,
            tty: None,
            shell_pid: entry
                .get("pane_id")
                .and_then(|v| v.as_u64())
                .map(|v| v as u32),
            shell: None,
            cwd,
            columns: cols,
            rows,
        };

        windows_map.entry(window_id).or_default().push(tab);
    }

    let windows: Vec<TerminalWindow> = windows_map
        .into_iter()
        .map(|(id, tabs)| TerminalWindow { id, tabs })
        .collect();

    Ok(Some(TerminalEmulator {
        app: "WezTerm".into(),
        pid: pids.first().copied(),
        windows,
    }))
}
