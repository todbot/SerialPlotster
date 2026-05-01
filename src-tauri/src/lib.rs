use serde::Serialize;
use serialport::{DataBits, FlowControl, Parity, StopBits};
use std::io::Write as _;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::menu::{AboutMetadata, CheckMenuItem, MenuBuilder, MenuItem, SubmenuBuilder};
use tauri::{AppHandle, Emitter, State};
#[cfg(debug_assertions)] use tauri::Manager;

// ── event payloads ────────────────────────────────────────────────────────────

#[derive(Serialize, Clone)]
struct SampleEvent {
    t_ms: u64,
    values: Vec<f64>,
    labels: Option<Vec<String>>,
}

#[derive(Serialize, Clone)]
struct SampleBatch {
    samples: Vec<SampleEvent>,
}

#[derive(Serialize, Clone)]
struct RawEvent {
    t_ms: u64,
    direction: String,
    text: String,
}

#[derive(Serialize, Clone)]
struct StatusEvent {
    state: String,
    reason: Option<String>,
}

// ── app state ─────────────────────────────────────────────────────────────────

struct ActiveConnection {
    /// None for mock stream (no real port to write to).
    write_port: Option<Box<dyn serialport::SerialPort>>,
    stop_flag: Arc<AtomicBool>,
}

struct AppStateInner {
    connection: Option<ActiveConnection>,
    status: String, // "connected" | "disconnected" | "error"
}

pub struct AppState(Mutex<AppStateInner>);

impl AppState {
    fn new() -> Self {
        AppState(Mutex::new(AppStateInner {
            connection: None,
            status: "disconnected".to_string(),
        }))
    }
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

// ── parser ────────────────────────────────────────────────────────────────────

mod parser;
use parser::{parse_line, LineResult};

// ── read loop ─────────────────────────────────────────────────────────────────

/// Token-bucket throttle for raw console events.
/// Refills at `rate_per_sec` tokens/second; one token consumed per allowed event.
struct RawThrottle {
    tokens: f64,
    last_ms: u64,
    rate_per_sec: f64,
}

impl RawThrottle {
    fn new(rate_per_sec: f64) -> Self {
        Self { tokens: rate_per_sec, last_ms: 0, rate_per_sec }
    }

    fn allow(&mut self, now_ms: u64) -> bool {
        if now_ms > self.last_ms {
            let elapsed = (now_ms - self.last_ms) as f64 / 1000.0;
            self.tokens = (self.tokens + elapsed * self.rate_per_sec).min(self.rate_per_sec);
            self.last_ms = now_ms;
        }
        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }
}

/// Parse one line, optionally emit `serial://raw`, and return a `SampleEvent`
/// if the line contains numeric data.  Emits `serial://gap` immediately on a
/// mid-stream header (can't be deferred to a batch).
///
/// `t_ms` is supplied by the caller for monotonic per-line timestamps.
/// `emit_raw` is false when the raw token bucket is exhausted.
fn process_line(
    line: &str,
    t_ms: u64,
    current_labels: &mut Vec<String>,
    app: &AppHandle,
    emit_raw: bool,
) -> Option<SampleEvent> {
    if emit_raw {
        let _ = app.emit(
            "serial://raw",
            RawEvent { t_ms, direction: "rx".to_string(), text: line.to_owned() },
        );
    }

    match parse_line(line) {
        Some(LineResult::Header(labels)) => {
            // A header mid-stream means the device restarted or re-identified its
            // channels.  Insert a gap so the chart shows a clean break, then
            // replace the labels so subsequent data uses the new names.
            if !current_labels.is_empty() {
                let _ = app.emit("serial://gap", ());
            }
            *current_labels = labels;
            None
        }
        Some(LineResult::Data { values, labels: inline }) => {
            let event_labels = inline.or_else(|| {
                if current_labels.is_empty() { None } else { Some(current_labels.clone()) }
            });
            Some(SampleEvent { t_ms, values, labels: event_labels })
        }
        None => None,
    }
}

/// Spawn a background thread that reads lines from `port` and emits events.
///
/// Exits when `stop_flag` is set or the port returns an error/EOF.
/// On an unexpected error/EOF it emits `serial://status { disconnected }`.
/// It does NOT emit on intentional stop (caller is responsible).
fn spawn_read_loop(
    mut port: Box<dyn serialport::SerialPort>,
    stop_flag: Arc<AtomicBool>,
    app: AppHandle,
) {
    std::thread::spawn(move || {
        let mut buf = [0u8; 4096];
        let mut line_buf = String::new();
        let mut current_labels: Vec<String> = Vec::new();
        // Discard the first partial line so we only ever process complete lines.
        // Connecting mid-transmission produces a tail fragment (e.g. "234\n") that
        // parses as a garbage value and throws off Y-axis auto-scale.
        let mut skip_first_line = true;
        // Monotonic timestamp: each parsed line gets at least 1 ms more than the
        // previous one so that lines arriving in the same OS read() batch (e.g.
        // several USB-buffered samples) are spread out on the time axis rather
        // than all collapsing to a single point.
        let mut last_t_ms: u64 = 0;
        // Batch buffer: collect all samples from one read() call, then emit as a
        // single serial://sample-batch event.  Reduces IPC crossings from
        // O(samples/sec) to O(read-calls/sec), which matters at high data rates.
        let mut batch: Vec<SampleEvent> = Vec::new();
        // Raw event throttle: at high data rates the console can't display every
        // line anyway, so cap raw events at 100/sec to avoid flooding IPC.
        let mut raw_throttle = RawThrottle::new(100.0);

        loop {
            if stop_flag.load(Ordering::SeqCst) {
                break;
            }

            match port.read(&mut buf) {
                Ok(0) => {
                    if !stop_flag.load(Ordering::Relaxed) {
                        let _ = app.emit(
                            "serial://status",
                            StatusEvent {
                                state: "disconnected".to_string(),
                                reason: Some("connection closed".to_string()),
                            },
                        );
                    }
                    break;
                }
                Ok(n) => {
                    let chunk = String::from_utf8_lossy(&buf[..n]);
                    for ch in chunk.chars() {
                        if ch == '\n' {
                            if skip_first_line {
                                skip_first_line = false;
                                line_buf.clear();
                            } else {
                                let line = line_buf.trim_end_matches('\r').to_string();
                                if !line.is_empty() {
                                    // Ensure each line gets a strictly increasing timestamp
                                    // even when multiple lines arrive in one read() call.
                                    let real_now = now_ms();
                                    let t = real_now.max(last_t_ms + 1);
                                    last_t_ms = t;
                                    let emit_raw = raw_throttle.allow(real_now);
                                    if let Some(s) = process_line(&line, t, &mut current_labels, &app, emit_raw) {
                                        batch.push(s);
                                    }
                                }
                                line_buf.clear();
                            }
                        } else if ch != '\r' {
                            line_buf.push(ch);
                        }
                    }
                    // Emit all samples from this read() call as a single batch event.
                    // Re-check stop_flag before emitting: if disconnect was called while
                    // we were processing this buffer, drop the batch rather than racing
                    // the status event to the frontend.
                    if !batch.is_empty() {
                        if stop_flag.load(Ordering::SeqCst) {
                            batch.clear();
                        } else {
                            let _ = app.emit(
                                "serial://sample-batch",
                                SampleBatch { samples: std::mem::take(&mut batch) },
                            );
                        }
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
                    // Normal: no data arrived within the timeout window.
                }
                Err(e) => {
                    if !stop_flag.load(Ordering::Relaxed) {
                        let _ = app.emit(
                            "serial://status",
                            StatusEvent {
                                state: "disconnected".to_string(),
                                reason: Some(e.to_string()),
                            },
                        );
                    }
                    break;
                }
            }
        }
    });
}

// ── internal disconnect (no event emitted) ────────────────────────────────────

fn stop_connection(state: &AppState) {
    let old = {
        let mut inner = state.0.lock().unwrap();
        inner.status = "disconnected".to_string();
        inner.connection.take()
    };
    if let Some(conn) = old {
        conn.stop_flag.store(true, Ordering::SeqCst);
        drop(conn.write_port);
    }
}

// ── commands ──────────────────────────────────────────────────────────────────

#[tauri::command]
fn list_ports() -> Result<Vec<String>, String> {
    serialport::available_ports()
        .map(|ports| ports.into_iter().map(|p| p.port_name).collect())
        .map_err(|e| e.to_string())
}

#[allow(clippy::too_many_arguments)]
#[tauri::command]
fn connect(
    path: String,
    baud: u32,
    data_bits: u8,
    parity: String,
    stop_bits: u8,
    flow_control: String,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<(), String> {
    // Stop any existing connection before opening a new one.
    stop_connection(&state);

    let db = match data_bits {
        5 => DataBits::Five,
        6 => DataBits::Six,
        7 => DataBits::Seven,
        8 => DataBits::Eight,
        _ => return Err(format!("invalid data_bits: {data_bits}")),
    };
    let par = match parity.as_str() {
        "none" => Parity::None,
        "odd" => Parity::Odd,
        "even" => Parity::Even,
        _ => return Err(format!("invalid parity: {parity}")),
    };
    let sb = match stop_bits {
        1 => StopBits::One,
        2 => StopBits::Two,
        _ => return Err(format!("invalid stop_bits: {stop_bits}")),
    };
    let fc = match flow_control.as_str() {
        "none" => FlowControl::None,
        "software" => FlowControl::Software,
        "hardware" => FlowControl::Hardware,
        _ => return Err(format!("invalid flow_control: {flow_control}")),
    };

    let port = serialport::new(&path, baud)
        .data_bits(db)
        .parity(par)
        .stop_bits(sb)
        .flow_control(fc)
        .timeout(Duration::from_millis(100))
        .open()
        .map_err(|e| e.to_string())?;

    let read_port = port.try_clone().map_err(|e| e.to_string())?;
    let stop_flag = Arc::new(AtomicBool::new(false));

    // Emit connected status and record state BEFORE spawning the read loop so
    // the frontend's acceptingSamples gate is open before any sample events arrive.
    let _ = app.emit(
        "serial://status",
        StatusEvent {
            state: "connected".to_string(),
            reason: None,
        },
    );

    {
        let mut inner = state.0.lock().unwrap();
        inner.connection = Some(ActiveConnection {
            write_port: Some(port),
            stop_flag: Arc::clone(&stop_flag),
        });
        inner.status = "connected".to_string();
    }

    spawn_read_loop(read_port, stop_flag, app.clone());

    Ok(())
}

#[tauri::command]
fn disconnect(state: State<'_, AppState>, app: AppHandle) -> Result<(), String> {
    stop_connection(&state);
    let _ = app.emit(
        "serial://status",
        StatusEvent {
            state: "disconnected".to_string(),
            reason: None,
        },
    );
    Ok(())
}

#[tauri::command]
fn send_line(
    text: String,
    line_ending: Option<String>,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<(), String> {
    let ending = line_ending.as_deref().unwrap_or("\n");
    let payload = format!("{text}{ending}");

    {
        let mut inner = state.0.lock().unwrap();
        let conn = inner.connection.as_mut().ok_or("not connected")?;
        let port = conn
            .write_port
            .as_mut()
            .ok_or("cannot send to mock stream")?;
        port.write_all(payload.as_bytes()).map_err(|e| e.to_string())?;
    }

    let _ = app.emit(
        "serial://raw",
        RawEvent {
            t_ms: now_ms(),
            direction: "tx".to_string(),
            text,
        },
    );

    Ok(())
}

#[tauri::command]
fn connection_status(state: State<'_, AppState>) -> String {
    state.0.lock().unwrap().status.clone()
}

// ── mock stream ───────────────────────────────────────────────────────────────

#[tauri::command]
fn start_mock_stream(
    rate_hz: f64,
    shape: String,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<(), String> {
    stop_connection(&state);

    let stop_flag = Arc::new(AtomicBool::new(false));
    let flag = Arc::clone(&stop_flag);
    let app_clone = app.clone();

    {
        let mut inner = state.0.lock().unwrap();
        inner.connection = Some(ActiveConnection {
            write_port: None,
            stop_flag: Arc::clone(&stop_flag),
        });
        inner.status = "connected".to_string();
    }
    let _ = app.emit(
        "serial://status",
        StatusEvent {
            state: "connected".to_string(),
            reason: Some("mock stream".to_string()),
        },
    );

    std::thread::spawn(move || {
        let hz = rate_hz.max(0.001);
        let interval = Duration::from_secs_f64(1.0 / hz);
        let dt = 1.0 / hz;
        let mut t: f64 = 0.0;

        while !flag.load(Ordering::Relaxed) {
            let (values, labels): (Vec<f64>, Vec<&str>) = match shape.as_str() {
                "sin" => (vec![t.sin()], vec!["sin"]),
                "cos" => (vec![t.cos()], vec!["cos"]),
                "tri" => {
                    let tri = 0.75 * (2.0 / std::f64::consts::PI) * (2.0 * t + std::f64::consts::FRAC_PI_2).sin().asin();
                    (vec![tri], vec!["tri"])
                }
                "noise" => {
                    let n = (t * 7.31).sin() * 0.7 + (t * 3.17).cos() * 0.3;
                    (vec![n], vec!["noise"])
                }
                "sincos" => (vec![t.sin(), t.cos()], vec!["sin", "cos"]),
                _ => {
                    // default "all": sin, cos, triangle, and a pseudo-noise signal
                    let tri = 0.75 * (2.0 / std::f64::consts::PI) * (2.0 * t + std::f64::consts::FRAC_PI_2).sin().asin();
                    let noise = ((t * 7.31).sin() * 0.7 + (t * 3.17).cos() * 0.3) * 0.5;
                    (vec![t.sin(), t.cos(), tri, noise], vec!["sin", "cos", "tri", "noise"])
                }
            };

            let t_ms = now_ms();
            let text = labels.iter().zip(values.iter())
                .map(|(l, v)| format!("{l}:{v:.4}"))
                .collect::<Vec<_>>()
                .join(",");
            let _ = app_clone.emit(
                "serial://raw",
                RawEvent { t_ms, direction: "rx".to_string(), text },
            );
            let _ = app_clone.emit(
                "serial://sample",
                SampleEvent {
                    t_ms,
                    values,
                    labels: Some(labels.iter().map(|s| s.to_string()).collect()),
                },
            );

            t += dt;
            std::thread::sleep(interval);
        }
    });

    Ok(())
}

// ── app entry point ───────────────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::new())
        .plugin(tauri_plugin_serialplugin::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            #[cfg(debug_assertions)]
            {
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
            }
            let about = AboutMetadata {
                name:      Some("SerialPlotster".into()),
                version:   Some(env!("CARGO_PKG_VERSION").into()),
                authors:   Some(vec!["Tod Kurt".into()]),
                comments:  Some("A cross-platform serial plotter for Arduino and other devices.".into()),
                copyright: Some("© 2026 Tod Kurt".into()),
                // Renders as a clickable link on Linux (GTK). On macOS the link
                // comes from Credits.html in the app bundle (see tauri.conf.json).
                website:       Some("https://github.com/todbot/SerialPlotster".into()),
                website_label: Some("github.com/todbot/SerialPlotster".into()),
                ..Default::default()
            };

            let edit_menu = SubmenuBuilder::new(app, "Edit")
                .undo()
                .redo()
                .separator()
                .cut()
                .copy()
                .paste()
                .select_all()
                .build()?;

            let connect_item = MenuItem::with_id(app, "toggle-connect", "Connect / Disconnect", true, Some("CmdOrCtrl+R"))?;
            let chart_item   = MenuItem::with_id(app, "chart-tab",      "Chart",               true, Some("CmdOrCtrl+1"))?;
            let console_item = MenuItem::with_id(app, "console-tab",    "Console",             true, Some("CmdOrCtrl+2"))?;
            let pause_item   = MenuItem::with_id(app, "toggle-pause",   "Pause / Resume",      true, Some("Space"))?;
            let live_item    = MenuItem::with_id(app, "back-to-live",   "Back to Live",        true, Some("Escape"))?;
            let clear_item   = MenuItem::with_id(app, "clear-console",  "Clear Console",       true, Some("CmdOrCtrl+L"))?;

            let mock_toggle_item = CheckMenuItem::with_id(
                app, "toggle-mock", "Mock Controls", true, false, None::<&str>,
            )?;

            let view_menu = SubmenuBuilder::new(app, "View")
                .item(&chart_item)
                .item(&console_item)
                .separator()
                .item(&pause_item)
                .item(&live_item)
                .separator()
                .item(&clear_item)
                .separator()
                .item(&mock_toggle_item)
                .build()?;

            // macOS: first submenu becomes the app menu (shown as the app name).
            // Hide/HideOthers/ShowAll are macOS-only conventions.
            #[cfg(target_os = "macos")]
            let menu = {
                let app_menu = SubmenuBuilder::new(app, "SerialPlotster")
                    .about(Some(about))
                    .separator()
                    .item(&connect_item)
                    .separator()
                    .hide()
                    .hide_others()
                    .show_all()
                    .separator()
                    .quit()
                    .build()?;
                MenuBuilder::new(app)
                    .item(&app_menu)
                    .item(&edit_menu)
                    .item(&view_menu)
                    .build()?
            };

            // Windows / Linux: standard File menu with About + Quit.
            #[cfg(not(target_os = "macos"))]
            let menu = {
                let file_menu = SubmenuBuilder::new(app, "File")
                    .item(&connect_item)
                    .separator()
                    .about(Some(about))
                    .separator()
                    .quit()
                    .build()?;
                MenuBuilder::new(app)
                    .item(&file_menu)
                    .item(&edit_menu)
                    .item(&view_menu)
                    .build()?
            };

            app.set_menu(menu)?;

            app.on_menu_event(|app, event| {
                let action = match event.id().as_ref() {
                    "chart-tab"      => "chart-tab",
                    "console-tab"    => "console-tab",
                    "toggle-connect" => "toggle-connect",
                    "toggle-pause"   => "toggle-pause",
                    "back-to-live"   => "back-to-live",
                    "clear-console"  => "clear-console",
                    "toggle-mock"    => "toggle-mock",
                    _ => return,
                };
                let _ = app.emit("menu://action", action);
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_ports,
            connect,
            disconnect,
            send_line,
            connection_status,
            start_mock_stream,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests;
