mod alacritty;
mod apple_terminal;
mod ghostty;
mod iterm2;
mod kitty;
mod wezterm;

use anyhow::Result;

use crate::types::TerminalEmulator;

pub fn detect_all() -> Result<Vec<TerminalEmulator>> {
    let mut terminals = Vec::new();

    if let Ok(Some(t)) = iterm2::detect() {
        terminals.push(t);
    }

    if let Ok(Some(t)) = apple_terminal::detect() {
        terminals.push(t);
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

    Ok(terminals)
}
