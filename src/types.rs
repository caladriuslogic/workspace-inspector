use serde::Serialize;

#[derive(Serialize)]
pub struct InspectorOutput {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub terminals: Vec<TerminalEmulator>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tmux: Vec<TmuxSession>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub shelldon: Vec<ShelldonInstance>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub zellij: Vec<ZellijSession>,
}

#[derive(Serialize)]
pub struct TerminalEmulator {
    pub app: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pid: Option<u32>,
    pub windows: Vec<TerminalWindow>,
}

#[derive(Serialize)]
pub struct TerminalWindow {
    pub id: String,
    pub tabs: Vec<TerminalTab>,
}

#[derive(Serialize)]
pub struct TerminalTab {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tty: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shell_pid: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shell: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub columns: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rows: Option<u32>,
}

#[derive(Serialize)]
pub struct TmuxSession {
    pub name: String,
    pub id: String,
    pub attached: bool,
    pub windows: Vec<TmuxWindow>,
}

#[derive(Serialize)]
pub struct TmuxWindow {
    pub index: u32,
    pub name: String,
    pub active: bool,
    pub panes: Vec<TmuxPane>,
}

#[derive(Serialize)]
pub struct TmuxPane {
    pub index: u32,
    pub pid: u32,
    pub command: String,
    pub cwd: String,
    pub width: u32,
    pub height: u32,
    pub active: bool,
}

#[derive(Serialize)]
pub struct ShelldonInstance {
    pub pid: u32,
    pub port: u16,
    pub session_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tty: Option<String>,
    pub panes: Vec<ShelldonPane>,
}

#[derive(Serialize)]
pub struct ShelldonPane {
    pub pane_id: u32,
    pub name: String,
    pub is_focused: bool,
    pub tabs: Vec<ShelldonTab>,
}

#[derive(Serialize)]
pub struct ShelldonTab {
    pub tab_id: String,
    pub title: String,
    pub pane_type: String,
    pub is_active: bool,
}

#[derive(Serialize)]
pub struct ZellijSession {
    pub name: String,
    pub tabs: Vec<ZellijTab>,
}

#[derive(Serialize)]
pub struct ZellijTab {
    pub id: u32,
    pub position: u32,
    pub name: String,
    pub active: bool,
    pub panes: Vec<ZellijPane>,
}

#[derive(Serialize, Clone)]
pub struct ZellijPane {
    #[serde(skip)]
    pub tab_id: u32,
    pub pane_id: u32,
    pub title: String,
    pub command: String,
    pub cwd: String,
    pub columns: u32,
    pub rows: u32,
    pub focused: bool,
}
