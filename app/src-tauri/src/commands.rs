use crate::serial::{ConnectionState, PortInfo, SerialManager, SerialState};
use serde::{Deserialize, Serialize};
use tauri::State;

/// Response for connection status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionStatus {
    pub state: String,
    pub port: Option<String>,
    pub error: Option<String>,
}

impl From<ConnectionState> for ConnectionStatus {
    fn from(state: ConnectionState) -> Self {
        match state {
            ConnectionState::Disconnected => ConnectionStatus {
                state: "disconnected".to_string(),
                port: None,
                error: None,
            },
            ConnectionState::Connecting => ConnectionStatus {
                state: "connecting".to_string(),
                port: None,
                error: None,
            },
            ConnectionState::Connected => ConnectionStatus {
                state: "connected".to_string(),
                port: None,
                error: None,
            },
            ConnectionState::Error(e) => ConnectionStatus {
                state: "error".to_string(),
                port: None,
                error: Some(e),
            },
        }
    }
}

/// List all available serial ports
#[tauri::command]
pub fn list_serial_ports() -> Result<Vec<PortInfo>, String> {
    log::info!("Listing serial ports...");
    let ports = SerialManager::list_ports()?;
    log::info!("Found {} ports", ports.len());
    Ok(ports)
}

/// Connect to a serial port
#[tauri::command]
pub fn serial_connect(
    state: State<SerialState>,
    port_name: String,
    baud_rate: Option<u32>,
) -> Result<ConnectionStatus, String> {
    let baud = baud_rate.unwrap_or(10400); // K-Line default
    log::info!("Connecting to {} at {} baud", port_name, baud);

    let mut manager = state
        .0
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;

    manager.connect(&port_name, baud)?;

    let mut status: ConnectionStatus = manager.get_state().into();
    status.port = manager.get_current_port();

    Ok(status)
}

/// Disconnect from the current port
#[tauri::command]
pub fn serial_disconnect(state: State<SerialState>) -> Result<ConnectionStatus, String> {
    log::info!("Disconnecting...");

    let mut manager = state
        .0
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;

    manager.disconnect()?;

    Ok(manager.get_state().into())
}

/// Get current connection status
#[tauri::command]
pub fn serial_status(state: State<SerialState>) -> Result<ConnectionStatus, String> {
    let manager = state
        .0
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;

    let mut status: ConnectionStatus = manager.get_state().into();
    status.port = manager.get_current_port();

    Ok(status)
}

/// Send raw bytes to the serial port
#[tauri::command]
pub fn serial_write(state: State<SerialState>, data: Vec<u8>) -> Result<usize, String> {
    let mut manager = state
        .0
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;

    manager.write(&data)
}

/// Read available bytes from the serial port
#[tauri::command]
pub fn serial_read(state: State<SerialState>) -> Result<Vec<u8>, String> {
    let mut manager = state
        .0
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;

    manager.read_available()
}

/// Send a command and wait for response (with hex strings for easier debugging)
#[tauri::command]
pub fn serial_send_hex(state: State<SerialState>, hex_data: String) -> Result<String, String> {
    // Parse hex string to bytes
    let hex_clean: String = hex_data.chars().filter(|c| c.is_ascii_hexdigit()).collect();

    if hex_clean.len() % 2 != 0 {
        return Err("Invalid hex string length".to_string());
    }

    let bytes: Result<Vec<u8>, _> = (0..hex_clean.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex_clean[i..i + 2], 16))
        .collect();

    let data = bytes.map_err(|e| format!("Invalid hex: {}", e))?;

    let mut manager = state
        .0
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;

    // Write data
    manager.write(&data)?;

    // Wait a bit and read response
    std::thread::sleep(std::time::Duration::from_millis(100));

    let response = manager.read_available()?;

    // Convert response to hex string
    let hex_response: String = response.iter().map(|b| format!("{:02X}", b)).collect();

    Ok(hex_response)
}

/// Set DTR line (used for K-Line switching on some adapters)
#[tauri::command]
pub fn serial_set_dtr(state: State<SerialState>, level: bool) -> Result<(), String> {
    let mut manager = state
        .0
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;

    manager.set_dtr(level)
}

/// Set RTS line
#[tauri::command]
pub fn serial_set_rts(state: State<SerialState>, level: bool) -> Result<(), String> {
    let mut manager = state
        .0
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;

    manager.set_rts(level)
}

/// Change baud rate
#[tauri::command]
pub fn serial_set_baud(state: State<SerialState>, baud_rate: u32) -> Result<(), String> {
    let mut manager = state
        .0
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;

    manager.set_baud_rate(baud_rate)
}

/// Clear serial buffers
#[tauri::command]
pub fn serial_clear(state: State<SerialState>) -> Result<(), String> {
    let mut manager = state
        .0
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;

    manager.clear_buffers()
}
