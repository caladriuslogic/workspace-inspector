use anyhow::Result;
use std::process::Command;

use crate::process;
use crate::types::{TerminalEmulator, TerminalTab, TerminalWindow};

const APPLESCRIPT: &str = r#"
tell application "Terminal"
    set output to ""
    repeat with w in windows
        try
            set wid to id of w
            repeat with t in tabs of w
                try
                    set ttyVal to tty of t
                    set titleVal to custom title of t
                    set colsVal to number of columns of t
                    set rowsVal to number of rows of t
                    set output to output & wid & "\t" & titleVal & "\t" & ttyVal & "\t" & colsVal & "\t" & rowsVal & "\n"
                end try
            end repeat
        end try
    end repeat
    return output
end tell
"#;

pub fn detect() -> Result<Option<TerminalEmulator>> {
    let pids = process::find_pids_by_name("Terminal");
    if pids.is_empty() {
        return Ok(None);
    }

    let output = Command::new("osascript")
        .args(["-e", APPLESCRIPT])
        .output()?;

    if !output.status.success() {
        eprintln!(
            "warning: could not query Terminal.app via AppleScript: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
        return Ok(Some(TerminalEmulator {
            app: "Terminal".into(),
            pid: pids.first().copied(),
            windows: vec![],
        }));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut windows_map: std::collections::BTreeMap<String, Vec<TerminalTab>> =
        std::collections::BTreeMap::new();

    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 5 {
            continue;
        }

        let window_id = parts[0].to_string();
        let title = parts[1].to_string();
        let tty = parts[2].to_string();
        let cols: Option<u32> = parts[3].parse().ok();
        let rows: Option<u32> = parts[4].parse().ok();

        let (shell_pid, shell) = process::get_shell_for_tty(&tty)
            .map(|(p, s)| (Some(p), Some(s)))
            .unwrap_or((None, None));

        let cwd = shell_pid.and_then(process::get_cwd);

        let tab = TerminalTab {
            title,
            uri: None,
            tty: Some(tty),
            shell_pid,
            shell,
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
        app: "Terminal".into(),
        pid: pids.first().copied(),
        windows,
    }))
}
