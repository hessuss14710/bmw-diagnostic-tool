//! WebSocket Server for Web Interface Communication
//!
//! Provides a WebSocket API for the web dashboard to communicate
//! with the FTDI daemon.

use crate::ftdi::{self, FtdiConnection};
use crate::kline::{self, KLine};

use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio_tungstenite::{accept_async, tungstenite::Message};
use tracing::{debug, error, info, warn};

/// Maximum concurrent WebSocket connections
const MAX_CONNECTIONS: usize = 5;

/// Rate limit: maximum commands per second per connection
const MAX_COMMANDS_PER_SECOND: usize = 20;

/// Global connection counter
static ACTIVE_CONNECTIONS: AtomicUsize = AtomicUsize::new(0);

/// Global state shared between connections
struct AppState {
    kline: Option<KLine>,
    connected_device: Option<String>,
}

/// WebSocket command from client
#[derive(Debug, Deserialize)]
#[serde(tag = "cmd", content = "data")]
enum WsCommand {
    #[serde(rename = "list_devices")]
    ListDevices,

    #[serde(rename = "connect")]
    Connect { device_index: i32 },

    #[serde(rename = "disconnect")]
    Disconnect,

    #[serde(rename = "init_ecu")]
    InitEcu { address: u8, fast: bool },

    #[serde(rename = "read_dtcs")]
    ReadDtcs,

    #[serde(rename = "clear_dtcs")]
    ClearDtcs,

    #[serde(rename = "read_pid")]
    ReadPid { pid: u8 },

    #[serde(rename = "read_pids")]
    ReadPids { pids: Vec<u8> },

    /// Read BMW manufacturer-specific PID (Service 0x21)
    #[serde(rename = "read_bmw_pid")]
    ReadBmwPid { pid: u8 },

    /// Read multiple BMW manufacturer PIDs
    #[serde(rename = "read_bmw_pids")]
    ReadBmwPids { pids: Vec<u8> },

    /// Read comprehensive engine data (DME)
    #[serde(rename = "read_engine_data")]
    ReadEngineData,

    /// Read comprehensive transmission data (EGS)
    #[serde(rename = "read_transmission_data")]
    ReadTransmissionData,

    #[serde(rename = "tester_present")]
    TesterPresent,

    #[serde(rename = "status")]
    Status,
}

/// WebSocket response to client
#[derive(Debug, Serialize)]
struct WsResponse {
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    latency_us: Option<u64>,
}

impl WsResponse {
    fn success(data: serde_json::Value) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            latency_us: None,
        }
    }

    fn success_with_latency(data: serde_json::Value, latency_us: u64) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            latency_us: Some(latency_us),
        }
    }

    fn error(msg: &str) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(msg.to_string()),
            latency_us: None,
        }
    }
}

/// Run the WebSocket server
pub async fn run_server(port: u16) -> Result<()> {
    let addr = format!("127.0.0.1:{}", port);
    let listener = TcpListener::bind(&addr).await?;

    info!("WebSocket server listening on ws://{}", addr);
    println!();
    println!("╔═══════════════════════════════════════════════════════╗");
    println!("║  WebSocket server ready!                              ║");
    println!("║                                                       ║");
    println!("║  Connect from browser: ws://localhost:{}           ║", port);
    println!("║                                                       ║");
    println!("║  Commands available:                                  ║");
    println!("║    - list_devices: List FTDI devices                  ║");
    println!("║    - connect: Connect to device                       ║");
    println!("║    - init_ecu: Initialize K-Line to ECU               ║");
    println!("║    - read_dtcs: Read diagnostic trouble codes         ║");
    println!("║    - clear_dtcs: Clear all DTCs                       ║");
    println!("║    - read_pid: Read single PID value                  ║");
    println!("║    - read_pids: Read multiple PIDs                    ║");
    println!("╚═══════════════════════════════════════════════════════╝");
    println!();

    let state = Arc::new(Mutex::new(AppState {
        kline: None,
        connected_device: None,
    }));

    while let Ok((stream, addr)) = listener.accept().await {
        // Check connection limit
        let current = ACTIVE_CONNECTIONS.load(Ordering::SeqCst);
        if current >= MAX_CONNECTIONS {
            warn!("Connection rejected from {}: max connections ({}) reached", addr, MAX_CONNECTIONS);
            drop(stream); // Close the connection
            continue;
        }

        // Increment connection counter
        ACTIVE_CONNECTIONS.fetch_add(1, Ordering::SeqCst);
        info!("New connection from: {} (active: {})", addr, current + 1);

        let state = Arc::clone(&state);
        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, state).await {
                error!("Connection error: {}", e);
            }
            // Decrement connection counter when done
            let remaining = ACTIVE_CONNECTIONS.fetch_sub(1, Ordering::SeqCst) - 1;
            info!("Connection closed (active: {})", remaining);
        });
    }

    Ok(())
}

async fn handle_connection(stream: TcpStream, state: Arc<Mutex<AppState>>) -> Result<()> {
    let ws_stream = accept_async(stream).await?;
    let (mut write, mut read) = ws_stream.split();

    // Send welcome message
    let welcome = WsResponse::success(serde_json::json!({
        "message": "BMW Diagnostic Daemon v1.0",
        "precision": "microsecond",
        "protocol": "KWP2000",
        "limits": {
            "max_connections": MAX_CONNECTIONS,
            "max_commands_per_second": MAX_COMMANDS_PER_SECOND
        }
    }));
    write
        .send(Message::Text(serde_json::to_string(&welcome)?))
        .await?;

    // Rate limiting state
    let mut command_count = 0usize;
    let mut rate_limit_start = Instant::now();

    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                // Rate limiting check
                let elapsed = rate_limit_start.elapsed().as_secs_f64();
                if elapsed >= 1.0 {
                    // Reset counter every second
                    command_count = 0;
                    rate_limit_start = Instant::now();
                }

                command_count += 1;
                if command_count > MAX_COMMANDS_PER_SECOND {
                    warn!("Rate limit exceeded: {} commands/sec", command_count);
                    let response = WsResponse::error("Rate limit exceeded. Max 20 commands/second.");
                    let json = serde_json::to_string(&response)?;
                    write.send(Message::Text(json)).await?;
                    continue;
                }

                debug!("Received: {}", text);

                let response = match serde_json::from_str::<WsCommand>(&text) {
                    Ok(cmd) => process_command(cmd, &state).await,
                    Err(e) => WsResponse::error(&format!("Invalid command: {}", e)),
                };

                let json = serde_json::to_string(&response)?;
                debug!("Sending: {}", json);
                write.send(Message::Text(json)).await?;
            }
            Ok(Message::Close(_)) => {
                info!("Client disconnected");
                break;
            }
            Ok(Message::Ping(data)) => {
                write.send(Message::Pong(data)).await?;
            }
            Err(e) => {
                error!("WebSocket error: {}", e);
                break;
            }
            _ => {}
        }
    }

    Ok(())
}

async fn process_command(cmd: WsCommand, state: &Arc<Mutex<AppState>>) -> WsResponse {
    let start = Instant::now();

    match cmd {
        WsCommand::ListDevices => {
            match ftdi::list_devices() {
                Ok(devices) => {
                    let device_list: Vec<_> = devices
                        .iter()
                        .map(|d| {
                            serde_json::json!({
                                "index": d.index,
                                "description": d.description,
                                "serial": d.serial_number
                            })
                        })
                        .collect();

                    WsResponse::success(serde_json::json!({ "devices": device_list }))
                }
                Err(e) => WsResponse::error(&format!("Failed to list devices: {}", e)),
            }
        }

        WsCommand::Connect { device_index } => {
            // Validate device index
            if device_index < 0 {
                return WsResponse::error("Invalid device index: must be >= 0");
            }

            let mut state = state.lock().await;

            // Disconnect existing connection
            state.kline = None;
            state.connected_device = None;

            match FtdiConnection::open(device_index) {
                Ok(ftdi) => {
                    let kline = KLine::new(ftdi);
                    state.kline = Some(kline);
                    state.connected_device = Some(format!("Device {}", device_index));

                    let latency = start.elapsed().as_micros() as u64;
                    WsResponse::success_with_latency(
                        serde_json::json!({
                            "connected": true,
                            "device": device_index
                        }),
                        latency,
                    )
                }
                Err(e) => WsResponse::error(&format!("Failed to connect: {}", e)),
            }
        }

        WsCommand::Disconnect => {
            let mut state = state.lock().await;
            state.kline = None;
            state.connected_device = None;
            WsResponse::success(serde_json::json!({ "disconnected": true }))
        }

        WsCommand::InitEcu { address, fast } => {
            let mut state = state.lock().await;

            if let Some(ref mut kline) = state.kline {
                let result = if fast {
                    kline.init_fast(address)
                } else {
                    kline.init_5baud(address)
                };

                match result {
                    Ok(init_result) => {
                        let latency = start.elapsed().as_micros() as u64;
                        WsResponse::success_with_latency(
                            serde_json::json!({
                                "initialized": init_result.success,
                                "key_bytes": init_result.key_bytes.map(|kb| format!("{:02X} {:02X}", kb[0], kb[1])),
                                "p2_max_ms": init_result.timing_p2_max,
                                "p3_min_ms": init_result.timing_p3_min
                            }),
                            latency,
                        )
                    }
                    Err(e) => WsResponse::error(&format!("Init failed: {}", e)),
                }
            } else {
                WsResponse::error("Not connected to FTDI device")
            }
        }

        WsCommand::ReadDtcs => {
            let mut state = state.lock().await;

            if let Some(ref mut kline) = state.kline {
                match kline.read_dtcs() {
                    Ok(dtcs) => {
                        let latency = start.elapsed().as_micros() as u64;
                        let dtc_list: Vec<_> = dtcs
                            .iter()
                            .map(|(code, status)| {
                                let decoded = kline::decode_dtc(*code);
                                let status_flags = kline::DtcStatus::from(*status);
                                serde_json::json!({
                                    "code": decoded,
                                    "raw": format!("{:04X}", code),
                                    "status": status,
                                    "confirmed": status_flags.confirmed,
                                    "pending": status_flags.pending,
                                    "test_failed": status_flags.test_failed
                                })
                            })
                            .collect();

                        WsResponse::success_with_latency(
                            serde_json::json!({
                                "count": dtcs.len(),
                                "dtcs": dtc_list
                            }),
                            latency,
                        )
                    }
                    Err(e) => WsResponse::error(&format!("Read DTCs failed: {}", e)),
                }
            } else {
                WsResponse::error("Not connected")
            }
        }

        WsCommand::ClearDtcs => {
            let mut state = state.lock().await;

            if let Some(ref mut kline) = state.kline {
                match kline.clear_dtcs() {
                    Ok(success) => {
                        let latency = start.elapsed().as_micros() as u64;
                        WsResponse::success_with_latency(
                            serde_json::json!({ "cleared": success }),
                            latency,
                        )
                    }
                    Err(e) => WsResponse::error(&format!("Clear DTCs failed: {}", e)),
                }
            } else {
                WsResponse::error("Not connected")
            }
        }

        WsCommand::ReadPid { pid } => {
            let mut state = state.lock().await;

            if let Some(ref mut kline) = state.kline {
                match kline.read_pid(pid) {
                    Ok(data) => {
                        let latency = start.elapsed().as_micros() as u64;

                        // Calculate value based on PID
                        let value = calculate_pid_value(pid, &data);

                        WsResponse::success_with_latency(
                            serde_json::json!({
                                "pid": format!("0x{:02X}", pid),
                                "raw": data.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" "),
                                "value": value.0,
                                "unit": value.1
                            }),
                            latency,
                        )
                    }
                    Err(e) => WsResponse::error(&format!("Read PID failed: {}", e)),
                }
            } else {
                WsResponse::error("Not connected")
            }
        }

        WsCommand::ReadPids { pids: pid_list } => {
            // Limit number of PIDs to prevent DoS (each read takes ~50-100ms)
            const MAX_PIDS: usize = 20;
            if pid_list.len() > MAX_PIDS {
                return WsResponse::error(&format!(
                    "Too many PIDs requested: {} (max {})",
                    pid_list.len(),
                    MAX_PIDS
                ));
            }

            let mut state = state.lock().await;

            if let Some(ref mut kline) = state.kline {
                let mut results = HashMap::new();
                let mut total_latency = 0u64;

                for pid in pid_list {
                    let pid_start = Instant::now();
                    match kline.read_pid(pid) {
                        Ok(data) => {
                            let latency = pid_start.elapsed().as_micros() as u64;
                            total_latency += latency;

                            let value = calculate_pid_value(pid, &data);
                            results.insert(
                                format!("0x{:02X}", pid),
                                serde_json::json!({
                                    "value": value.0,
                                    "unit": value.1,
                                    "latency_us": latency
                                }),
                            );
                        }
                        Err(e) => {
                            results.insert(
                                format!("0x{:02X}", pid),
                                serde_json::json!({ "error": format!("{}", e) }),
                            );
                        }
                    }
                }

                WsResponse::success_with_latency(
                    serde_json::json!({
                        "pids": results,
                        "total_latency_us": total_latency
                    }),
                    total_latency,
                )
            } else {
                WsResponse::error("Not connected")
            }
        }

        WsCommand::TesterPresent => {
            let mut state = state.lock().await;

            if let Some(ref mut kline) = state.kline {
                match kline.tester_present() {
                    Ok(success) => {
                        let latency = start.elapsed().as_micros() as u64;
                        WsResponse::success_with_latency(
                            serde_json::json!({ "alive": success }),
                            latency,
                        )
                    }
                    Err(e) => WsResponse::error(&format!("TesterPresent failed: {}", e)),
                }
            } else {
                WsResponse::error("Not connected")
            }
        }

        WsCommand::Status => {
            let state = state.lock().await;
            let connected = state.kline.is_some();
            let initialized = state
                .kline
                .as_ref()
                .map(|k| k.is_initialized())
                .unwrap_or(false);

            WsResponse::success(serde_json::json!({
                "connected": connected,
                "initialized": initialized,
                "device": state.connected_device
            }))
        }

        WsCommand::ReadBmwPid { pid } => {
            let mut state = state.lock().await;

            if let Some(ref mut kline) = state.kline {
                match kline.read_manufacturer_pid(pid) {
                    Ok(data) => {
                        let latency = start.elapsed().as_micros() as u64;
                        let value = calculate_bmw_pid_value(pid, &data);

                        WsResponse::success_with_latency(
                            serde_json::json!({
                                "pid": format!("0x{:02X}", pid),
                                "raw": data.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" "),
                                "value": value.0,
                                "unit": value.1
                            }),
                            latency,
                        )
                    }
                    Err(e) => WsResponse::error(&format!("Read BMW PID failed: {}", e)),
                }
            } else {
                WsResponse::error("Not connected")
            }
        }

        WsCommand::ReadBmwPids { pids: pid_list } => {
            const MAX_PIDS: usize = 20;
            if pid_list.len() > MAX_PIDS {
                return WsResponse::error(&format!(
                    "Too many PIDs requested: {} (max {})",
                    pid_list.len(),
                    MAX_PIDS
                ));
            }

            let mut state = state.lock().await;

            if let Some(ref mut kline) = state.kline {
                let mut results = HashMap::new();
                let mut total_latency = 0u64;

                for pid in pid_list {
                    let pid_start = Instant::now();
                    match kline.read_manufacturer_pid(pid) {
                        Ok(data) => {
                            let latency = pid_start.elapsed().as_micros() as u64;
                            total_latency += latency;

                            let value = calculate_bmw_pid_value(pid, &data);
                            results.insert(
                                format!("0x{:02X}", pid),
                                serde_json::json!({
                                    "value": value.0,
                                    "unit": value.1,
                                    "latency_us": latency
                                }),
                            );
                        }
                        Err(e) => {
                            results.insert(
                                format!("0x{:02X}", pid),
                                serde_json::json!({ "error": format!("{}", e) }),
                            );
                        }
                    }
                }

                WsResponse::success_with_latency(
                    serde_json::json!({
                        "pids": results,
                        "total_latency_us": total_latency
                    }),
                    total_latency,
                )
            } else {
                WsResponse::error("Not connected")
            }
        }

        WsCommand::ReadEngineData => {
            let mut state = state.lock().await;

            if let Some(ref mut kline) = state.kline {
                // Read standard OBD-II PIDs for comprehensive engine data
                let engine_pids = vec![
                    0x04, // Engine load
                    0x05, // Coolant temp
                    0x0C, // RPM
                    0x0D, // Speed
                    0x0F, // Intake air temp
                    0x11, // Throttle position
                    0x0B, // Intake manifold pressure
                    0x0E, // Timing advance
                    0x10, // MAF rate
                    0x42, // Control module voltage
                    0x5C, // Oil temperature
                    0x06, // Short term fuel trim B1
                    0x07, // Long term fuel trim B1
                ];

                let mut results = serde_json::Map::new();
                let mut total_latency = 0u64;
                let mut errors = Vec::new();

                for pid in engine_pids {
                    let pid_start = Instant::now();
                    match kline.read_pid(pid) {
                        Ok(data) => {
                            let latency = pid_start.elapsed().as_micros() as u64;
                            total_latency += latency;

                            let (value, unit) = calculate_pid_value(pid, &data);
                            let name = get_pid_name(pid);
                            results.insert(
                                name.to_string(),
                                serde_json::json!({
                                    "pid": format!("0x{:02X}", pid),
                                    "value": value,
                                    "unit": unit
                                }),
                            );
                        }
                        Err(e) => {
                            errors.push(format!("PID 0x{:02X}: {}", pid, e));
                        }
                    }
                }

                WsResponse::success_with_latency(
                    serde_json::json!({
                        "engine": results,
                        "errors": errors,
                        "total_latency_us": total_latency
                    }),
                    total_latency,
                )
            } else {
                WsResponse::error("Not connected")
            }
        }

        WsCommand::ReadTransmissionData => {
            let mut state = state.lock().await;

            if let Some(ref mut kline) = state.kline {
                // Try to read transmission data using manufacturer PIDs (Service 0x21)
                // Note: Requires prior init to EGS (address 0x18)
                let trans_pids = vec![
                    0x01, // Current gear
                    0x02, // Target gear
                    0x03, // Gear selector position
                    0x10, // Input shaft speed
                    0x11, // Output shaft speed
                    0x20, // Transmission oil temp
                    0x40, // Engine torque
                    0x50, // Torque converter lockup
                    0x70, // Driving program
                ];

                let mut results = serde_json::Map::new();
                let mut total_latency = 0u64;
                let mut errors = Vec::new();

                for pid in trans_pids {
                    let pid_start = Instant::now();
                    match kline.read_manufacturer_pid(pid) {
                        Ok(data) => {
                            let latency = pid_start.elapsed().as_micros() as u64;
                            total_latency += latency;

                            let (value, unit) = calculate_transmission_value(pid, &data);
                            let name = get_transmission_pid_name(pid);
                            results.insert(
                                name.to_string(),
                                serde_json::json!({
                                    "pid": format!("0x{:02X}", pid),
                                    "value": value,
                                    "unit": unit,
                                    "raw": data.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ")
                                }),
                            );
                        }
                        Err(e) => {
                            errors.push(format!("PID 0x{:02X}: {}", pid, e));
                        }
                    }
                }

                WsResponse::success_with_latency(
                    serde_json::json!({
                        "transmission": results,
                        "errors": errors,
                        "note": "Requires init to EGS (0x18). Some PIDs may not be supported by your vehicle.",
                        "total_latency_us": total_latency
                    }),
                    total_latency,
                )
            } else {
                WsResponse::error("Not connected")
            }
        }
    }
}

/// Calculate PID value from raw bytes
fn calculate_pid_value(pid: u8, data: &[u8]) -> (f64, &'static str) {
    match pid {
        0x0C => {
            // RPM: ((A * 256) + B) / 4
            if data.len() >= 2 {
                let rpm = ((data[0] as f64 * 256.0) + data[1] as f64) / 4.0;
                (rpm, "RPM")
            } else {
                (0.0, "RPM")
            }
        }
        0x05 => {
            // Coolant temp: A - 40
            if !data.is_empty() {
                let temp = data[0] as f64 - 40.0;
                (temp, "°C")
            } else {
                (0.0, "°C")
            }
        }
        0x0D => {
            // Vehicle speed: A
            if !data.is_empty() {
                (data[0] as f64, "km/h")
            } else {
                (0.0, "km/h")
            }
        }
        0x11 => {
            // Throttle position: (A * 100) / 255
            if !data.is_empty() {
                let throttle = (data[0] as f64 * 100.0) / 255.0;
                (throttle, "%")
            } else {
                (0.0, "%")
            }
        }
        0x04 => {
            // Engine load: (A * 100) / 255
            if !data.is_empty() {
                let load = (data[0] as f64 * 100.0) / 255.0;
                (load, "%")
            } else {
                (0.0, "%")
            }
        }
        0x0F => {
            // Intake air temp: A - 40
            if !data.is_empty() {
                (data[0] as f64 - 40.0, "°C")
            } else {
                (0.0, "°C")
            }
        }
        0x42 => {
            // Battery voltage: ((A * 256) + B) / 1000
            if data.len() >= 2 {
                let voltage = ((data[0] as f64 * 256.0) + data[1] as f64) / 1000.0;
                (voltage, "V")
            } else {
                (0.0, "V")
            }
        }
        0x0B => {
            // Intake manifold absolute pressure: A (kPa)
            if !data.is_empty() {
                (data[0] as f64, "kPa")
            } else {
                (0.0, "kPa")
            }
        }
        0x0E => {
            // Timing advance: (A / 2) - 64
            if !data.is_empty() {
                let advance = (data[0] as f64 / 2.0) - 64.0;
                (advance, "°")
            } else {
                (0.0, "°")
            }
        }
        0x10 => {
            // MAF air flow rate: ((A * 256) + B) / 100
            if data.len() >= 2 {
                let maf = ((data[0] as f64 * 256.0) + data[1] as f64) / 100.0;
                (maf, "g/s")
            } else {
                (0.0, "g/s")
            }
        }
        0x5C => {
            // Engine oil temperature: A - 40
            if !data.is_empty() {
                (data[0] as f64 - 40.0, "°C")
            } else {
                (0.0, "°C")
            }
        }
        0x06 | 0x07 | 0x08 | 0x09 => {
            // Fuel trims: (A - 128) * 100 / 128
            if !data.is_empty() {
                let trim = ((data[0] as f64 - 128.0) * 100.0) / 128.0;
                (trim, "%")
            } else {
                (0.0, "%")
            }
        }
        _ => {
            // Unknown PID - return raw value
            if !data.is_empty() {
                (data[0] as f64, "raw")
            } else {
                (0.0, "raw")
            }
        }
    }
}

/// Get human-readable name for OBD-II PID
fn get_pid_name(pid: u8) -> &'static str {
    match pid {
        0x04 => "engine_load",
        0x05 => "coolant_temp",
        0x06 => "short_fuel_trim_b1",
        0x07 => "long_fuel_trim_b1",
        0x08 => "short_fuel_trim_b2",
        0x09 => "long_fuel_trim_b2",
        0x0B => "intake_manifold_pressure",
        0x0C => "rpm",
        0x0D => "speed",
        0x0E => "timing_advance",
        0x0F => "intake_air_temp",
        0x10 => "maf_rate",
        0x11 => "throttle_position",
        0x42 => "control_module_voltage",
        0x5C => "oil_temp",
        _ => "unknown",
    }
}

/// Calculate BMW manufacturer-specific PID value
fn calculate_bmw_pid_value(pid: u8, data: &[u8]) -> (f64, &'static str) {
    match pid {
        // Temperatures (typically: value - 48 or value - 40)
        0x10 | 0x11 | 0x12 | 0x13 => {
            if !data.is_empty() {
                (data[0] as f64 - 48.0, "°C")
            } else {
                (0.0, "°C")
            }
        }
        // RPM (typically: ((A * 256) + B) or A * 40)
        0x20 => {
            if data.len() >= 2 {
                let rpm = (data[0] as f64 * 256.0) + data[1] as f64;
                (rpm, "RPM")
            } else if !data.is_empty() {
                (data[0] as f64 * 40.0, "RPM")
            } else {
                (0.0, "RPM")
            }
        }
        // Percentages (throttle, load, etc.)
        0x21 | 0x30 | 0x31 => {
            if !data.is_empty() {
                let pct = (data[0] as f64 * 100.0) / 255.0;
                (pct, "%")
            } else {
                (0.0, "%")
            }
        }
        // Speed
        0x22 => {
            if !data.is_empty() {
                (data[0] as f64, "km/h")
            } else {
                (0.0, "km/h")
            }
        }
        // Angles (ignition, VANOS)
        0x40..=0x46 | 0x80..=0x83 => {
            if !data.is_empty() {
                let angle = (data[0] as f64 * 0.75) - 24.0; // Typical BMW scaling
                (angle, "°")
            } else {
                (0.0, "°")
            }
        }
        // Voltages
        0xA0 | 0xA1 => {
            if data.len() >= 2 {
                let voltage = ((data[0] as f64 * 256.0) + data[1] as f64) / 1000.0;
                (voltage, "V")
            } else if !data.is_empty() {
                (data[0] as f64 / 10.0, "V")
            } else {
                (0.0, "V")
            }
        }
        // Injection time (ms)
        0x50 => {
            if data.len() >= 2 {
                let ms = ((data[0] as f64 * 256.0) + data[1] as f64) / 1000.0;
                (ms, "ms")
            } else {
                (0.0, "ms")
            }
        }
        // Lambda
        0x60..=0x63 => {
            if data.len() >= 2 {
                let lambda = ((data[0] as f64 * 256.0) + data[1] as f64) / 32768.0;
                (lambda, "λ")
            } else if !data.is_empty() {
                (data[0] as f64 / 128.0, "λ")
            } else {
                (0.0, "λ")
            }
        }
        _ => {
            // Unknown - return raw
            if !data.is_empty() {
                (data[0] as f64, "raw")
            } else {
                (0.0, "raw")
            }
        }
    }
}

/// Calculate transmission-specific PID value
fn calculate_transmission_value(pid: u8, data: &[u8]) -> (f64, &'static str) {
    match pid {
        // Current gear (0=N, 1-6=gears, 7=R)
        0x01 => {
            if !data.is_empty() {
                let gear = data[0] as f64;
                (gear, "gear")
            } else {
                (0.0, "gear")
            }
        }
        // Target gear
        0x02 => {
            if !data.is_empty() {
                (data[0] as f64, "gear")
            } else {
                (0.0, "gear")
            }
        }
        // Gear selector (P=0, R=1, N=2, D=3, S=4, M=5)
        0x03 => {
            if !data.is_empty() {
                (data[0] as f64, "pos")
            } else {
                (0.0, "pos")
            }
        }
        // Shaft speeds (RPM)
        0x10 | 0x11 | 0x12 | 0x13 => {
            if data.len() >= 2 {
                let rpm = (data[0] as f64 * 256.0) + data[1] as f64;
                (rpm, "RPM")
            } else if !data.is_empty() {
                (data[0] as f64 * 40.0, "RPM")
            } else {
                (0.0, "RPM")
            }
        }
        // Temperature
        0x20 | 0x21 => {
            if !data.is_empty() {
                (data[0] as f64 - 40.0, "°C")
            } else {
                (0.0, "°C")
            }
        }
        // Pressure (bar)
        0x30 | 0x31 | 0x32 => {
            if data.len() >= 2 {
                let pressure = ((data[0] as f64 * 256.0) + data[1] as f64) / 100.0;
                (pressure, "bar")
            } else if !data.is_empty() {
                (data[0] as f64 / 10.0, "bar")
            } else {
                (0.0, "bar")
            }
        }
        // Torque (Nm)
        0x40 | 0x41 => {
            if data.len() >= 2 {
                let torque = ((data[0] as f64 * 256.0) + data[1] as f64) - 500.0;
                (torque, "Nm")
            } else if !data.is_empty() {
                (data[0] as f64 * 4.0, "Nm")
            } else {
                (0.0, "Nm")
            }
        }
        // Lockup status (0=open, 1=slipping, 2=locked)
        0x50 => {
            if !data.is_empty() {
                (data[0] as f64, "status")
            } else {
                (0.0, "status")
            }
        }
        // Driving program (0=Normal, 1=Sport, 2=Manual)
        0x70 => {
            if !data.is_empty() {
                (data[0] as f64, "mode")
            } else {
                (0.0, "mode")
            }
        }
        _ => {
            if !data.is_empty() {
                (data[0] as f64, "raw")
            } else {
                (0.0, "raw")
            }
        }
    }
}

/// Get human-readable name for transmission PID
fn get_transmission_pid_name(pid: u8) -> &'static str {
    match pid {
        0x01 => "current_gear",
        0x02 => "target_gear",
        0x03 => "selector_position",
        0x10 => "input_shaft_rpm",
        0x11 => "output_shaft_rpm",
        0x12 => "turbine_rpm",
        0x13 => "converter_slip",
        0x20 => "oil_temp",
        0x21 => "converter_temp",
        0x30 => "main_pressure",
        0x31 => "converter_pressure",
        0x32 => "shift_pressure",
        0x40 => "engine_torque",
        0x41 => "output_torque",
        0x50 => "lockup_status",
        0x70 => "driving_program",
        _ => "unknown",
    }
}
