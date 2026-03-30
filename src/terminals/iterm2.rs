use anyhow::Result;
use std::process::Command;

use crate::process;
use crate::types::{TerminalEmulator, TerminalTab, TerminalWindow};

const APPLESCRIPT: &str = r#"
tell application "iTerm2"
    set output to ""
    repeat with w in windows
        try
            set wid to id of w
            set tabIdx to 0
            repeat with t in tabs of w
                try
                    set tabIdx to tabIdx + 1
                    repeat with s in sessions of t
                        try
                            set ttyVal to tty of s
                            set titleVal to name of s
                            set colsVal to columns of s
                            set rowsVal to rows of s
                            set output to output & wid & "\t" & tabIdx & "\t" & titleVal & "\t" & ttyVal & "\t" & colsVal & "\t" & rowsVal & "\n"
                        end try
                    end repeat
                end try
            end repeat
        end try
    end repeat
    return output
end tell
"#;

pub fn detect() -> Result<Option<TerminalEmulator>> {
    let pids = process::find_pids_by_name("iTerm2");
    if pids.is_empty() {
        return Ok(None);
    }

    let output = Command::new("osascript")
        .args(["-e", APPLESCRIPT])
        .output()?;

    if !output.status.success() {
        // Likely no permission or iTerm2 not scriptable
        eprintln!(
            "warning: could not query iTerm2 via AppleScript: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
        return Ok(Some(TerminalEmulator {
            app: "iTerm2".into(),
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
        if parts.len() < 6 {
            continue;
        }

        let window_id = parts[0].to_string();
        let _tab_idx = parts[1];
        let title = parts[2].to_string();
        let tty = parts[3].to_string();
        let cols: Option<u32> = parts[4].parse().ok();
        let rows: Option<u32> = parts[5].parse().ok();

        let (shell_pid, shell) = process::get_shell_for_tty(&tty)
            .map(|(p, s)| (Some(p), Some(s)))
            .unwrap_or((None, None));

        let cwd = shell_pid.and_then(process::get_cwd);

        let tab = TerminalTab {
            title,
            tty: Some(tty),
            shell_pid,
            shell,
            cwd,
            columns: cols,
            rows,
        };

        windows_map
            .entry(window_id)
            .or_default()
            .push(tab);
    }

    let windows: Vec<TerminalWindow> = windows_map
        .into_iter()
        .map(|(id, tabs)| TerminalWindow { id, tabs })
        .collect();

    Ok(Some(TerminalEmulator {
        app: "iTerm2".into(),
        pid: pids.first().copied(),
        windows,
    }))
}
