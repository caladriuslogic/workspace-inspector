use anyhow::Result;
use std::process::Command;

use crate::process;
use crate::types::{TerminalEmulator, TerminalTab, TerminalWindow};

pub fn detect() -> Result<Option<TerminalEmulator>> {
    let pids = process::find_pids_by_name("kitty");
    if pids.is_empty() {
        return Ok(None);
    }

    // Try kitty's remote control JSON API via each running instance
    let mut all_windows = Vec::new();

    for pid in &pids {
        let sock = format!("/tmp/kitty-sock-{}", pid);
        let output = Command::new("kitty")
            .args(["@", "--to", &format!("unix:{}", sock), "ls"])
            .output();

        let output = match output {
            Ok(o) if o.status.success() => o,
            _ => continue,
        };

        let json: serde_json::Value =
            match serde_json::from_slice(&output.stdout) {
                Ok(v) => v,
                Err(_) => continue,
            };

        // kitty ls returns an array of OS windows
        if let Some(os_windows) = json.as_array() {
            for os_win in os_windows {
                let win_id = os_win
                    .get("id")
                    .and_then(|v| v.as_u64())
                    .map(|v| v.to_string())
                    .unwrap_or_default();

                let mut tabs = Vec::new();
                if let Some(kitty_tabs) = os_win.get("tabs").and_then(|v| v.as_array()) {
                    for kt in kitty_tabs {
                        let title = kt
                            .get("title")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();

                        // Each kitty tab has windows (what kitty calls "windows" inside tabs)
                        // For simplicity, report the active/first kitty-window per tab
                        if let Some(kitty_windows) =
                            kt.get("windows").and_then(|v| v.as_array())
                        {
                            for kw in kitty_windows {
                                let fg_pid = kw
                                    .get("foreground_processes")
                                    .and_then(|v| v.as_array())
                                    .and_then(|a| a.first())
                                    .and_then(|p| p.get("pid"))
                                    .and_then(|v| v.as_u64())
                                    .map(|v| v as u32);

                                let cwd = kw
                                    .get("cwd")
                                    .and_then(|v| v.as_str())
                                    .map(|s| s.to_string())
                                    .or_else(|| fg_pid.and_then(process::get_cwd));

                                let cols = kw
                                    .get("columns")
                                    .and_then(|v| v.as_u64())
                                    .map(|v| v as u32);
                                let rows = kw
                                    .get("lines")
                                    .and_then(|v| v.as_u64())
                                    .map(|v| v as u32);

                                let cmd = kw
                                    .get("foreground_processes")
                                    .and_then(|v| v.as_array())
                                    .and_then(|a| a.first())
                                    .and_then(|p| p.get("cmdline"))
                                    .and_then(|v| v.as_array())
                                    .and_then(|a| a.first())
                                    .and_then(|v| v.as_str())
                                    .map(|s| {
                                        let name = s.rsplit('/').next().unwrap_or(s);
                                        name.strip_prefix('-')
                                            .unwrap_or(name)
                                            .to_string()
                                    });

                                tabs.push(TerminalTab {
                                    title: title.clone(),
                                    tty: None,
                                    shell_pid: fg_pid,
                                    shell: cmd,
                                    cwd,
                                    columns: cols,
                                    rows,
                                });
                            }
                        }
                    }
                }

                all_windows.push(TerminalWindow {
                    id: win_id,
                    tabs,
                });
            }
        }
    }

    Ok(Some(TerminalEmulator {
        app: "kitty".into(),
        pid: pids.first().copied(),
        windows: all_windows,
    }))
}
