use serde::Serialize;
use serialport::{DataBits, FlowControl, Parity, StopBits};
use std::io::Write as _;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, State};

// ── event payloads ────────────────────────────────────────────────────────────

#[derive(Serialize, Clone)]
struct SampleEvent {
    t_ms: u64,
    values: Vec<f64>,
    labels: Option<Vec<String>>,
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

#[derive(Debug, PartialEq)]
enum LineResult {
    /// A `# label1 label2 …` header line.
    Header(Vec<String>),
    /// A line of numeric data, optionally with inline `label:value` labels.
    Data {
        values: Vec<f64>,
        /// `Some` when every token carried a `label:` prefix; `None` for bare values.
        labels: Option<Vec<String>>,
    },
}

/// Parse one trimmed, newline-stripped line.
///
/// Delimiter precedence: `,` → `\t` → ` ` (first delimiter yielding ≥1 valid token wins).
/// Tokens may be bare floats or `label:float` pairs.
/// Any token that cannot be interpreted as `f64` discards the entire line.
fn parse_line(line: &str) -> Option<LineResult> {
    let line = line.trim();
    if line.is_empty() {
        return None;
    }

    // Header line
    if let Some(rest) = line.strip_prefix('#') {
        let labels: Vec<String> = rest.split_whitespace().map(str::to_owned).collect();
        return if labels.is_empty() { None } else { Some(LineResult::Header(labels)) };
    }

    // Data line — try delimiters in precedence order
    for &delim in &[',', '\t', ' '] {
        let tokens: Vec<&str> = line
            .split(delim)
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect();
        if tokens.is_empty() {
            continue;
        }

        let mut values: Vec<f64> = Vec::with_capacity(tokens.len());
        let mut labels: Vec<String> = Vec::with_capacity(tokens.len());
        let mut has_labels = false;
        let mut all_valid = true;

        for token in &tokens {
            if let Some(colon) = token.find(':') {
                let label = &token[..colon];
                let val_str = &token[colon + 1..];
                match val_str.parse::<f64>() {
                    Ok(v) => {
                        values.push(v);
                        labels.push(label.to_owned());
                        has_labels = true;
                    }
                    Err(_) => {
                        all_valid = false;
                        break;
                    }
                }
            } else {
                match token.parse::<f64>() {
                    Ok(v) => {
                        values.push(v);
                        labels.push(String::new());
                    }
                    Err(_) => {
                        all_valid = false;
                        break;
                    }
                }
            }
        }

        if all_valid && !values.is_empty() {
            return Some(LineResult::Data {
                values,
                labels: if has_labels { Some(labels) } else { None },
            });
        }
    }

    None
}

// ── read loop ─────────────────────────────────────────────────────────────────

/// Emit `serial://raw` and, on successful parse, `serial://sample`.
fn process_line(line: &str, current_labels: &mut Vec<String>, app: &AppHandle) {
    let _ = app.emit(
        "serial://raw",
        RawEvent {
            t_ms: now_ms(),
            direction: "rx".to_string(),
            text: line.to_owned(),
        },
    );

    match parse_line(line) {
        Some(LineResult::Header(labels)) => {
            *current_labels = labels;
        }
        Some(LineResult::Data { values, labels: inline }) => {
            let event_labels = inline.or_else(|| {
                if current_labels.is_empty() {
                    None
                } else {
                    Some(current_labels.clone())
                }
            });
            let _ = app.emit(
                "serial://sample",
                SampleEvent {
                    t_ms: now_ms(),
                    values,
                    labels: event_labels,
                },
            );
        }
        None => {}
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

        loop {
            if stop_flag.load(Ordering::Relaxed) {
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
                            let line = line_buf.trim_end_matches('\r').to_string();
                            if !line.is_empty() {
                                process_line(&line, &mut current_labels, &app);
                            }
                            line_buf.clear();
                        } else if ch != '\r' {
                            line_buf.push(ch);
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
        conn.stop_flag.store(true, Ordering::Relaxed);
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
        .timeout(Duration::from_millis(50))
        .open()
        .map_err(|e| e.to_string())?;

    let read_port = port.try_clone().map_err(|e| e.to_string())?;
    let stop_flag = Arc::new(AtomicBool::new(false));

    spawn_read_loop(read_port, Arc::clone(&stop_flag), app.clone());

    {
        let mut inner = state.0.lock().unwrap();
        inner.connection = Some(ActiveConnection {
            write_port: Some(port),
            stop_flag,
        });
        inner.status = "connected".to_string();
    }

    let _ = app.emit(
        "serial://status",
        StatusEvent {
            state: "connected".to_string(),
            reason: None,
        },
    );

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

// ── mock stream (debug builds only) ──────────────────────────────────────────

#[cfg(debug_assertions)]
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

    std::thread::spawn(move || {
        let hz = rate_hz.max(0.001);
        let interval = Duration::from_secs_f64(1.0 / hz);
        let dt = 1.0 / hz;
        let mut t: f64 = 0.0;

        while !flag.load(Ordering::Relaxed) {
            let (values, labels): (Vec<f64>, Vec<&str>) = match shape.as_str() {
                "sin" => (vec![t.sin()], vec!["sin"]),
                "cos" => (vec![t.cos()], vec!["cos"]),
                "noise" => {
                    let n = ((t * 1234.567).sin() * 999.0).fract();
                    (vec![n], vec!["noise"])
                }
                "sincos" => (vec![t.sin(), t.cos()], vec!["sin", "cos"]),
                _ => {
                    // default "all": sin, cos, and a pseudo-noise signal
                    let noise = ((t * 7.31).sin() * 0.7 + (t * 3.17).cos() * 0.3) * 0.5;
                    (vec![t.sin(), t.cos(), noise], vec!["sin", "cos", "noise"])
                }
            };

            let _ = app_clone.emit(
                "serial://sample",
                SampleEvent {
                    t_ms: now_ms(),
                    values,
                    labels: Some(labels.iter().map(|s| s.to_string()).collect()),
                },
            );

            t += dt;
            std::thread::sleep(interval);
        }
    });

    {
        let mut inner = state.0.lock().unwrap();
        inner.connection = Some(ActiveConnection {
            write_port: None,
            stop_flag,
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

    Ok(())
}

// ── app entry point ───────────────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::new())
        .plugin(tauri_plugin_serialplugin::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            list_ports,
            connect,
            disconnect,
            send_line,
            connection_status,
            #[cfg(debug_assertions)]
            start_mock_stream,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── header lines ──────────────────────────────────────────────────────────

    #[test]
    fn header_basic() {
        assert_eq!(
            parse_line("# temp hum pres"),
            Some(LineResult::Header(vec![
                "temp".into(),
                "hum".into(),
                "pres".into()
            ]))
        );
    }

    #[test]
    fn header_single_label() {
        assert_eq!(
            parse_line("# voltage"),
            Some(LineResult::Header(vec!["voltage".into()]))
        );
    }

    #[test]
    fn header_empty_is_none() {
        assert_eq!(parse_line("#"), None);
        assert_eq!(parse_line("#   "), None);
    }

    // ── bare numeric values ───────────────────────────────────────────────────

    #[test]
    fn bare_comma_delimited() {
        assert_eq!(
            parse_line("1.0,2.0,3.0"),
            Some(LineResult::Data {
                values: vec![1.0, 2.0, 3.0],
                labels: None,
            })
        );
    }

    #[test]
    fn bare_tab_delimited() {
        assert_eq!(
            parse_line("1.0\t2.0\t3.0"),
            Some(LineResult::Data {
                values: vec![1.0, 2.0, 3.0],
                labels: None,
            })
        );
    }

    #[test]
    fn bare_space_delimited() {
        assert_eq!(
            parse_line("1.0 2.0 3.0"),
            Some(LineResult::Data {
                values: vec![1.0, 2.0, 3.0],
                labels: None,
            })
        );
    }

    #[test]
    fn single_value() {
        assert_eq!(
            parse_line("42.0"),
            Some(LineResult::Data {
                values: vec![42.0],
                labels: None,
            })
        );
    }

    #[test]
    fn negative_values() {
        assert_eq!(
            parse_line("-1.5,2.3,-0.0"),
            Some(LineResult::Data {
                values: vec![-1.5, 2.3, -0.0],
                labels: None,
            })
        );
    }

    #[test]
    fn scientific_notation() {
        let r = parse_line("1e3,2.5e-1");
        assert_eq!(
            r,
            Some(LineResult::Data {
                values: vec![1000.0, 0.25],
                labels: None,
            })
        );
    }

    // ── label:value pairs ─────────────────────────────────────────────────────

    #[test]
    fn label_value_comma() {
        assert_eq!(
            parse_line("temp:23.5,hum:45.2"),
            Some(LineResult::Data {
                values: vec![23.5, 45.2],
                labels: Some(vec!["temp".into(), "hum".into()]),
            })
        );
    }

    #[test]
    fn label_value_space() {
        assert_eq!(
            parse_line("x:1.0 y:2.0 z:3.0"),
            Some(LineResult::Data {
                values: vec![1.0, 2.0, 3.0],
                labels: Some(vec!["x".into(), "y".into(), "z".into()]),
            })
        );
    }

    #[test]
    fn label_value_single() {
        assert_eq!(
            parse_line("voltage:3.3"),
            Some(LineResult::Data {
                values: vec![3.3],
                labels: Some(vec!["voltage".into()]),
            })
        );
    }

    // ── invalid / dropped lines ───────────────────────────────────────────────

    #[test]
    fn non_numeric_text_is_none() {
        assert_eq!(parse_line("hello world"), None);
    }

    #[test]
    fn mixed_invalid_token_drops_line() {
        assert_eq!(parse_line("1.0,foo,3.0"), None);
    }

    #[test]
    fn empty_line_is_none() {
        assert_eq!(parse_line(""), None);
        assert_eq!(parse_line("   "), None);
    }

    #[test]
    fn bad_label_value_drops_line() {
        assert_eq!(parse_line("temp:notanumber"), None);
    }

    // ── delimiter precedence ──────────────────────────────────────────────────

    #[test]
    fn comma_takes_precedence_over_space() {
        // "1.0,2.0" — comma wins, not split by space
        assert_eq!(
            parse_line("1.0,2.0"),
            Some(LineResult::Data {
                values: vec![1.0, 2.0],
                labels: None,
            })
        );
    }

    #[test]
    fn tab_takes_precedence_over_space() {
        // "1.0\t2.0" — tab wins over space
        assert_eq!(
            parse_line("1.0\t2.0"),
            Some(LineResult::Data {
                values: vec![1.0, 2.0],
                labels: None,
            })
        );
    }

    // ── whitespace tolerance ──────────────────────────────────────────────────

    #[test]
    fn leading_trailing_whitespace_trimmed() {
        assert_eq!(
            parse_line("  1.0,2.0  "),
            Some(LineResult::Data {
                values: vec![1.0, 2.0],
                labels: None,
            })
        );
    }

    #[test]
    fn spaces_around_comma_tokens() {
        assert_eq!(
            parse_line("1.0 , 2.0 , 3.0"),
            Some(LineResult::Data {
                values: vec![1.0, 2.0, 3.0],
                labels: None,
            })
        );
    }
}
