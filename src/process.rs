use std::process::Command;

/// Get the current working directory of a process by PID using lsof.
pub fn get_cwd(pid: u32) -> Option<String> {
    let output = Command::new("lsof")
        .args(["-p", &pid.to_string(), "-a", "-d", "cwd", "-Fn"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if let Some(path) = line.strip_prefix('n') {
            return Some(path.to_string());
        }
    }
    None
}

/// Given a tty name (e.g. "/dev/ttys000" or "ttys000"), find the shell process and its PID.
pub fn get_shell_for_tty(tty: &str) -> Option<(u32, String)> {
    let tty_short = tty.strip_prefix("/dev/").unwrap_or(tty);

    let output = Command::new("ps")
        .args(["-t", tty_short, "-o", "pid=,comm="])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let known_shells = [
        "zsh", "bash", "fish", "sh", "tcsh", "csh", "dash", "ksh", "nu", "nushell", "elvish",
    ];

    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let mut parts = line.splitn(2, char::is_whitespace);
        let pid_str = parts.next()?;
        let comm = parts.next()?.trim();
        let basename = comm.rsplit('/').next().unwrap_or(comm);
        let basename = basename.strip_prefix('-').unwrap_or(basename);

        if known_shells.contains(&basename) {
            if let Ok(pid) = pid_str.parse::<u32>() {
                return Some((pid, basename.to_string()));
            }
        }
    }
    None
}

/// Check if a macOS GUI app is running via System Events, return its PIDs.
/// Falls back to pgrep for non-GUI processes.
pub fn find_pids_by_name(name: &str) -> Vec<u32> {
    // Try macOS System Events first — reliably finds GUI apps that pgrep misses
    let script = format!(
        r#"tell application "System Events" to get the unix id of every process whose name is "{}""#,
        name
    );
    let output = Command::new("osascript")
        .args(["-e", &script])
        .output();

    if let Ok(ref o) = output {
        if o.status.success() {
            let stdout = String::from_utf8_lossy(&o.stdout);
            let pids: Vec<u32> = stdout
                .trim()
                .split(", ")
                .filter_map(|s| s.trim().parse::<u32>().ok())
                .collect();
            if !pids.is_empty() {
                return pids;
            }
        }
    }

    // Fall back to pgrep for non-GUI processes
    let output = Command::new("pgrep")
        .args(["-x", name])
        .output();

    let output = match output {
        Ok(o) if o.status.success() => o,
        _ => return vec![],
    };

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|l| l.trim().parse::<u32>().ok())
        .collect()
}
