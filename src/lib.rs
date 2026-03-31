//! Inspect running terminal emulators, multiplexer sessions, and browsers.
//!
//! # Example
//!
//! ```no_run
//! use workspace_inspector::{inspect_all, locate};
//!
//! // Get a canonical URI for the current terminal location
//! let uri = locate().unwrap();
//! println!("{}", uri); // workspace://iterm2/window:1229/tab:3/tmux:main/window:1/pane:0
//!
//! // Inspect all running terminals and multiplexers
//! let output = inspect_all().unwrap();
//! for term in &output.terminals {
//!     println!("{} (pid {:?})", term.app, term.pid);
//! }
//! ```

pub mod ides;
mod locate;
mod process;
mod shelldon;
pub mod terminals;
pub mod tmux;
mod types;
pub mod zellij;

pub use locate::locate;
pub use types::*;

use anyhow::Result;

/// Inspect all running terminals and multiplexer sessions.
pub fn inspect_all() -> Result<InspectorOutput> {
    let mut output = InspectorOutput {
        terminals: terminals::detect_all()?,
        tmux: tmux::detect()?,
        shelldon: shelldon::detect()?,
        zellij: zellij::detect()?,
        ides: ides::detect_all()?,
    };
    output.populate_uris();
    Ok(output)
}

/// Inspect only running IDEs.
pub fn inspect_ides() -> Result<Vec<IdeInstance>> {
    let mut out = InspectorOutput::empty();
    out.ides = ides::detect_all()?;
    out.populate_uris();
    Ok(out.ides)
}

/// Inspect only running terminal emulators.
pub fn inspect_terminals() -> Result<Vec<TerminalEmulator>> {
    let mut out = InspectorOutput::empty();
    out.terminals = terminals::detect_all()?;
    out.populate_uris();
    Ok(out.terminals)
}

/// Inspect only tmux sessions.
pub fn inspect_tmux() -> Result<Vec<TmuxSession>> {
    let mut out = InspectorOutput::empty();
    out.tmux = tmux::detect()?;
    out.populate_uris();
    Ok(out.tmux)
}

/// Inspect only shelldon instances.
pub fn inspect_shelldon() -> Result<Vec<ShelldonInstance>> {
    let mut out = InspectorOutput::empty();
    out.shelldon = shelldon::detect()?;
    out.populate_uris();
    Ok(out.shelldon)
}

/// Inspect only zellij sessions.
pub fn inspect_zellij() -> Result<Vec<ZellijSession>> {
    let mut out = InspectorOutput::empty();
    out.zellij = zellij::detect()?;
    out.populate_uris();
    Ok(out.zellij)
}
