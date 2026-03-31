use anyhow::Result;
use clap::{Parser, Subcommand};
use workspace_inspector::InspectorOutput;

#[derive(Parser)]
#[command(name = "workspace-inspector", about = "Inspect running terminals, multiplexers, and browsers")]
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
    /// Detect running IDEs (Xcode, etc.)
    Ides,
    /// Print a canonical URI for the current terminal location
    Where,
    /// Show everything (default)
    All,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let command = cli.command.unwrap_or(Command::All);

    if matches!(command, Command::Where) {
        let uri = workspace_inspector::locate()?;
        println!("{}", uri);
        return Ok(());
    }

    let empty = InspectorOutput::empty();

    let output = match command {
        Command::Terminals => InspectorOutput {
            terminals: workspace_inspector::inspect_terminals()?,
            ..empty
        },
        Command::Tmux => InspectorOutput {
            tmux: workspace_inspector::inspect_tmux()?,
            ..empty
        },
        Command::Shelldon => InspectorOutput {
            shelldon: workspace_inspector::inspect_shelldon()?,
            ..empty
        },
        Command::Zellij => InspectorOutput {
            zellij: workspace_inspector::inspect_zellij()?,
            ..empty
        },
        Command::Ides => InspectorOutput {
            ides: workspace_inspector::inspect_ides()?,
            ..empty
        },
        Command::Where => unreachable!("handled above"),
        Command::All => workspace_inspector::inspect_all()?,
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
                    let uri_str = tab
                        .uri
                        .as_ref()
                        .map(|u| format!("\n      {}", u))
                        .unwrap_or_default();
                    println!(
                        "    Tab {}: \"{}\"{}{}{}{}{}",
                        i + 1,
                        tab.title,
                        tty_str,
                        shell_str,
                        cwd_str,
                        size_str,
                        uri_str
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
                    let uri_str = pane
                        .uri
                        .as_ref()
                        .map(|u| format!("\n      {}", u))
                        .unwrap_or_default();
                    println!(
                        "    Pane {}: {} @ {} [{}x{}] (pid {}){}{}",
                        pane.index,
                        pane.command,
                        pane.cwd,
                        pane.width,
                        pane.height,
                        pane.pid,
                        active_str,
                        uri_str
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
                    let uri_str = tab
                        .uri
                        .as_ref()
                        .map(|u| format!("\n      {}", u))
                        .unwrap_or_default();
                    println!(
                        "    Tab {}: \"{}\" ({}){}{}",
                        tab.tab_id, tab.title, tab.pane_type, active_str, uri_str
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
                    let uri_str = pane
                        .uri
                        .as_ref()
                        .map(|u| format!("\n      {}", u))
                        .unwrap_or_default();
                    println!(
                        "    Pane {}: {}@ {} [{}x{}]{}{}",
                        pane.pane_id, cmd_str, pane.cwd, pane.columns, pane.rows, focus_str, uri_str
                    );
                }
            }
            println!();
        }
    }

    if !output.ides.is_empty() {
        println!("== IDEs ==\n");
        for ide in &output.ides {
            let pid_str = ide
                .pid
                .map(|p| format!(" (pid {})", p))
                .unwrap_or_default();
            println!("{}{}", ide.app, pid_str);

            for project in &ide.projects {
                let active_str = if project.active { " [active]" } else { "" };
                let uri_str = project
                    .uri
                    .as_ref()
                    .map(|u| format!("\n    {}", u))
                    .unwrap_or_default();
                println!("  {}{}", project.name, active_str);
                println!("    {}{}", project.path, uri_str);
            }
            println!();
        }
    }

    let all_empty = output.terminals.is_empty()
        && output.tmux.is_empty()
        && output.shelldon.is_empty()
        && output.zellij.is_empty()
        && output.ides.is_empty();

    if all_empty {
        println!("No terminals or multiplexer sessions detected.");
    }
}
