use anyhow::Result;

use crate::process;
use crate::types::TerminalEmulator;

pub fn detect() -> Result<Option<TerminalEmulator>> {
    let pids = process::find_pids_by_name("Alacritty");
    if pids.is_empty() {
        let pids2 = process::find_pids_by_name("alacritty");
        if pids2.is_empty() {
            return Ok(None);
        }
    }

    // Alacritty has no scripting/IPC API for listing windows.
    // We can only report that it's running.
    Ok(Some(TerminalEmulator {
        app: "Alacritty".into(),
        pid: pids.first().copied(),
        windows: vec![],
    }))
}
