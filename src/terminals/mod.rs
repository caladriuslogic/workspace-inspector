mod alacritty;
mod ghostty;
mod kitty;
mod wezterm;

#[cfg(target_os = "macos")]
mod apple_terminal;
#[cfg(target_os = "macos")]
mod iterm2;

#[cfg(target_os = "linux")]
mod gnome_terminal;

#[cfg(target_os = "windows")]
mod powershell;

use anyhow::Result;

use crate::types::TerminalEmulator;

pub fn detect_all() -> Result<Vec<TerminalEmulator>> {
    let mut terminals = Vec::new();

    #[cfg(target_os = "macos")]
    {
        if let Ok(Some(t)) = iterm2::detect() {
            terminals.push(t);
        }

        if let Ok(Some(t)) = apple_terminal::detect() {
            terminals.push(t);
        }
    }

    if let Ok(Some(t)) = kitty::detect() {
        terminals.push(t);
    }

    if let Ok(Some(t)) = ghostty::detect() {
        terminals.push(t);
    }

    if let Ok(Some(t)) = wezterm::detect() {
        terminals.push(t);
    }

    if let Ok(Some(t)) = alacritty::detect() {
        terminals.push(t);
    }

    #[cfg(target_os = "linux")]
    {
        if let Ok(Some(t)) = gnome_terminal::detect() {
            terminals.push(t);
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(Some(t)) = powershell::detect() {
            terminals.push(t);
        }
    }

    Ok(terminals)
}
