#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Read, Write},
    net::{TcpListener, TcpStream},
    sync::{mpsc, Arc, Mutex},
    thread,
    time::Duration,
};
use tauri::{AppHandle, Emitter, Manager, State, WebviewUrl, WebviewWindowBuilder};

#[derive(Default)]
struct PendingRequests(Arc<Mutex<HashMap<String, mpsc::Sender<String>>>>);

#[derive(Clone, Serialize)]
struct HookEvent {
    request_id: Option<String>,
    session_id: Option<String>,
    hook_event_name: Option<String>,
    tool_name: Option<String>,
    tool_input: Value,
    raw: Value,
    summary: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ResolveArgs {
    request_id: String,
    decision: String,
}

#[tauri::command]
fn resolve_permission(args: ResolveArgs, pending: State<'_, PendingRequests>) -> Result<(), String> {
    let decision = match args.decision.as_str() {
        "allow" => "allow",
        "deny" => "deny",
        "ask" => "ask",
        _ => return Err("Decision must be allow, deny, or ask".into()),
    };
    let sender = pending.0.lock().map_err(|_| "Permission queue unavailable")?
        .remove(&args.request_id).ok_or_else(|| "This request is no longer waiting".to_string())?;
    sender.send(decision.to_string()).map_err(|_| "The waiting hook is no longer connected".to_string())
}

#[tauri::command]
fn hook_server_status() -> Value {
    json!({"address":"127.0.0.1:49321", "purpose":"Local-only AI agent hook bridge", "noTelemetry":true})
}

#[tauri::command]
fn open_dashboard(app: AppHandle) -> Result<(), String> {
    if let Some(win) = app.get_webview_window("dashboard") {
        win.show().map_err(|e| e.to_string())?;
        win.set_focus().map_err(|e| e.to_string())?;
        return Ok(());
    }
    WebviewWindowBuilder::new(&app, "dashboard", WebviewUrl::App("index.html".into()))
        .title("Mochi Island — Dashboard")
        .inner_size(1220.0, 820.0)
        .min_inner_size(960.0, 640.0)
        .center()
        .decorations(false)
        .transparent(true)
        .resizable(true)
        .always_on_top(false)
        .shadow(false)
        .build()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn set_always_on_top(app: AppHandle, enabled: bool) -> Result<(), String> {
    let win = app.get_webview_window("main").ok_or_else(|| "Island window not found".to_string())?;
    win.set_always_on_top(enabled).map_err(|e| e.to_string())
}

#[tauri::command]
fn hide_island(app: AppHandle) -> Result<(), String> {
    let win = app.get_webview_window("main").ok_or_else(|| "Island window not found".to_string())?;
    win.hide().map_err(|e| e.to_string())
}

#[tauri::command]
fn show_island(app: AppHandle) -> Result<(), String> {
    let win = app.get_webview_window("main").ok_or_else(|| "Island window not found".to_string())?;
    win.show().map_err(|e| e.to_string())?;
    win.set_focus().map_err(|e| e.to_string())
}

fn respond(stream: &mut TcpStream, status: &str, body: &str) {
    // The bridge is for the app and local command-line hooks, not arbitrary browser origins.
    // Intentionally omit permissive CORS headers so a random website cannot POST events
    // into the local permission/activity stream through browser JavaScript.
    let _ = write!(stream,
        "HTTP/1.1 {status}\r\nContent-Type: application/json; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.as_bytes().len(), body);
    let _ = stream.flush();
}

fn read_http(stream: &mut TcpStream) -> std::io::Result<(String, String, Vec<u8>)> {
    stream.set_read_timeout(Some(Duration::from_secs(130)))?;
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut request_line = String::new();
    reader.read_line(&mut request_line)?;
    let request_line = request_line.trim().to_string();
    let mut length = 0usize;
    loop {
        let mut line = String::new();
        let n = reader.read_line(&mut line)?;
        if n == 0 || line == "\r\n" || line == "\n" { break; }
        if let Some((name, value)) = line.split_once(':') {
            if name.eq_ignore_ascii_case("content-length") {
                length = value.trim().parse::<usize>().unwrap_or(0).min(1_000_000);
            }
        }
    }
    let mut body = vec![0u8; length];
    if length > 0 { reader.read_exact(&mut body)?; }
    Ok((request_line, String::new(), body))
}

fn handle_hook(mut stream: TcpStream, app: AppHandle, pending: Arc<Mutex<HashMap<String, mpsc::Sender<String>>>>) {
    let (request_line, _unused, body) = match read_http(&mut stream) {
        Ok(v) => v,
        Err(_) => { respond(&mut stream, "400 Bad Request", "{}"); return; }
    };
    let parts: Vec<&str> = request_line.split_whitespace().collect();
    let method = parts.first().copied().unwrap_or("");
    let path = parts.get(1).copied().unwrap_or("/");

    if method == "OPTIONS" { respond(&mut stream, "204 No Content", "{}"); return; }
    if method == "GET" && path == "/health" {
        respond(&mut stream, "200 OK", r#"{"ok":true,"service":"Mochi Island local hook bridge"}"#);
        return;
    }
    if method != "POST" || (path != "/hook" && path != "/event") {
        respond(&mut stream, "404 Not Found", r#"{"error":"Use POST /hook or POST /event"}"#);
        return;
    }

    let payload: Value = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(_) => { respond(&mut stream, "400 Bad Request", r#"{"error":"Invalid JSON"}"#); return; }
    };
    let event_name = payload.get("hook_event_name").and_then(Value::as_str)
        .or_else(|| payload.get("hookEventName").and_then(Value::as_str)).unwrap_or("AgentEvent").to_string();
    let session_id = payload.get("session_id").and_then(Value::as_str)
        .or_else(|| payload.get("sessionId").and_then(Value::as_str)).map(str::to_string);
    let tool_name = payload.get("tool_name").and_then(Value::as_str)
        .or_else(|| payload.get("toolName").and_then(Value::as_str)).map(str::to_string);
    let tool_input = payload.get("tool_input").or_else(|| payload.get("toolInput")).cloned().unwrap_or(Value::Null);
    let summary = if let Some(name) = tool_name.as_deref() {
        format!("{event_name} · {name}")
    } else {
        event_name.clone()
    };

    if event_name == "PreToolUse" {
        let request_id = uuid::Uuid::new_v4().to_string();
        let (tx, rx) = mpsc::channel::<String>();
        if let Ok(mut queue) = pending.lock() { queue.insert(request_id.clone(), tx); }
        let envelope = HookEvent {
            request_id: Some(request_id.clone()), session_id, hook_event_name: Some(event_name),
            tool_name, tool_input, raw: payload, summary,
        };
        if app.emit("permission-request", &envelope).is_err() {
            if let Ok(mut queue) = pending.lock() { queue.remove(&request_id); }
            respond(&mut stream, "200 OK", "{}");
            return;
        }
        match rx.recv_timeout(Duration::from_secs(120)) {
            Ok(decision) if decision == "allow" => {
                let output = json!({"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"allow","permissionDecisionReason":"Approved from Mochi Island"}}).to_string();
                respond(&mut stream, "200 OK", &output);
            }
            Ok(decision) if decision == "ask" => {
                let output = json!({"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"ask","permissionDecisionReason":"Return to the agent for approval"}}).to_string();
                respond(&mut stream, "200 OK", &output);
            }
            Ok(_) => {
                let output = json!({"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"deny","permissionDecisionReason":"Denied from Mochi Island"}}).to_string();
                respond(&mut stream, "200 OK", &output);
            }
            Err(_) => {
                // Fail safely: a missing UI decision must never become an automatic approval.
                let output = json!({"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"deny","permissionDecisionReason":"No decision was received from Mochi Island before timeout"}}).to_string();
                respond(&mut stream, "200 OK", &output);
            }
        }
    } else {
        let envelope = HookEvent {
            request_id: None, session_id, hook_event_name: Some(event_name), tool_name,
            tool_input, raw: payload, summary,
        };
        let _ = app.emit("agent-event", &envelope);
        respond(&mut stream, "200 OK", "{}");
    }
}

fn start_hook_server(app: AppHandle, pending: Arc<Mutex<HashMap<String, mpsc::Sender<String>>>>) {
    thread::spawn(move || {
        let listener = match TcpListener::bind("127.0.0.1:49321") {
            Ok(listener) => listener,
            Err(err) => { let _ = app.emit("hook-server-error", err.to_string()); return; }
        };
        for stream in listener.incoming() {
            if let Ok(stream) = stream {
                let handle = app.clone();
                let queue = pending.clone();
                thread::spawn(move || handle_hook(stream, handle, queue));
            }
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(PendingRequests::default())
        .invoke_handler(tauri::generate_handler![resolve_permission, hook_server_status, open_dashboard, hide_island, show_island, set_always_on_top])
        .setup(|app| {
            let app_handle = app.handle().clone();
            let queue = app.state::<PendingRequests>().0.clone();
            start_hook_server(app_handle, queue);
            if let Some(window) = app.get_webview_window("main") {
                if let Ok(Some(monitor)) = window.current_monitor() {
                    let size = monitor.size();
                    let scale = monitor.scale_factor();
                    let logical_width = 1020.0 * scale;
                    let x = ((size.width as f64 - logical_width) / 2.0).max(0.0) as i32;
                    let _ = window.set_position(tauri::PhysicalPosition::new(x, (12.0 * scale) as i32));
                }
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Mochi Island");
}

fn main() {
    run();
}
