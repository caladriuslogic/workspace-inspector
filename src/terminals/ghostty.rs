use anyhow::Result;

use crate::process;
use crate::types::TerminalEmulator;

pub fn detect() -> Result<Option<TerminalEmulator>> {
    let pids = process::find_pids_by_name("Ghostty");
    if pids.is_empty() {
        return Ok(None);
    }

    // Ghostty doesn't have an IPC API for listing windows/tabs yet.
    // We can only report that it's running.
    Ok(Some(TerminalEmulator {
        app: "Ghostty".into(),
        pid: pids.first().copied(),
        windows: vec![],
    }))
}
