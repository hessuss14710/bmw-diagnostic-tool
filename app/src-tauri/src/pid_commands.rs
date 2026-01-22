//! PID Reading Commands for Live Data
//!
//! Reads OBD-II PIDs and BMW-specific live data from ECUs.

use crate::kline::KLineHandler;
use crate::serial::SerialState;
use serde::{Deserialize, Serialize};
use tauri::State;

/// PID definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PidDefinition {
    pub id: u16,
    pub name: String,
    pub short_name: String,
    pub unit: String,
    pub min: f64,
    pub max: f64,
    pub format: String, // "temperature", "rpm", "percent", "speed", "voltage", "pressure"
}

/// Live data value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveDataValue {
    pub pid: u16,
    pub name: String,
    pub value: f64,
    pub unit: String,
    pub raw: Vec<u8>,
    pub timestamp: u64,
}

/// Available PIDs that can be read
#[tauri::command]
pub fn get_available_pids() -> Vec<PidDefinition> {
    vec![
        PidDefinition {
            id: 0x05,
            name: "Engine Coolant Temperature".to_string(),
            short_name: "Coolant".to_string(),
            unit: "°C".to_string(),
            min: -40.0,
            max: 215.0,
            format: "temperature".to_string(),
        },
        PidDefinition {
            id: 0x0C,
            name: "Engine RPM".to_string(),
            short_name: "RPM".to_string(),
            unit: "rpm".to_string(),
            min: 0.0,
            max: 8000.0,
            format: "rpm".to_string(),
        },
        PidDefinition {
            id: 0x0D,
            name: "Vehicle Speed".to_string(),
            short_name: "Speed".to_string(),
            unit: "km/h".to_string(),
            min: 0.0,
            max: 255.0,
            format: "speed".to_string(),
        },
        PidDefinition {
            id: 0x0F,
            name: "Intake Air Temperature".to_string(),
            short_name: "Intake".to_string(),
            unit: "°C".to_string(),
            min: -40.0,
            max: 215.0,
            format: "temperature".to_string(),
        },
        PidDefinition {
            id: 0x10,
            name: "MAF Air Flow Rate".to_string(),
            short_name: "MAF".to_string(),
            unit: "g/s".to_string(),
            min: 0.0,
            max: 655.35,
            format: "flow".to_string(),
        },
        PidDefinition {
            id: 0x11,
            name: "Throttle Position".to_string(),
            short_name: "Throttle".to_string(),
            unit: "%".to_string(),
            min: 0.0,
            max: 100.0,
            format: "percent".to_string(),
        },
        PidDefinition {
            id: 0x2F,
            name: "Fuel Tank Level".to_string(),
            short_name: "Fuel".to_string(),
            unit: "%".to_string(),
            min: 0.0,
            max: 100.0,
            format: "percent".to_string(),
        },
        PidDefinition {
            id: 0x42,
            name: "Control Module Voltage".to_string(),
            short_name: "Voltage".to_string(),
            unit: "V".to_string(),
            min: 0.0,
            max: 65.535,
            format: "voltage".to_string(),
        },
        PidDefinition {
            id: 0x46,
            name: "Ambient Air Temperature".to_string(),
            short_name: "Ambient".to_string(),
            unit: "°C".to_string(),
            min: -40.0,
            max: 215.0,
            format: "temperature".to_string(),
        },
        PidDefinition {
            id: 0x5C,
            name: "Engine Oil Temperature".to_string(),
            short_name: "Oil Temp".to_string(),
            unit: "°C".to_string(),
            min: -40.0,
            max: 210.0,
            format: "temperature".to_string(),
        },
    ]
}

/// Read a single PID value via K-Line
#[tauri::command]
pub fn read_pid_kline(
    state: State<SerialState>,
    target_address: u8,
    pid: u16,
) -> Result<LiveDataValue, String> {
    let source = 0xF1;

    let mut manager = state
        .0
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;

    let port = manager
        .get_port_mut()
        .ok_or_else(|| "Not connected".to_string())?;

    // OBD-II Mode 01 - Show current data
    // Request format: [0x01] [PID]
    let request = if pid <= 0xFF {
        vec![0x01, pid as u8]
    } else {
        // Extended PID (2 bytes)
        vec![0x01, (pid >> 8) as u8, (pid & 0xFF) as u8]
    };

    let response = KLineHandler::send_request(port, target_address, source, &request)?;

    // Parse response
    // Response format: [0x41] [PID] [DATA...]
    if response.first() != Some(&0x41) {
        if response.first() == Some(&0x7F) {
            let nrc = response.get(2).copied().unwrap_or(0);
            return Err(format!("Negative response: 0x{:02X}", nrc));
        }
        return Err(format!("Unexpected response: {:02X?}", response));
    }

    // Extract data bytes (skip service ID and PID)
    let data_start = if pid <= 0xFF { 2 } else { 3 };
    let data = &response[data_start..];

    // Calculate value based on PID
    let (value, unit, name) = calculate_pid_value(pid, data)?;

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);

    Ok(LiveDataValue {
        pid,
        name,
        value,
        unit,
        raw: data.to_vec(),
        timestamp,
    })
}

/// Read multiple PIDs in sequence
#[tauri::command]
pub fn read_pids_kline(
    state: State<SerialState>,
    target_address: u8,
    pids: Vec<u16>,
) -> Result<Vec<LiveDataValue>, String> {
    let source = 0xF1;

    let mut manager = state
        .0
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;

    let port = manager
        .get_port_mut()
        .ok_or_else(|| "Not connected".to_string())?;

    let mut results = Vec::new();

    for pid in pids {
        let request = if pid <= 0xFF {
            vec![0x01, pid as u8]
        } else {
            vec![0x01, (pid >> 8) as u8, (pid & 0xFF) as u8]
        };

        match KLineHandler::send_request(port, target_address, source, &request) {
            Ok(response) => {
                if response.first() == Some(&0x41) {
                    let data_start = if pid <= 0xFF { 2 } else { 3 };
                    let data = &response[data_start..];

                    if let Ok((value, unit, name)) = calculate_pid_value(pid, data) {
                        let timestamp = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map(|d| d.as_millis() as u64)
                            .unwrap_or(0);

                        results.push(LiveDataValue {
                            pid,
                            name,
                            value,
                            unit,
                            raw: data.to_vec(),
                            timestamp,
                        });
                    }
                }
            }
            Err(e) => {
                log::warn!("Failed to read PID 0x{:02X}: {}", pid, e);
            }
        }

        // Small delay between PIDs to avoid overwhelming the ECU
        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    Ok(results)
}

/// Calculate PID value from raw bytes
fn calculate_pid_value(pid: u16, data: &[u8]) -> Result<(f64, String, String), String> {
    let a = data.first().copied().unwrap_or(0) as f64;
    let b = data.get(1).copied().unwrap_or(0) as f64;

    let (value, unit, name) = match pid {
        // Engine coolant temperature
        0x05 => (a - 40.0, "°C".to_string(), "Coolant Temp".to_string()),

        // Engine RPM
        0x0C => (
            (256.0 * a + b) / 4.0,
            "rpm".to_string(),
            "Engine RPM".to_string(),
        ),

        // Vehicle speed
        0x0D => (a, "km/h".to_string(), "Vehicle Speed".to_string()),

        // Intake air temperature
        0x0F => (a - 40.0, "°C".to_string(), "Intake Air Temp".to_string()),

        // MAF air flow rate
        0x10 => (
            (256.0 * a + b) / 100.0,
            "g/s".to_string(),
            "MAF Rate".to_string(),
        ),

        // Throttle position
        0x11 => (
            a * 100.0 / 255.0,
            "%".to_string(),
            "Throttle Position".to_string(),
        ),

        // Fuel tank level input
        0x2F => (
            a * 100.0 / 255.0,
            "%".to_string(),
            "Fuel Level".to_string(),
        ),

        // Control module voltage
        0x42 => (
            (256.0 * a + b) / 1000.0,
            "V".to_string(),
            "Battery Voltage".to_string(),
        ),

        // Ambient air temperature
        0x46 => (a - 40.0, "°C".to_string(), "Ambient Temp".to_string()),

        // Engine oil temperature
        0x5C => (a - 40.0, "°C".to_string(), "Oil Temp".to_string()),

        // Absolute load value
        0x43 => (
            (256.0 * a + b) * 100.0 / 255.0,
            "%".to_string(),
            "Absolute Load".to_string(),
        ),

        // Timing advance
        0x0E => (a / 2.0 - 64.0, "°".to_string(), "Timing Advance".to_string()),

        // Short term fuel trim Bank 1
        0x06 => (
            (a - 128.0) * 100.0 / 128.0,
            "%".to_string(),
            "STFT Bank 1".to_string(),
        ),

        // Long term fuel trim Bank 1
        0x07 => (
            (a - 128.0) * 100.0 / 128.0,
            "%".to_string(),
            "LTFT Bank 1".to_string(),
        ),

        // Intake manifold pressure
        0x0B => (a, "kPa".to_string(), "Intake Pressure".to_string()),

        // Unknown PID - return raw value
        _ => (a, "raw".to_string(), format!("PID 0x{:02X}", pid)),
    };

    Ok((value, unit, name))
}
