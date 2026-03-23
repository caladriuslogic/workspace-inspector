use anyhow::Result;
use std::io::Write;
use std::net::TcpStream;
use std::process::Command;

use crate::types::{ShelldonInstance, ShelldonPane, ShelldonTab};

/// Discovery file written by each shelldon instance.
#[derive(serde::Deserialize)]
struct DiscoveryInfo {
    pid: u32,
    port: u16,
    auth_token: String,
    session_id: String,
}

pub fn detect() -> Result<Vec<ShelldonInstance>> {
    let entries = std::fs::read_dir("/tmp")?;
    let mut instances = Vec::new();

    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if !name.starts_with("shelldon-") || !name.ends_with(".json") {
            continue;
        }

        let contents = match std::fs::read_to_string(entry.path()) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let info: DiscoveryInfo = match serde_json::from_str(&contents) {
            Ok(i) => i,
            Err(_) => continue,
        };

        // Verify the process is still running
        if !process_alive(info.pid) {
            continue;
        }

        let tty = get_tty(info.pid);
        let panes = query_panes(&info).unwrap_or_default();

        instances.push(ShelldonInstance {
            pid: info.pid,
            port: info.port,
            session_id: info.session_id,
            tty,
            panes,
        });
    }

    Ok(instances)
}

fn process_alive(pid: u32) -> bool {
    Command::new("kill")
        .args(["-0", &pid.to_string()])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn get_tty(pid: u32) -> Option<String> {
    let output = Command::new("ps")
        .args(["-p", &pid.to_string(), "-o", "tty="])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let tty = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if tty.is_empty() || tty == "??" {
        None
    } else {
        Some(format!("/dev/{}", tty))
    }
}

fn query_panes(info: &DiscoveryInfo) -> Result<Vec<ShelldonPane>> {
    let panes_json = mcp_call(info, "list_panes", "{}")?;
    let tabs_json = mcp_call(info, "list_tabs", "{}")?;

    let panes_raw: Vec<serde_json::Value> = serde_json::from_str(&panes_json)?;
    let tabs_raw: Vec<serde_json::Value> = serde_json::from_str(&tabs_json)?;

    let mut panes = Vec::new();

    for p in &panes_raw {
        let pane_id = p.get("pane_id").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
        let name = p
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let is_focused = p.get("is_focused").and_then(|v| v.as_bool()).unwrap_or(false);

        // Find matching tabs for this pane
        let tabs: Vec<ShelldonTab> = tabs_raw
            .iter()
            .filter(|t| {
                t.get("pane_id").and_then(|v| v.as_u64()).unwrap_or(u64::MAX) == pane_id as u64
            })
            .flat_map(|t| {
                t.get("tabs")
                    .and_then(|v| v.as_array())
                    .cloned()
                    .unwrap_or_default()
            })
            .map(|t| ShelldonTab {
                tab_id: t
                    .get("tab_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                title: t
                    .get("title")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                pane_type: t
                    .get("pane_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                is_active: t.get("is_active").and_then(|v| v.as_bool()).unwrap_or(false),
            })
            .collect();

        panes.push(ShelldonPane {
            pane_id,
            name,
            is_focused,
            tabs,
        });
    }

    Ok(panes)
}

/// Make a JSON-RPC call to a shelldon MCP server over raw TCP + HTTP/1.1.
fn mcp_call(info: &DiscoveryInfo, method: &str, args: &str) -> Result<String> {
    let body = format!(
        r#"{{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{{"name":"{}","arguments":{}}}}}"#,
        method, args
    );

    let request = format!(
        "POST /mcp HTTP/1.1\r\n\
         Host: localhost:{}\r\n\
         Content-Type: application/json\r\n\
         Authorization: Bearer {}\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\
         \r\n\
         {}",
        info.port,
        info.auth_token,
        body.len(),
        body
    );

    let mut stream = TcpStream::connect(format!("127.0.0.1:{}", info.port))?;
    stream.set_read_timeout(Some(std::time::Duration::from_secs(5)))?;
    stream.write_all(request.as_bytes())?;

    let mut response = Vec::new();
    use std::io::Read;
    stream.read_to_end(&mut response)?;

    let response = String::from_utf8_lossy(&response);

    // Extract JSON body after the HTTP headers
    let body = response
        .split("\r\n\r\n")
        .nth(1)
        .unwrap_or("");

    // Parse JSON-RPC response and extract the text content
    let rpc: serde_json::Value = serde_json::from_str(body)?;
    let text = rpc
        .get("result")
        .and_then(|r| r.get("content"))
        .and_then(|c| c.as_array())
        .and_then(|a| a.first())
        .and_then(|c| c.get("text"))
        .and_then(|t| t.as_str())
        .unwrap_or("[]");

    Ok(text.to_string())
}
