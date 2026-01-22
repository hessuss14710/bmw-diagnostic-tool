//! BMW Diagnostic Commands for Tauri
//!
//! These commands expose BMW-specific diagnostic functions to the frontend.

use crate::bmw::{self, Dtc, EcuInfo};
use crate::dcan::DCanHandler;
use crate::kline::KLineHandler;
use crate::serial::SerialState;
use serde::{Deserialize, Serialize};
use tauri::State;

/// BMW initialization result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BmwInitResult {
    pub success: bool,
    pub protocol: String,
    pub message: String,
}

/// DTC read result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DtcReadResult {
    pub success: bool,
    pub dtcs: Vec<Dtc>,
    pub count: usize,
    pub message: String,
}

/// Get list of known BMW E60 ECUs
#[tauri::command]
pub fn bmw_get_ecus() -> Vec<EcuInfo> {
    bmw::e60_ecus()
}

/// Switch to K-Line mode
#[tauri::command]
pub fn bmw_switch_kline(state: State<SerialState>) -> Result<String, String> {
    let mut manager = state
        .0
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;

    let port = manager
        .get_port_mut()
        .ok_or_else(|| "Not connected".to_string())?;

    DCanHandler::switch_to_kline_mode(port)?;

    Ok("Switched to K-Line mode (10400 baud)".to_string())
}

/// Switch to D-CAN mode
#[tauri::command]
pub fn bmw_switch_dcan(state: State<SerialState>) -> Result<String, String> {
    let mut manager = state
        .0
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;

    let port = manager
        .get_port_mut()
        .ok_or_else(|| "Not connected".to_string())?;

    DCanHandler::switch_to_dcan_mode(port)?;

    Ok("Switched to D-CAN mode (500 kbaud)".to_string())
}

/// Initialize K-Line communication with fast init
#[tauri::command]
pub fn bmw_kline_init(
    state: State<SerialState>,
    target_address: Option<u8>,
) -> Result<BmwInitResult, String> {
    let target = target_address.unwrap_or(0x12); // Default DME
    let source = 0xF1; // Tester

    log::info!("Starting K-Line init to ECU 0x{:02X}", target);

    let mut manager = state
        .0
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;

    let port = manager
        .get_port_mut()
        .ok_or_else(|| "Not connected".to_string())?;

    // Ensure we're in K-Line mode
    DCanHandler::switch_to_kline_mode(port)?;

    // Try fast init first
    match KLineHandler::init_fast(port, target, source) {
        Ok(response) => {
            log::info!("Fast init successful: {:02X?}", response);
            Ok(BmwInitResult {
                success: true,
                protocol: "KWP2000 Fast Init".to_string(),
                message: format!(
                    "Connected to ECU 0x{:02X}, response: {:02X?}",
                    target, response
                ),
            })
        }
        Err(e) => {
            log::warn!("Fast init failed: {}, trying 5 baud init", e);

            // Try 5 baud init as fallback
            match KLineHandler::init_5baud(port, target) {
                Ok((kb1, kb2)) => {
                    log::info!("5 baud init successful: KB1=0x{:02X}, KB2=0x{:02X}", kb1, kb2);
                    Ok(BmwInitResult {
                        success: true,
                        protocol: "ISO 9141 5-baud Init".to_string(),
                        message: format!(
                            "Connected to ECU 0x{:02X}, KB1=0x{:02X}, KB2=0x{:02X}",
                            target, kb1, kb2
                        ),
                    })
                }
                Err(e2) => {
                    log::error!("Both init methods failed");
                    Ok(BmwInitResult {
                        success: false,
                        protocol: "None".to_string(),
                        message: format!("Fast init: {}. 5-baud init: {}", e, e2),
                    })
                }
            }
        }
    }
}

/// Send a diagnostic request via K-Line and get response
#[tauri::command]
pub fn bmw_kline_request(
    state: State<SerialState>,
    target_address: u8,
    service_data: Vec<u8>,
) -> Result<Vec<u8>, String> {
    let source = 0xF1; // Tester

    let mut manager = state
        .0
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;

    let port = manager
        .get_port_mut()
        .ok_or_else(|| "Not connected".to_string())?;

    KLineHandler::send_request(port, target_address, source, &service_data)
}

/// Read DTCs from ECU via K-Line
#[tauri::command]
pub fn bmw_read_dtcs_kline(
    state: State<SerialState>,
    target_address: Option<u8>,
) -> Result<DtcReadResult, String> {
    let target = target_address.unwrap_or(0x12);
    let source = 0xF1;

    let mut manager = state
        .0
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;

    let port = manager
        .get_port_mut()
        .ok_or_else(|| "Not connected".to_string())?;

    // Read DTCs using KWP2000 service 0x18 (ReadDTCByStatus)
    // or UDS service 0x19 (ReadDTCInformation)

    // Try UDS style first (0x19 with sub-function 0x02 = reportDTCByStatusMask)
    let request = vec![0x19, 0x02, 0xFF]; // Read all DTCs with any status

    match KLineHandler::send_request(port, target, source, &request) {
        Ok(response) => {
            if response.first() == Some(&0x59) {
                // Positive response
                // Response format: [0x59] [sub-function] [status_mask] [DTC1_HI] [DTC1_LO] [STATUS1] ...
                let dtcs = parse_uds_dtc_response(&response);
                Ok(DtcReadResult {
                    success: true,
                    count: dtcs.len(),
                    dtcs,
                    message: "DTCs read successfully (UDS)".to_string(),
                })
            } else if response.first() == Some(&0x7F) {
                // Negative response, try KWP2000 style
                let kwp_request = vec![0x18, 0x00, 0xFF, 0x00]; // ReadDTCByStatus
                match KLineHandler::send_request(port, target, source, &kwp_request) {
                    Ok(kwp_response) => {
                        if kwp_response.first() == Some(&0x58) {
                            let dtcs = parse_kwp_dtc_response(&kwp_response);
                            Ok(DtcReadResult {
                                success: true,
                                count: dtcs.len(),
                                dtcs,
                                message: "DTCs read successfully (KWP2000)".to_string(),
                            })
                        } else {
                            Ok(DtcReadResult {
                                success: false,
                                count: 0,
                                dtcs: vec![],
                                message: format!("Unexpected KWP response: {:02X?}", kwp_response),
                            })
                        }
                    }
                    Err(e) => Ok(DtcReadResult {
                        success: false,
                        count: 0,
                        dtcs: vec![],
                        message: format!("KWP2000 request failed: {}", e),
                    }),
                }
            } else {
                Ok(DtcReadResult {
                    success: false,
                    count: 0,
                    dtcs: vec![],
                    message: format!("Unexpected response: {:02X?}", response),
                })
            }
        }
        Err(e) => Ok(DtcReadResult {
            success: false,
            count: 0,
            dtcs: vec![],
            message: format!("Request failed: {}", e),
        }),
    }
}

/// Clear DTCs from ECU via K-Line
#[tauri::command]
pub fn bmw_clear_dtcs_kline(
    state: State<SerialState>,
    target_address: Option<u8>,
) -> Result<String, String> {
    let target = target_address.unwrap_or(0x12);
    let source = 0xF1;

    let mut manager = state
        .0
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;

    let port = manager
        .get_port_mut()
        .ok_or_else(|| "Not connected".to_string())?;

    // UDS ClearDiagnosticInformation (0x14) with group = all (0xFFFFFF)
    let request = vec![0x14, 0xFF, 0xFF, 0xFF];

    match KLineHandler::send_request(port, target, source, &request) {
        Ok(response) => {
            if response.first() == Some(&0x54) {
                Ok("DTCs cleared successfully".to_string())
            } else if response.first() == Some(&0x7F) {
                let nrc = response.get(2).copied().unwrap_or(0);
                Err(format!(
                    "Clear failed: {} (0x{:02X})",
                    bmw::nrc::description(nrc),
                    nrc
                ))
            } else {
                Err(format!("Unexpected response: {:02X?}", response))
            }
        }
        Err(e) => Err(format!("Request failed: {}", e)),
    }
}

/// Read ECU identification
#[tauri::command]
pub fn bmw_read_ecu_id(
    state: State<SerialState>,
    target_address: Option<u8>,
) -> Result<String, String> {
    let target = target_address.unwrap_or(0x12);
    let source = 0xF1;

    let mut manager = state
        .0
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;

    let port = manager
        .get_port_mut()
        .ok_or_else(|| "Not connected".to_string())?;

    // UDS ReadDataByIdentifier (0x22) with ID 0xF190 (VIN)
    let request = vec![0x22, 0xF1, 0x90];

    match KLineHandler::send_request(port, target, source, &request) {
        Ok(response) => {
            if response.first() == Some(&0x62) {
                // Skip service ID and identifier
                let data = &response[3..];
                // Convert to string (VIN is ASCII)
                let vin: String = data
                    .iter()
                    .filter(|&&b| b >= 0x20 && b <= 0x7E)
                    .map(|&b| b as char)
                    .collect();
                Ok(vin)
            } else if response.first() == Some(&0x7F) {
                let nrc = response.get(2).copied().unwrap_or(0);
                Err(format!(
                    "Read failed: {} (0x{:02X})",
                    bmw::nrc::description(nrc),
                    nrc
                ))
            } else {
                Err(format!("Unexpected response: {:02X?}", response))
            }
        }
        Err(e) => Err(format!("Request failed: {}", e)),
    }
}

/// Send TesterPresent to keep session alive
#[tauri::command]
pub fn bmw_tester_present(
    state: State<SerialState>,
    target_address: Option<u8>,
) -> Result<(), String> {
    let target = target_address.unwrap_or(0x12);
    let source = 0xF1;

    let mut manager = state
        .0
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;

    let port = manager
        .get_port_mut()
        .ok_or_else(|| "Not connected".to_string())?;

    KLineHandler::tester_present(port, target, source)
}

// Helper functions

fn parse_uds_dtc_response(response: &[u8]) -> Vec<Dtc> {
    let mut dtcs = Vec::new();

    if response.len() < 3 {
        return dtcs;
    }

    // Skip service ID (0x59), sub-function, and status mask availability
    let data = &response[3..];

    // Each DTC is 3 bytes: DTC_HI, DTC_LO, STATUS
    for chunk in data.chunks(3) {
        if let Some(dtc) = Dtc::from_bytes(chunk) {
            dtcs.push(dtc);
        }
    }

    dtcs
}

fn parse_kwp_dtc_response(response: &[u8]) -> Vec<Dtc> {
    let mut dtcs = Vec::new();

    if response.len() < 2 {
        return dtcs;
    }

    // Skip service ID (0x58) and count
    let data = &response[2..];

    // Format varies by ECU, common is: DTC_HI, DTC_LO, STATUS
    for chunk in data.chunks(3) {
        if let Some(dtc) = Dtc::from_bytes(chunk) {
            dtcs.push(dtc);
        }
    }

    dtcs
}

// ============================================================================
// DPF (Diesel Particulate Filter) Commands
// ============================================================================

use crate::bmw::{dpf_routines, dpf_dids, security, routine, DpfRoutineResult, DpfStatus};

/// Session control result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionResult {
    pub success: bool,
    pub session_type: u8,
    pub message: String,
}

/// Security access result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityResult {
    pub success: bool,
    pub level: u8,
    pub message: String,
}

/// Start a diagnostic session (required for DPF functions)
#[tauri::command]
pub fn bmw_start_session(
    state: State<SerialState>,
    target_address: Option<u8>,
    session_type: u8,
) -> Result<SessionResult, String> {
    let target = target_address.unwrap_or(0x12); // DDE address
    let source = 0xF1;

    log::info!("Starting diagnostic session 0x{:02X} on ECU 0x{:02X}", session_type, target);

    let mut manager = state
        .0
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;

    let port = manager
        .get_port_mut()
        .ok_or_else(|| "Not connected".to_string())?;

    // UDS DiagnosticSessionControl (0x10)
    let request = vec![0x10, session_type];

    match KLineHandler::send_request(port, target, source, &request) {
        Ok(response) => {
            if response.first() == Some(&0x50) {
                // Positive response
                log::info!("Session 0x{:02X} started successfully", session_type);
                Ok(SessionResult {
                    success: true,
                    session_type,
                    message: format!("Session 0x{:02X} active", session_type),
                })
            } else if response.first() == Some(&0x7F) {
                let nrc = response.get(2).copied().unwrap_or(0);
                Ok(SessionResult {
                    success: false,
                    session_type,
                    message: format!("Session rejected: {} (0x{:02X})", bmw::nrc::description(nrc), nrc),
                })
            } else {
                Ok(SessionResult {
                    success: false,
                    session_type,
                    message: format!("Unexpected response: {:02X?}", response),
                })
            }
        }
        Err(e) => Ok(SessionResult {
            success: false,
            session_type,
            message: format!("Request failed: {}", e),
        }),
    }
}

/// Perform security access (may be required for some DPF functions)
#[tauri::command]
pub fn bmw_security_access(
    state: State<SerialState>,
    target_address: Option<u8>,
    level: u8,
) -> Result<SecurityResult, String> {
    let target = target_address.unwrap_or(0x12);
    let source = 0xF1;

    log::info!("Starting security access level 0x{:02X} on ECU 0x{:02X}", level, target);

    let mut manager = state
        .0
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;

    let port = manager
        .get_port_mut()
        .ok_or_else(|| "Not connected".to_string())?;

    // Step 1: Request seed
    let seed_request = vec![0x27, level];
    let seed_response = KLineHandler::send_request(port, target, source, &seed_request)
        .map_err(|e| format!("Seed request failed: {}", e))?;

    if seed_response.first() == Some(&0x7F) {
        let nrc = seed_response.get(2).copied().unwrap_or(0);
        return Ok(SecurityResult {
            success: false,
            level,
            message: format!("Seed request rejected: {} (0x{:02X})", bmw::nrc::description(nrc), nrc),
        });
    }

    if seed_response.first() != Some(&0x67) {
        return Ok(SecurityResult {
            success: false,
            level,
            message: format!("Unexpected seed response: {:02X?}", seed_response),
        });
    }

    // Extract seed (skip service ID and sub-function)
    let seed = &seed_response[2..];
    log::info!("Received seed: {:02X?}", seed);

    // Check if already unlocked (seed = all zeros)
    if seed.iter().all(|&b| b == 0) {
        log::info!("ECU already unlocked");
        return Ok(SecurityResult {
            success: true,
            level,
            message: "Already unlocked".to_string(),
        });
    }

    // Step 2: Calculate and send key
    let key = security::calculate_key_simple(seed);
    log::info!("Calculated key: {:02X?}", key);

    let mut key_request = vec![0x27, level + 1]; // sendKey is requestSeed + 1
    key_request.extend_from_slice(&key);

    let key_response = KLineHandler::send_request(port, target, source, &key_request)
        .map_err(|e| format!("Key request failed: {}", e))?;

    if key_response.first() == Some(&0x67) {
        log::info!("Security access granted");
        Ok(SecurityResult {
            success: true,
            level,
            message: "Security access granted".to_string(),
        })
    } else if key_response.first() == Some(&0x7F) {
        let nrc = key_response.get(2).copied().unwrap_or(0);
        Ok(SecurityResult {
            success: false,
            level,
            message: format!("Key rejected: {} (0x{:02X})", bmw::nrc::description(nrc), nrc),
        })
    } else {
        Ok(SecurityResult {
            success: false,
            level,
            message: format!("Unexpected key response: {:02X?}", key_response),
        })
    }
}

/// Execute a DPF routine (internal helper)
fn execute_dpf_routine(
    port: &mut Box<dyn serialport::SerialPort>,
    target: u8,
    source: u8,
    routine_id: u16,
    sub_function: u8,
) -> Result<DpfRoutineResult, String> {
    let routine_hi = (routine_id >> 8) as u8;
    let routine_lo = (routine_id & 0xFF) as u8;

    // RoutineControl (0x31) with sub-function and routine ID
    let request = vec![0x31, sub_function, routine_hi, routine_lo];

    log::info!(
        "Executing routine 0x{:04X} with sub-function 0x{:02X}",
        routine_id,
        sub_function
    );

    match KLineHandler::send_request(port, target, source, &request) {
        Ok(response) => {
            if response.first() == Some(&0x71) {
                // Positive response
                let status = match sub_function {
                    routine::START => "Routine started",
                    routine::STOP => "Routine stopped",
                    routine::REQUEST_RESULTS => "Results received",
                    _ => "OK",
                };
                Ok(DpfRoutineResult {
                    success: true,
                    routine_id,
                    status: status.to_string(),
                    data: response[3..].to_vec(),
                })
            } else if response.first() == Some(&0x7F) {
                let nrc = response.get(2).copied().unwrap_or(0);
                Ok(DpfRoutineResult {
                    success: false,
                    routine_id,
                    status: format!("Routine failed: {} (0x{:02X})", bmw::nrc::description(nrc), nrc),
                    data: vec![],
                })
            } else {
                Ok(DpfRoutineResult {
                    success: false,
                    routine_id,
                    status: format!("Unexpected response: {:02X?}", response),
                    data: vec![],
                })
            }
        }
        Err(e) => Ok(DpfRoutineResult {
            success: false,
            routine_id,
            status: format!("Request failed: {}", e),
            data: vec![],
        }),
    }
}

/// Reset DPF soot/ash loading counter
#[tauri::command]
pub fn bmw_dpf_reset_ash(
    state: State<SerialState>,
    target_address: Option<u8>,
) -> Result<DpfRoutineResult, String> {
    let target = target_address.unwrap_or(0x12);
    let source = 0xF1;

    log::info!("Resetting DPF ash counter on ECU 0x{:02X}", target);

    let mut manager = state
        .0
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;

    let port = manager
        .get_port_mut()
        .ok_or_else(|| "Not connected".to_string())?;

    // Try primary routine ID first
    let result = execute_dpf_routine(port, target, source, dpf_routines::RESET_ASH_LOADING, routine::START)?;

    if !result.success {
        // Try alternative routine ID
        log::info!("Primary routine failed, trying alternative ID");
        return execute_dpf_routine(port, target, source, dpf_routines::alt::RESET_ASH, routine::START);
    }

    Ok(result)
}

/// Reset DPF learned/adaptation values
#[tauri::command]
pub fn bmw_dpf_reset_learned(
    state: State<SerialState>,
    target_address: Option<u8>,
) -> Result<DpfRoutineResult, String> {
    let target = target_address.unwrap_or(0x12);
    let source = 0xF1;

    log::info!("Resetting DPF learned values on ECU 0x{:02X}", target);

    let mut manager = state
        .0
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;

    let port = manager
        .get_port_mut()
        .ok_or_else(|| "Not connected".to_string())?;

    // Try primary routine ID first
    let result = execute_dpf_routine(port, target, source, dpf_routines::RESET_LEARNED_VALUES, routine::START)?;

    if !result.success {
        // Try alternative routine ID
        log::info!("Primary routine failed, trying alternative ID");
        return execute_dpf_routine(port, target, source, dpf_routines::alt::RESET_ADAPTATION, routine::START);
    }

    Ok(result)
}

/// Register new DPF installed
#[tauri::command]
pub fn bmw_dpf_new_installed(
    state: State<SerialState>,
    target_address: Option<u8>,
) -> Result<DpfRoutineResult, String> {
    let target = target_address.unwrap_or(0x12);
    let source = 0xF1;

    log::info!("Registering new DPF on ECU 0x{:02X}", target);

    let mut manager = state
        .0
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;

    let port = manager
        .get_port_mut()
        .ok_or_else(|| "Not connected".to_string())?;

    // Try primary routine ID first
    let result = execute_dpf_routine(port, target, source, dpf_routines::NEW_DPF_INSTALLED, routine::START)?;

    if !result.success {
        // Try alternative routine ID
        log::info!("Primary routine failed, trying alternative ID");
        return execute_dpf_routine(port, target, source, dpf_routines::alt::NEW_DPF, routine::START);
    }

    Ok(result)
}

/// Start forced DPF regeneration
/// WARNING: Vehicle must be stationary with engine running!
#[tauri::command]
pub fn bmw_dpf_start_regen(
    state: State<SerialState>,
    target_address: Option<u8>,
) -> Result<DpfRoutineResult, String> {
    let target = target_address.unwrap_or(0x12);
    let source = 0xF1;

    log::warn!("Starting forced DPF regeneration on ECU 0x{:02X}", target);
    log::warn!("WARNING: Ensure vehicle is stationary and engine is running!");

    let mut manager = state
        .0
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;

    let port = manager
        .get_port_mut()
        .ok_or_else(|| "Not connected".to_string())?;

    let result = execute_dpf_routine(port, target, source, dpf_routines::START_FORCED_REGEN, routine::START)?;

    if !result.success {
        log::info!("Primary routine failed, trying alternative ID");
        return execute_dpf_routine(port, target, source, dpf_routines::alt::FORCED_REGEN, routine::START);
    }

    Ok(result)
}

/// Stop forced DPF regeneration
#[tauri::command]
pub fn bmw_dpf_stop_regen(
    state: State<SerialState>,
    target_address: Option<u8>,
) -> Result<DpfRoutineResult, String> {
    let target = target_address.unwrap_or(0x12);
    let source = 0xF1;

    log::info!("Stopping forced DPF regeneration on ECU 0x{:02X}", target);

    let mut manager = state
        .0
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;

    let port = manager
        .get_port_mut()
        .ok_or_else(|| "Not connected".to_string())?;

    execute_dpf_routine(port, target, source, dpf_routines::STOP_FORCED_REGEN, routine::STOP)
}

/// Read DPF status information
#[tauri::command]
pub fn bmw_dpf_read_status(
    state: State<SerialState>,
    target_address: Option<u8>,
) -> Result<DpfStatus, String> {
    let target = target_address.unwrap_or(0x12);
    let source = 0xF1;

    log::info!("Reading DPF status from ECU 0x{:02X}", target);

    let mut manager = state
        .0
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;

    let port = manager
        .get_port_mut()
        .ok_or_else(|| "Not connected".to_string())?;

    let mut status = DpfStatus {
        soot_loading_percent: None,
        ash_loading_grams: None,
        differential_pressure_mbar: None,
        temp_before_dpf: None,
        temp_after_dpf: None,
        distance_since_regen_km: None,
        regen_count: None,
        regen_active: false,
    };

    // Helper to read a DID
    let read_did = |port: &mut Box<dyn serialport::SerialPort>, did: u16| -> Option<Vec<u8>> {
        let did_hi = (did >> 8) as u8;
        let did_lo = (did & 0xFF) as u8;
        let request = vec![0x22, did_hi, did_lo];

        match KLineHandler::send_request(port, target, source, &request) {
            Ok(response) if response.first() == Some(&0x62) => {
                Some(response[3..].to_vec())
            }
            _ => None,
        }
    };

    // Read soot loading
    if let Some(data) = read_did(port, dpf_dids::SOOT_LOADING) {
        if !data.is_empty() {
            status.soot_loading_percent = Some(data[0] as f32 * 100.0 / 255.0);
        }
    }

    // Read ash loading
    if let Some(data) = read_did(port, dpf_dids::ASH_LOADING) {
        if data.len() >= 2 {
            status.ash_loading_grams = Some(((data[0] as u16) << 8 | data[1] as u16) as f32);
        }
    }

    // Read differential pressure
    if let Some(data) = read_did(port, dpf_dids::DIFFERENTIAL_PRESSURE) {
        if data.len() >= 2 {
            status.differential_pressure_mbar = Some(((data[0] as u16) << 8 | data[1] as u16) as f32 * 0.1);
        }
    }

    // Read temperature before DPF
    if let Some(data) = read_did(port, dpf_dids::TEMP_BEFORE_DPF) {
        if data.len() >= 2 {
            let raw = ((data[0] as u16) << 8 | data[1] as u16) as i16;
            status.temp_before_dpf = Some(raw as f32 * 0.1 - 40.0);
        }
    }

    // Read temperature after DPF
    if let Some(data) = read_did(port, dpf_dids::TEMP_AFTER_DPF) {
        if data.len() >= 2 {
            let raw = ((data[0] as u16) << 8 | data[1] as u16) as i16;
            status.temp_after_dpf = Some(raw as f32 * 0.1 - 40.0);
        }
    }

    // Read distance since regen
    if let Some(data) = read_did(port, dpf_dids::DISTANCE_SINCE_REGEN) {
        if data.len() >= 2 {
            status.distance_since_regen_km = Some(((data[0] as u16) << 8 | data[1] as u16) as f32);
        }
    }

    // Read regen count
    if let Some(data) = read_did(port, dpf_dids::REGEN_COUNT) {
        if data.len() >= 2 {
            status.regen_count = Some(((data[0] as u16) << 8 | data[1] as u16) as u32);
        }
    }

    // Read regen status
    if let Some(data) = read_did(port, dpf_dids::REGEN_STATUS) {
        if !data.is_empty() {
            status.regen_active = data[0] != 0;
        }
    }

    Ok(status)
}

/// Execute generic routine by ID (for advanced users)
#[tauri::command]
pub fn bmw_routine_control(
    state: State<SerialState>,
    target_address: Option<u8>,
    routine_id: u16,
    sub_function: u8,
    data: Option<Vec<u8>>,
) -> Result<DpfRoutineResult, String> {
    let target = target_address.unwrap_or(0x12);
    let source = 0xF1;

    log::info!(
        "Executing routine 0x{:04X} sub-function 0x{:02X} on ECU 0x{:02X}",
        routine_id,
        sub_function,
        target
    );

    let mut manager = state
        .0
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;

    let port = manager
        .get_port_mut()
        .ok_or_else(|| "Not connected".to_string())?;

    let routine_hi = (routine_id >> 8) as u8;
    let routine_lo = (routine_id & 0xFF) as u8;

    let mut request = vec![0x31, sub_function, routine_hi, routine_lo];
    if let Some(extra_data) = data {
        request.extend_from_slice(&extra_data);
    }

    match KLineHandler::send_request(port, target, source, &request) {
        Ok(response) => {
            if response.first() == Some(&0x71) {
                Ok(DpfRoutineResult {
                    success: true,
                    routine_id,
                    status: "OK".to_string(),
                    data: response[3..].to_vec(),
                })
            } else if response.first() == Some(&0x7F) {
                let nrc = response.get(2).copied().unwrap_or(0);
                Ok(DpfRoutineResult {
                    success: false,
                    routine_id,
                    status: format!("{} (0x{:02X})", bmw::nrc::description(nrc), nrc),
                    data: vec![],
                })
            } else {
                Ok(DpfRoutineResult {
                    success: false,
                    routine_id,
                    status: format!("Unexpected: {:02X?}", response),
                    data: vec![],
                })
            }
        }
        Err(e) => Ok(DpfRoutineResult {
            success: false,
            routine_id,
            status: format!("Failed: {}", e),
            data: vec![],
        }),
    }
}
