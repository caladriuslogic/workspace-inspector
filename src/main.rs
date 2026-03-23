mod process;
mod shelldon;
mod terminals;
mod tmux;
mod types;
mod zellij;

use anyhow::Result;
use clap::{Parser, Subcommand};
use types::InspectorOutput;

#[derive(Parser)]
#[command(name = "terminal-inspector", about = "Inspect running terminals and multiplexer sessions")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    /// Pretty-print human-readable output instead of JSON
    #[arg(long, global = true)]
    pretty: bool,
}

#[derive(Subcommand)]
enum Command {
    /// Detect running terminal emulators
    Terminals,
    /// Detect tmux sessions
    Tmux,
    /// Detect shelldon instances
    Shelldon,
    /// Detect zellij sessions
    Zellij,
    /// Show everything (default)
    All,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let command = cli.command.unwrap_or(Command::All);

    let empty = InspectorOutput {
        terminals: vec![],
        tmux: vec![],
        shelldon: vec![],
        zellij: vec![],
    };

    let output = match command {
        Command::Terminals => InspectorOutput {
            terminals: terminals::detect_all()?,
            ..empty
        },
        Command::Tmux => InspectorOutput {
            tmux: tmux::detect()?,
            ..empty
        },
        Command::Shelldon => InspectorOutput {
            shelldon: shelldon::detect()?,
            ..empty
        },
        Command::Zellij => InspectorOutput {
            zellij: zellij::detect()?,
            ..empty
        },
        Command::All => InspectorOutput {
            terminals: terminals::detect_all()?,
            tmux: tmux::detect()?,
            shelldon: shelldon::detect()?,
            zellij: zellij::detect()?,
        },
    };

    if cli.pretty {
        print_pretty(&output);
    } else {
        println!("{}", serde_json::to_string(&output)?);
    }

    Ok(())
}

fn print_pretty(output: &InspectorOutput) {
    if !output.terminals.is_empty() {
        println!("== Terminals ==\n");
        for term in &output.terminals {
            let pid_str = term
                .pid
                .map(|p| format!(" (pid {})", p))
                .unwrap_or_default();
            println!("{}{}", term.app, pid_str);

            for win in &term.windows {
                println!("  Window {}", win.id);
                for (i, tab) in win.tabs.iter().enumerate() {
                    let tty_str = tab
                        .tty
                        .as_ref()
                        .map(|t| format!(" [{}]", t))
                        .unwrap_or_default();
                    let shell_str = tab
                        .shell
                        .as_ref()
                        .map(|s| {
                            let pid_part = tab
                                .shell_pid
                                .map(|p| format!(" (pid {})", p))
                                .unwrap_or_default();
                            format!(" {}{}", s, pid_part)
                        })
                        .unwrap_or_default();
                    let cwd_str = tab
                        .cwd
                        .as_ref()
                        .map(|c| format!(" @ {}", c))
                        .unwrap_or_default();
                    let size_str = match (tab.columns, tab.rows) {
                        (Some(c), Some(r)) => format!(" [{}x{}]", c, r),
                        _ => String::new(),
                    };
                    println!(
                        "    Tab {}: \"{}\"{}{}{}{}",
                        i + 1,
                        tab.title,
                        tty_str,
                        shell_str,
                        cwd_str,
                        size_str
                    );
                }
            }
            println!();
        }
    }

    if !output.tmux.is_empty() {
        println!("== tmux ==\n");
        for session in &output.tmux {
            let attached_str = if session.attached {
                " (attached)"
            } else {
                " (detached)"
            };
            println!("Session \"{}\"{}", session.name, attached_str);

            for win in &session.windows {
                let active_str = if win.active { " [active]" } else { "" };
                println!("  Window {}: {}{}", win.index, win.name, active_str);

                for pane in &win.panes {
                    let active_str = if pane.active { " *" } else { "" };
                    println!(
                        "    Pane {}: {} @ {} [{}x{}] (pid {}){}",
                        pane.index,
                        pane.command,
                        pane.cwd,
                        pane.width,
                        pane.height,
                        pane.pid,
                        active_str
                    );
                }
            }
            println!();
        }
    }

    if !output.shelldon.is_empty() {
        println!("== Shelldon ==\n");
        for inst in &output.shelldon {
            let tty_str = inst
                .tty
                .as_ref()
                .map(|t| format!(" [{}]", t))
                .unwrap_or_default();
            println!(
                "Instance {} (pid {}, port {}){}",
                inst.session_id, inst.pid, inst.port, tty_str
            );

            for pane in &inst.panes {
                let focus_str = if pane.is_focused { " *" } else { "" };
                println!("  Pane {}: {}{}", pane.pane_id, pane.name, focus_str);

                for tab in &pane.tabs {
                    let active_str = if tab.is_active { " [active]" } else { "" };
                    println!(
                        "    Tab {}: \"{}\" ({}){}",
                        tab.tab_id, tab.title, tab.pane_type, active_str
                    );
                }
            }
            println!();
        }
    }

    if !output.zellij.is_empty() {
        println!("== Zellij ==\n");
        for session in &output.zellij {
            println!("Session \"{}\"", session.name);

            for tab in &session.tabs {
                let active_str = if tab.active { " [active]" } else { "" };
                println!("  Tab {}: {}{}", tab.position, tab.name, active_str);

                for pane in &tab.panes {
                    let focus_str = if pane.focused { " *" } else { "" };
                    let cmd_str = if pane.command.is_empty() {
                        String::new()
                    } else {
                        format!("{} ", pane.command)
                    };
                    println!(
                        "    Pane {}: {}@ {} [{}x{}]{}",
                        pane.pane_id, cmd_str, pane.cwd, pane.columns, pane.rows, focus_str
                    );
                }
            }
            println!();
        }
    }

    let all_empty = output.terminals.is_empty()
        && output.tmux.is_empty()
        && output.shelldon.is_empty()
        && output.zellij.is_empty();

    if all_empty {
        println!("No terminals or multiplexer sessions detected.");
    }
}
