//! BMW Diagnostic Commands for Tauri
//!
//! These commands expose BMW-specific diagnostic functions to the frontend.

use crate::bmw::{self, Dtc, EcuInfo};
use crate::constants::addresses;
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
    state.with_port(|port| {
        DCanHandler::switch_to_kline_mode(port)?;
        Ok("Switched to K-Line mode (10400 baud)".to_string())
    })
}

/// Switch to D-CAN mode
#[tauri::command]
pub fn bmw_switch_dcan(state: State<SerialState>) -> Result<String, String> {
    state.with_port(|port| {
        DCanHandler::switch_to_dcan_mode(port)?;
        Ok("Switched to D-CAN mode (500 kbaud)".to_string())
    })
}

/// Initialize K-Line communication with fast init
#[tauri::command]
pub fn bmw_kline_init(
    state: State<SerialState>,
    target_address: Option<u8>,
) -> Result<BmwInitResult, String> {
    let target = target_address.unwrap_or(addresses::DME_DDE);
    let source = addresses::TESTER;

    log::info!("Starting K-Line init to ECU 0x{:02X}", target);

    state.with_port(|port| {
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
    })
}

/// Send a diagnostic request via K-Line and get response
#[tauri::command]
pub fn bmw_kline_request(
    state: State<SerialState>,
    target_address: u8,
    service_data: Vec<u8>,
) -> Result<Vec<u8>, String> {
    state.with_port(|port| {
        KLineHandler::send_request(port, target_address, addresses::TESTER, &service_data)
    })
}

/// Read DTCs from ECU via K-Line
#[tauri::command]
pub fn bmw_read_dtcs_kline(
    state: State<SerialState>,
    target_address: Option<u8>,
) -> Result<DtcReadResult, String> {
    let target = target_address.unwrap_or(addresses::DME_DDE);
    let source = addresses::TESTER;

    state.with_port(|port| {
        // Try UDS style first (0x19 with sub-function 0x02 = reportDTCByStatusMask)
        let request = vec![0x19, 0x02, 0xFF]; // Read all DTCs with any status

        match KLineHandler::send_request(port, target, source, &request) {
            Ok(response) => {
                if response.first() == Some(&0x59) {
                    // Positive response
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
    })
}

/// Clear DTCs from ECU via K-Line
#[tauri::command]
pub fn bmw_clear_dtcs_kline(
    state: State<SerialState>,
    target_address: Option<u8>,
) -> Result<String, String> {
    let target = target_address.unwrap_or(addresses::DME_DDE);
    let source = addresses::TESTER;

    state.with_port(|port| {
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
    })
}

/// Read ECU identification
#[tauri::command]
pub fn bmw_read_ecu_id(
    state: State<SerialState>,
    target_address: Option<u8>,
) -> Result<String, String> {
    let target = target_address.unwrap_or(addresses::DME_DDE);
    let source = addresses::TESTER;

    state.with_port(|port| {
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
    })
}

/// Send TesterPresent to keep session alive
#[tauri::command]
pub fn bmw_tester_present(
    state: State<SerialState>,
    target_address: Option<u8>,
) -> Result<(), String> {
    let target = target_address.unwrap_or(addresses::DME_DDE);
    let source = addresses::TESTER;

    state.with_port(|port| KLineHandler::tester_present(port, target, source))
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
    let target = target_address.unwrap_or(addresses::DME_DDE);
    let source = addresses::TESTER;

    log::info!("Starting diagnostic session 0x{:02X} on ECU 0x{:02X}", session_type, target);

    state.with_port(|port| {
        // UDS DiagnosticSessionControl (0x10)
        let request = vec![0x10, session_type];

        match KLineHandler::send_request(port, target, source, &request) {
            Ok(response) => {
                if response.first() == Some(&0x50) {
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
    })
}

/// Perform security access (may be required for some DPF functions)
#[tauri::command]
pub fn bmw_security_access(
    state: State<SerialState>,
    target_address: Option<u8>,
    level: u8,
) -> Result<SecurityResult, String> {
    let target = target_address.unwrap_or(addresses::DME_DDE);
    let source = addresses::TESTER;

    log::info!("Starting security access level 0x{:02X} on ECU 0x{:02X}", level, target);

    state.with_port(|port| {
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
    })
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
    let target = target_address.unwrap_or(addresses::DME_DDE);
    let source = addresses::TESTER;

    log::info!("Resetting DPF ash counter on ECU 0x{:02X}", target);

    state.with_port(|port| {
        // Try primary routine ID first
        let result = execute_dpf_routine(port, target, source, dpf_routines::RESET_ASH_LOADING, routine::START)?;

        if !result.success {
            log::info!("Primary routine failed, trying alternative ID");
            return execute_dpf_routine(port, target, source, dpf_routines::alt::RESET_ASH, routine::START);
        }

        Ok(result)
    })
}

/// Reset DPF learned/adaptation values
#[tauri::command]
pub fn bmw_dpf_reset_learned(
    state: State<SerialState>,
    target_address: Option<u8>,
) -> Result<DpfRoutineResult, String> {
    let target = target_address.unwrap_or(addresses::DME_DDE);
    let source = addresses::TESTER;

    log::info!("Resetting DPF learned values on ECU 0x{:02X}", target);

    state.with_port(|port| {
        let result = execute_dpf_routine(port, target, source, dpf_routines::RESET_LEARNED_VALUES, routine::START)?;

        if !result.success {
            log::info!("Primary routine failed, trying alternative ID");
            return execute_dpf_routine(port, target, source, dpf_routines::alt::RESET_ADAPTATION, routine::START);
        }

        Ok(result)
    })
}

/// Register new DPF installed
#[tauri::command]
pub fn bmw_dpf_new_installed(
    state: State<SerialState>,
    target_address: Option<u8>,
) -> Result<DpfRoutineResult, String> {
    let target = target_address.unwrap_or(addresses::DME_DDE);
    let source = addresses::TESTER;

    log::info!("Registering new DPF on ECU 0x{:02X}", target);

    state.with_port(|port| {
        let result = execute_dpf_routine(port, target, source, dpf_routines::NEW_DPF_INSTALLED, routine::START)?;

        if !result.success {
            log::info!("Primary routine failed, trying alternative ID");
            return execute_dpf_routine(port, target, source, dpf_routines::alt::NEW_DPF, routine::START);
        }

        Ok(result)
    })
}

/// Start forced DPF regeneration
/// WARNING: Vehicle must be stationary with engine running!
#[tauri::command]
pub fn bmw_dpf_start_regen(
    state: State<SerialState>,
    target_address: Option<u8>,
) -> Result<DpfRoutineResult, String> {
    let target = target_address.unwrap_or(addresses::DME_DDE);
    let source = addresses::TESTER;

    log::warn!("Starting forced DPF regeneration on ECU 0x{:02X}", target);
    log::warn!("WARNING: Ensure vehicle is stationary and engine is running!");

    state.with_port(|port| {
        let result = execute_dpf_routine(port, target, source, dpf_routines::START_FORCED_REGEN, routine::START)?;

        if !result.success {
            log::info!("Primary routine failed, trying alternative ID");
            return execute_dpf_routine(port, target, source, dpf_routines::alt::FORCED_REGEN, routine::START);
        }

        Ok(result)
    })
}

/// Stop forced DPF regeneration
#[tauri::command]
pub fn bmw_dpf_stop_regen(
    state: State<SerialState>,
    target_address: Option<u8>,
) -> Result<DpfRoutineResult, String> {
    let target = target_address.unwrap_or(addresses::DME_DDE);
    let source = addresses::TESTER;

    log::info!("Stopping forced DPF regeneration on ECU 0x{:02X}", target);

    state.with_port(|port| {
        execute_dpf_routine(port, target, source, dpf_routines::STOP_FORCED_REGEN, routine::STOP)
    })
}

/// Read DPF status information
#[tauri::command]
pub fn bmw_dpf_read_status(
    state: State<SerialState>,
    target_address: Option<u8>,
) -> Result<DpfStatus, String> {
    let target = target_address.unwrap_or(addresses::DME_DDE);
    let source = addresses::TESTER;

    log::info!("Reading DPF status from ECU 0x{:02X}", target);

    let mut manager = state.lock_manager()?;
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
    let target = target_address.unwrap_or(addresses::DME_DDE);
    let source = addresses::TESTER;

    log::info!(
        "Executing routine 0x{:04X} sub-function 0x{:02X} on ECU 0x{:02X}",
        routine_id,
        sub_function,
        target
    );

    state.with_port(|port| {
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
    })
}

// ============================================================================
// DSC (Dynamic Stability Control) Commands - ECU Address 0x44
// ============================================================================

/// Wheel speed sensor data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WheelSpeedData {
    pub front_left: f32,
    pub front_right: f32,
    pub rear_left: f32,
    pub rear_right: f32,
    pub timestamp: u64,
}

/// DSC sensor status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DscSensorStatus {
    pub steering_angle: Option<f32>,
    pub yaw_rate: Option<f32>,
    pub lateral_acceleration: Option<f32>,
    pub longitudinal_acceleration: Option<f32>,
    pub brake_pressure: Option<f32>,
}

/// Read DTCs from DSC module
#[tauri::command]
pub fn bmw_dsc_read_dtcs(state: State<SerialState>) -> Result<DtcReadResult, String> {
    bmw_read_dtcs_kline(state, Some(addresses::DSC))
}

/// Read wheel speed sensors from DSC
#[tauri::command]
pub fn bmw_dsc_read_wheel_speeds(state: State<SerialState>) -> Result<WheelSpeedData, String> {
    let target = addresses::DSC;
    let source = addresses::TESTER;

    let mut manager = state.lock_manager()?;
    let port = manager
        .get_port_mut()
        .ok_or_else(|| "Not connected".to_string())?;

    // DIDs for wheel speeds (BMW specific)
    // Front Left: 0x4001, Front Right: 0x4002, Rear Left: 0x4003, Rear Right: 0x4004
    let wheel_dids: [(u16, &str); 4] = [
        (0x4001, "FL"),
        (0x4002, "FR"),
        (0x4003, "RL"),
        (0x4004, "RR"),
    ];

    let mut speeds = [0.0f32; 4];

    for (i, (did, name)) in wheel_dids.iter().enumerate() {
        let did_hi = (did >> 8) as u8;
        let did_lo = (did & 0xFF) as u8;
        let request = vec![0x22, did_hi, did_lo];

        match KLineHandler::send_request(port, target, source, &request) {
            Ok(response) if response.first() == Some(&0x62) && response.len() >= 5 => {
                // Speed is typically 2 bytes, scale factor 0.01 km/h
                let raw = ((response[3] as u16) << 8) | (response[4] as u16);
                speeds[i] = raw as f32 * 0.01;
            }
            Ok(response) => {
                log::warn!("Unexpected response for {} wheel: {:02X?}", name, response);
            }
            Err(e) => {
                log::warn!("Failed to read {} wheel speed: {}", name, e);
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(30));
    }

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);

    Ok(WheelSpeedData {
        front_left: speeds[0],
        front_right: speeds[1],
        rear_left: speeds[2],
        rear_right: speeds[3],
        timestamp,
    })
}

/// Read DSC sensor status
#[tauri::command]
pub fn bmw_dsc_read_sensors(state: State<SerialState>) -> Result<DscSensorStatus, String> {
    let target = addresses::DSC;
    let source = addresses::TESTER;

    let mut manager = state.lock_manager()?;
    let port = manager
        .get_port_mut()
        .ok_or_else(|| "Not connected".to_string())?;

    let mut status = DscSensorStatus {
        steering_angle: None,
        yaw_rate: None,
        lateral_acceleration: None,
        longitudinal_acceleration: None,
        brake_pressure: None,
    };

    // Read steering angle (DID 0x4010)
    let request = vec![0x22, 0x40, 0x10];
    if let Ok(response) = KLineHandler::send_request(port, target, source, &request) {
        if response.first() == Some(&0x62) && response.len() >= 5 {
            let raw = ((response[3] as i16) << 8) | (response[4] as i16);
            status.steering_angle = Some(raw as f32 * 0.1); // degrees
        }
    }

    // Read yaw rate (DID 0x4011)
    let request = vec![0x22, 0x40, 0x11];
    if let Ok(response) = KLineHandler::send_request(port, target, source, &request) {
        if response.first() == Some(&0x62) && response.len() >= 5 {
            let raw = ((response[3] as i16) << 8) | (response[4] as i16);
            status.yaw_rate = Some(raw as f32 * 0.01); // degrees/s
        }
    }

    // Read lateral acceleration (DID 0x4012)
    let request = vec![0x22, 0x40, 0x12];
    if let Ok(response) = KLineHandler::send_request(port, target, source, &request) {
        if response.first() == Some(&0x62) && response.len() >= 5 {
            let raw = ((response[3] as i16) << 8) | (response[4] as i16);
            status.lateral_acceleration = Some(raw as f32 * 0.001); // g
        }
    }

    // Read brake pressure (DID 0x4020)
    let request = vec![0x22, 0x40, 0x20];
    if let Ok(response) = KLineHandler::send_request(port, target, source, &request) {
        if response.first() == Some(&0x62) && response.len() >= 5 {
            let raw = ((response[3] as u16) << 8) | (response[4] as u16);
            status.brake_pressure = Some(raw as f32 * 0.1); // bar
        }
    }

    Ok(status)
}

/// Start ABS brake bleed routine
/// Requires extended session and security access
#[tauri::command]
pub fn bmw_dsc_bleed_brakes(
    state: State<SerialState>,
    corner: String, // "FL", "FR", "RL", "RR", or "ALL"
) -> Result<DpfRoutineResult, String> {
    let target = addresses::DSC;
    let source = addresses::TESTER;

    let routine_id: u16 = match corner.as_str() {
        "FL" => 0xFF01,
        "FR" => 0xFF02,
        "RL" => 0xFF03,
        "RR" => 0xFF04,
        "ALL" => 0xFF00,
        _ => return Err(format!("Invalid corner: {}", corner)),
    };

    log::warn!("Starting ABS bleed routine for {} on DSC", corner);

    state.with_port(|port| {
        execute_dpf_routine(port, target, source, routine_id, routine::START)
    })
}

// ============================================================================
// KOMBI (Instrument Cluster) Commands - ECU Address 0x60
// ============================================================================

/// Service interval info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub oil_service_km: Option<i32>,
    pub oil_service_days: Option<i32>,
    pub inspection_km: Option<i32>,
    pub inspection_days: Option<i32>,
    pub brake_fluid_months: Option<i32>,
}

/// Vehicle info from KOMBI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleInfo {
    pub vin: Option<String>,
    pub mileage_km: Option<u32>,
    pub fuel_level_percent: Option<f32>,
    pub coolant_temp: Option<f32>,
    pub outside_temp: Option<f32>,
}

/// Read DTCs from instrument cluster
#[tauri::command]
pub fn bmw_kombi_read_dtcs(state: State<SerialState>) -> Result<DtcReadResult, String> {
    bmw_read_dtcs_kline(state, Some(addresses::KOMBI))
}

/// Read service intervals from KOMBI
#[tauri::command]
pub fn bmw_kombi_read_service(state: State<SerialState>) -> Result<ServiceInfo, String> {
    let target = addresses::KOMBI;
    let source = addresses::TESTER;

    let mut manager = state.lock_manager()?;
    let port = manager
        .get_port_mut()
        .ok_or_else(|| "Not connected".to_string())?;

    let mut info = ServiceInfo {
        oil_service_km: None,
        oil_service_days: None,
        inspection_km: None,
        inspection_days: None,
        brake_fluid_months: None,
    };

    // Oil service distance (DID 0x6001)
    let request = vec![0x22, 0x60, 0x01];
    if let Ok(response) = KLineHandler::send_request(port, target, source, &request) {
        if response.first() == Some(&0x62) && response.len() >= 5 {
            let raw = ((response[3] as i32) << 8) | (response[4] as i32);
            info.oil_service_km = Some(raw * 100); // in 100km units
        }
    }

    // Oil service days (DID 0x6002)
    let request = vec![0x22, 0x60, 0x02];
    if let Ok(response) = KLineHandler::send_request(port, target, source, &request) {
        if response.first() == Some(&0x62) && response.len() >= 5 {
            let raw = ((response[3] as i16) << 8) | (response[4] as i16);
            info.oil_service_days = Some(raw as i32);
        }
    }

    // Inspection distance (DID 0x6003)
    let request = vec![0x22, 0x60, 0x03];
    if let Ok(response) = KLineHandler::send_request(port, target, source, &request) {
        if response.first() == Some(&0x62) && response.len() >= 5 {
            let raw = ((response[3] as i32) << 8) | (response[4] as i32);
            info.inspection_km = Some(raw * 100);
        }
    }

    // Inspection days (DID 0x6004)
    let request = vec![0x22, 0x60, 0x04];
    if let Ok(response) = KLineHandler::send_request(port, target, source, &request) {
        if response.first() == Some(&0x62) && response.len() >= 5 {
            let raw = ((response[3] as i16) << 8) | (response[4] as i16);
            info.inspection_days = Some(raw as i32);
        }
    }

    Ok(info)
}

/// Reset service intervals in KOMBI
#[tauri::command]
pub fn bmw_kombi_reset_service(
    state: State<SerialState>,
    service_type: String, // "oil", "inspection", "brake_fluid"
) -> Result<DpfRoutineResult, String> {
    let target = addresses::KOMBI;
    let source = addresses::TESTER;

    let routine_id: u16 = match service_type.as_str() {
        "oil" => 0xAB01,
        "inspection" => 0xAB02,
        "brake_fluid" => 0xAB03,
        "all" => 0xAB00,
        _ => return Err(format!("Invalid service type: {}", service_type)),
    };

    log::info!("Resetting {} service on KOMBI", service_type);

    state.with_port(|port| {
        execute_dpf_routine(port, target, source, routine_id, routine::START)
    })
}

/// Run gauge sweep test on KOMBI
#[tauri::command]
pub fn bmw_kombi_gauge_test(state: State<SerialState>) -> Result<DpfRoutineResult, String> {
    let target = addresses::KOMBI;
    let source = addresses::TESTER;

    log::info!("Starting gauge sweep test on KOMBI");

    state.with_port(|port| {
        // Gauge test routine ID varies by KOMBI version - try common IDs
        let routine_ids = [0xDF00, 0xF000, 0xFF00];

        for &routine_id in &routine_ids {
            let result = execute_dpf_routine(port, target, source, routine_id, routine::START)?;
            if result.success {
                return Ok(result);
            }
        }

        Ok(DpfRoutineResult {
            success: false,
            routine_id: 0,
            status: "Gauge test routine not supported".to_string(),
            data: vec![],
        })
    })
}

/// Read vehicle info from KOMBI (mileage, fuel, etc)
#[tauri::command]
pub fn bmw_kombi_read_info(state: State<SerialState>) -> Result<VehicleInfo, String> {
    let target = addresses::KOMBI;
    let source = addresses::TESTER;

    let mut manager = state.lock_manager()?;
    let port = manager
        .get_port_mut()
        .ok_or_else(|| "Not connected".to_string())?;

    let mut info = VehicleInfo {
        vin: None,
        mileage_km: None,
        fuel_level_percent: None,
        coolant_temp: None,
        outside_temp: None,
    };

    // Read VIN (DID 0xF190)
    let request = vec![0x22, 0xF1, 0x90];
    if let Ok(response) = KLineHandler::send_request(port, target, source, &request) {
        if response.first() == Some(&0x62) && response.len() > 3 {
            let vin: String = response[3..]
                .iter()
                .filter(|&&b| b >= 0x20 && b <= 0x7E)
                .map(|&b| b as char)
                .collect();
            if !vin.is_empty() {
                info.vin = Some(vin);
            }
        }
    }

    // Read mileage (DID 0x6010)
    let request = vec![0x22, 0x60, 0x10];
    if let Ok(response) = KLineHandler::send_request(port, target, source, &request) {
        if response.first() == Some(&0x62) && response.len() >= 6 {
            let km = ((response[3] as u32) << 16) | ((response[4] as u32) << 8) | (response[5] as u32);
            info.mileage_km = Some(km);
        }
    }

    // Read fuel level (DID 0x6011)
    let request = vec![0x22, 0x60, 0x11];
    if let Ok(response) = KLineHandler::send_request(port, target, source, &request) {
        if response.first() == Some(&0x62) && response.len() >= 4 {
            info.fuel_level_percent = Some(response[3] as f32 * 100.0 / 255.0);
        }
    }

    // Read outside temperature (DID 0x6012)
    let request = vec![0x22, 0x60, 0x12];
    if let Ok(response) = KLineHandler::send_request(port, target, source, &request) {
        if response.first() == Some(&0x62) && response.len() >= 4 {
            info.outside_temp = Some(response[3] as f32 - 40.0);
        }
    }

    Ok(info)
}

// ============================================================================
// FRM (Footwell Module - Lights) Commands - ECU Address 0x68
// ============================================================================

/// Lamp status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LampStatus {
    pub front_left_low: bool,
    pub front_right_low: bool,
    pub front_left_high: bool,
    pub front_right_high: bool,
    pub rear_left: bool,
    pub rear_right: bool,
    pub brake_left: bool,
    pub brake_right: bool,
    pub brake_center: bool,
    pub turn_front_left: bool,
    pub turn_front_right: bool,
    pub turn_rear_left: bool,
    pub turn_rear_right: bool,
    pub fog_front_left: bool,
    pub fog_front_right: bool,
    pub fog_rear: bool,
    pub reverse_left: bool,
    pub reverse_right: bool,
}

/// Read DTCs from FRM
#[tauri::command]
pub fn bmw_frm_read_dtcs(state: State<SerialState>) -> Result<DtcReadResult, String> {
    bmw_read_dtcs_kline(state, Some(addresses::FRM))
}

/// Read lamp failure status from FRM
#[tauri::command]
pub fn bmw_frm_read_lamp_status(state: State<SerialState>) -> Result<LampStatus, String> {
    let target = addresses::FRM;
    let source = addresses::TESTER;

    let mut manager = state.lock_manager()?;
    let port = manager
        .get_port_mut()
        .ok_or_else(|| "Not connected".to_string())?;

    // Read lamp status (DID 0x6800) - returns bitfield of working lamps
    let request = vec![0x22, 0x68, 0x00];
    let response = KLineHandler::send_request(port, target, source, &request)
        .map_err(|e| format!("Request failed: {}", e))?;

    if response.first() != Some(&0x62) {
        return Err("Invalid response from FRM".to_string());
    }

    // Parse lamp status from response bytes
    // Byte 3: Front lights, Byte 4: Rear lights, Byte 5: Turn signals, Byte 6: Misc
    let front = response.get(3).copied().unwrap_or(0);
    let rear = response.get(4).copied().unwrap_or(0);
    let turn = response.get(5).copied().unwrap_or(0);
    let misc = response.get(6).copied().unwrap_or(0);

    Ok(LampStatus {
        front_left_low: (front & 0x01) != 0,
        front_right_low: (front & 0x02) != 0,
        front_left_high: (front & 0x04) != 0,
        front_right_high: (front & 0x08) != 0,
        fog_front_left: (front & 0x10) != 0,
        fog_front_right: (front & 0x20) != 0,
        rear_left: (rear & 0x01) != 0,
        rear_right: (rear & 0x02) != 0,
        brake_left: (rear & 0x04) != 0,
        brake_right: (rear & 0x08) != 0,
        brake_center: (rear & 0x10) != 0,
        fog_rear: (rear & 0x20) != 0,
        turn_front_left: (turn & 0x01) != 0,
        turn_front_right: (turn & 0x02) != 0,
        turn_rear_left: (turn & 0x04) != 0,
        turn_rear_right: (turn & 0x08) != 0,
        reverse_left: (misc & 0x01) != 0,
        reverse_right: (misc & 0x02) != 0,
    })
}

/// Run lamp test on FRM (flash all lights)
#[tauri::command]
pub fn bmw_frm_lamp_test(state: State<SerialState>) -> Result<DpfRoutineResult, String> {
    let target = addresses::FRM;
    let source = addresses::TESTER;

    log::info!("Starting lamp test on FRM");

    state.with_port(|port| {
        execute_dpf_routine(port, target, source, 0xF001, routine::START)
    })
}

/// Control a specific lamp (for testing)
#[tauri::command]
pub fn bmw_frm_control_lamp(
    state: State<SerialState>,
    lamp_id: u8,
    on: bool,
) -> Result<String, String> {
    let target = addresses::FRM;
    let source = addresses::TESTER;

    state.with_port(|port| {
        // IO Control (0x2F) to control lamp
        let control_param = if on { 0x03 } else { 0x00 }; // 0x03 = ON, 0x00 = Return control
        let request = vec![0x2F, 0x68, lamp_id, control_param];

        match KLineHandler::send_request(port, target, source, &request) {
            Ok(response) => {
                if response.first() == Some(&0x6F) {
                    Ok(format!("Lamp {} {}", lamp_id, if on { "ON" } else { "OFF" }))
                } else if response.first() == Some(&0x7F) {
                    let nrc = response.get(2).copied().unwrap_or(0);
                    Err(format!("Control failed: {} (0x{:02X})", bmw::nrc::description(nrc), nrc))
                } else {
                    Err(format!("Unexpected response: {:02X?}", response))
                }
            }
            Err(e) => Err(format!("Request failed: {}", e)),
        }
    })
}

// ============================================================================
// EGS (Electronic Gearbox Control) Commands - ECU Address 0x32
// ============================================================================

/// EGS transmission status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EgsStatus {
    pub oil_temp: Option<f32>,
    pub gear_position: Option<String>,
    pub target_gear: Option<u8>,
    pub actual_gear: Option<u8>,
    pub torque_converter_lockup: bool,
    pub sport_mode: bool,
}

/// Read DTCs from EGS
#[tauri::command]
pub fn bmw_egs_read_dtcs(state: State<SerialState>) -> Result<DtcReadResult, String> {
    bmw_read_dtcs_kline(state, Some(addresses::EGS))
}

/// Read EGS transmission status
#[tauri::command]
pub fn bmw_egs_read_status(state: State<SerialState>) -> Result<EgsStatus, String> {
    let target = addresses::EGS;
    let source = addresses::TESTER;

    let mut manager = state.lock_manager()?;
    let port = manager
        .get_port_mut()
        .ok_or_else(|| "Not connected".to_string())?;

    let mut status = EgsStatus {
        oil_temp: None,
        gear_position: None,
        target_gear: None,
        actual_gear: None,
        torque_converter_lockup: false,
        sport_mode: false,
    };

    // Read oil temperature (DID 0x3201)
    let request = vec![0x22, 0x32, 0x01];
    if let Ok(response) = KLineHandler::send_request(port, target, source, &request) {
        if response.first() == Some(&0x62) && response.len() >= 4 {
            status.oil_temp = Some(response[3] as f32 - 40.0);
        }
    }

    // Read gear position (DID 0x3202)
    let request = vec![0x22, 0x32, 0x02];
    if let Ok(response) = KLineHandler::send_request(port, target, source, &request) {
        if response.first() == Some(&0x62) && response.len() >= 4 {
            let gear_byte = response[3];
            status.gear_position = Some(match gear_byte {
                0x00 => "P".to_string(),
                0x01 => "R".to_string(),
                0x02 => "N".to_string(),
                0x03 => "D".to_string(),
                0x04..=0x0A => format!("{}", gear_byte - 3),
                _ => format!("?{}", gear_byte),
            });
        }
    }

    // Read target/actual gear (DID 0x3203)
    let request = vec![0x22, 0x32, 0x03];
    if let Ok(response) = KLineHandler::send_request(port, target, source, &request) {
        if response.first() == Some(&0x62) && response.len() >= 5 {
            status.target_gear = Some(response[3]);
            status.actual_gear = Some(response[4]);
        }
    }

    // Read status flags (DID 0x3204)
    let request = vec![0x22, 0x32, 0x04];
    if let Ok(response) = KLineHandler::send_request(port, target, source, &request) {
        if response.first() == Some(&0x62) && response.len() >= 4 {
            let flags = response[3];
            status.torque_converter_lockup = (flags & 0x01) != 0;
            status.sport_mode = (flags & 0x02) != 0;
        }
    }

    Ok(status)
}

/// Reset EGS transmission adaptations
/// Requires extended session and possibly security access
#[tauri::command]
pub fn bmw_egs_reset_adaptations(state: State<SerialState>) -> Result<DpfRoutineResult, String> {
    let target = addresses::EGS;
    let source = addresses::TESTER;

    log::info!("Resetting EGS adaptations");

    state.with_port(|port| {
        // Reset adaptation routine - try common IDs
        let routine_ids = [0xFF01, 0xAB01, 0x0001];

        for &routine_id in &routine_ids {
            let result = execute_dpf_routine(port, target, source, routine_id, routine::START)?;
            if result.success {
                return Ok(result);
            }
        }

        Ok(DpfRoutineResult {
            success: false,
            routine_id: 0,
            status: "Reset adaptation routine not supported".to_string(),
            data: vec![],
        })
    })
}

// ============================================================================
// Generic Multi-ECU Commands
// ============================================================================

// ============================================================================
// D-CAN Specific Commands (for ECUs that prefer/require CAN)
// ============================================================================

use crate::dcan::{can_ids, detect_ecu_protocol};

/// Read DTCs via D-CAN
#[tauri::command]
pub fn bmw_read_dtcs_dcan(
    state: State<SerialState>,
    ecu_name: String,
) -> Result<DtcReadResult, String> {
    let (tx_id, rx_id) = can_ids::for_ecu(&ecu_name)
        .ok_or_else(|| format!("Unknown ECU for D-CAN: {}", ecu_name))?;

    state.with_port(|port| {
        // Switch to D-CAN mode
        DCanHandler::switch_to_dcan_mode(port)?;

        // Read DTCs
        match DCanHandler::read_dtcs(port, tx_id, rx_id) {
            Ok(dtcs) => Ok(DtcReadResult {
                success: true,
                count: dtcs.len(),
                dtcs,
                message: format!("DTCs read from {} via D-CAN", ecu_name),
            }),
            Err(e) => Ok(DtcReadResult {
                success: false,
                count: 0,
                dtcs: vec![],
                message: e,
            }),
        }
    })
}

/// Auto-detect protocol and read DTCs
#[tauri::command]
pub fn bmw_read_dtcs_auto(
    state: State<SerialState>,
    ecu_name: String,
    kline_address: Option<u8>,
) -> Result<DtcReadResult, String> {
    let mut manager = state.lock_manager()?;
    let port = manager
        .get_port_mut()
        .ok_or_else(|| "Not connected".to_string())?;

    // Detect protocol
    let protocol = detect_ecu_protocol(port, &ecu_name)?;

    match protocol.as_str() {
        "D-CAN" => {
            let (tx_id, rx_id) = can_ids::for_ecu(&ecu_name)
                .ok_or_else(|| format!("Unknown ECU: {}", ecu_name))?;

            match DCanHandler::read_dtcs(port, tx_id, rx_id) {
                Ok(dtcs) => Ok(DtcReadResult {
                    success: true,
                    count: dtcs.len(),
                    dtcs,
                    message: format!("DTCs read from {} via D-CAN", ecu_name),
                }),
                Err(e) => Ok(DtcReadResult {
                    success: false,
                    count: 0,
                    dtcs: vec![],
                    message: e,
                }),
            }
        }
        "K-Line" => {
            let target = kline_address.unwrap_or_else(|| {
                match ecu_name.to_uppercase().as_str() {
                    "DDE" | "DME" => addresses::DME_DDE,
                    "EGS" => addresses::EGS,
                    "DSC" => addresses::DSC,
                    "KOMBI" => addresses::KOMBI,
                    "FRM" => addresses::FRM,
                    "ACSM" => addresses::AIRBAG,
                    "CAS" => addresses::CAS,
                    _ => 0x00,
                }
            });
            let source = addresses::TESTER;

            // UDS ReadDTCInformation
            let request = vec![0x19, 0x02, 0xFF];
            match KLineHandler::send_request(port, target, source, &request) {
                Ok(response) if response.first() == Some(&0x59) => {
                    let dtcs = parse_uds_dtc_response(&response);
                    Ok(DtcReadResult {
                        success: true,
                        count: dtcs.len(),
                        dtcs,
                        message: format!("DTCs read from {} via K-Line", ecu_name),
                    })
                }
                _ => Ok(DtcReadResult {
                    success: false,
                    count: 0,
                    dtcs: vec![],
                    message: "Failed to read DTCs via K-Line".to_string(),
                }),
            }
        }
        _ => Err(format!("Unknown protocol: {}", protocol)),
    }
}

/// Detect ECU protocol (K-Line or D-CAN)
#[tauri::command]
pub fn bmw_detect_protocol(
    state: State<SerialState>,
    ecu_name: String,
) -> Result<String, String> {
    state.with_port(|port| detect_ecu_protocol(port, &ecu_name))
}

/// Read DID via D-CAN
#[tauri::command]
pub fn bmw_read_did_dcan(
    state: State<SerialState>,
    ecu_name: String,
    did: u16,
) -> Result<Vec<u8>, String> {
    let (tx_id, rx_id) = can_ids::for_ecu(&ecu_name)
        .ok_or_else(|| format!("Unknown ECU for D-CAN: {}", ecu_name))?;

    state.with_port(|port| {
        DCanHandler::switch_to_dcan_mode(port)?;
        DCanHandler::read_data_by_id(port, tx_id, rx_id, did)
    })
}

/// Start session via D-CAN
#[tauri::command]
pub fn bmw_start_session_dcan(
    state: State<SerialState>,
    ecu_name: String,
    session_type: u8,
) -> Result<SessionResult, String> {
    let (tx_id, rx_id) = can_ids::for_ecu(&ecu_name)
        .ok_or_else(|| format!("Unknown ECU for D-CAN: {}", ecu_name))?;

    state.with_port(|port| {
        DCanHandler::switch_to_dcan_mode(port)?;

        match DCanHandler::start_session(port, tx_id, rx_id, session_type) {
            Ok(()) => Ok(SessionResult {
                success: true,
                session_type,
                message: format!("Session 0x{:02X} active on {} via D-CAN", session_type, ecu_name),
            }),
            Err(e) => Ok(SessionResult {
                success: false,
                session_type,
                message: e,
            }),
        }
    })
}

/// Execute routine via D-CAN
#[tauri::command]
pub fn bmw_routine_control_dcan(
    state: State<SerialState>,
    ecu_name: String,
    routine_id: u16,
    sub_function: u8,
    data: Option<Vec<u8>>,
) -> Result<DpfRoutineResult, String> {
    let (tx_id, rx_id) = can_ids::for_ecu(&ecu_name)
        .ok_or_else(|| format!("Unknown ECU for D-CAN: {}", ecu_name))?;

    state.with_port(|port| {
        DCanHandler::switch_to_dcan_mode(port)?;

        match DCanHandler::routine_control(port, tx_id, rx_id, routine_id, sub_function, data.as_deref()) {
            Ok(result_data) => Ok(DpfRoutineResult {
                success: true,
                routine_id,
                status: "OK".to_string(),
                data: result_data,
            }),
            Err(e) => Ok(DpfRoutineResult {
                success: false,
                routine_id,
                status: e,
                data: vec![],
            }),
        }
    })
}

/// Auto-detect and read DTCs from all known ECUs
#[tauri::command]
pub fn bmw_read_all_dtcs(state: State<SerialState>) -> Result<Vec<(String, DtcReadResult)>, String> {
    let ecus = bmw::e60_ecus();
    let mut all_results = Vec::new();
    let source = addresses::TESTER;

    // We need to re-acquire the lock for each ECU
    // This is not efficient but works with the current architecture
    for ecu in ecus {
        if let Some(target) = ecu.kline_address {
            let result = {
                let mut manager = state.lock_manager()?;
                let port = manager
                    .get_port_mut()
                    .ok_or_else(|| "Not connected".to_string())?;

                // Try to init communication with this ECU first
                match KLineHandler::init_fast(port, target, source) {
                    Ok(_) => {
                        // Read DTCs
                        let request = vec![0x19, 0x02, 0xFF];
                        match KLineHandler::send_request(port, target, source, &request) {
                            Ok(response) if response.first() == Some(&0x59) => {
                                let dtcs = parse_uds_dtc_response(&response);
                                DtcReadResult {
                                    success: true,
                                    count: dtcs.len(),
                                    dtcs,
                                    message: "OK".to_string(),
                                }
                            }
                            _ => DtcReadResult {
                                success: false,
                                count: 0,
                                dtcs: vec![],
                                message: "No response".to_string(),
                            },
                        }
                    }
                    Err(_) => DtcReadResult {
                        success: false,
                        count: 0,
                        dtcs: vec![],
                        message: "ECU not responding".to_string(),
                    },
                }
            };

            all_results.push((ecu.id.clone(), result));

            // Delay between ECUs
            std::thread::sleep(std::time::Duration::from_millis(200));
        }
    }

    Ok(all_results)
}
