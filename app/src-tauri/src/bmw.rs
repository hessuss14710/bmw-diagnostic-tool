//! BMW Diagnostic Types and Structures
//!
//! This module contains BMW-specific types, ECU definitions, and diagnostic services.

use serde::{Deserialize, Serialize};

/// BMW ECU definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcuInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub kline_address: Option<u8>,
    pub can_tx_id: Option<u32>,
    pub can_rx_id: Option<u32>,
    pub protocol: Protocol,
}

/// Communication protocol
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum Protocol {
    KLine,
    DCan,
    Both,
}

/// Diagnostic Trouble Code (DTC)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dtc {
    pub code: String,
    pub status: DtcStatus,
    pub description: Option<String>,
    pub raw_bytes: Vec<u8>,
}

/// DTC Status byte flags
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DtcStatus {
    pub test_failed: bool,
    pub test_failed_this_cycle: bool,
    pub pending: bool,
    pub confirmed: bool,
    pub test_not_completed_since_clear: bool,
    pub test_failed_since_clear: bool,
    pub test_not_completed_this_cycle: bool,
    pub warning_indicator_requested: bool,
    pub raw: u8,
}

impl DtcStatus {
    pub fn from_byte(byte: u8) -> Self {
        Self {
            test_failed: (byte & 0x01) != 0,
            test_failed_this_cycle: (byte & 0x02) != 0,
            pending: (byte & 0x04) != 0,
            confirmed: (byte & 0x08) != 0,
            test_not_completed_since_clear: (byte & 0x10) != 0,
            test_failed_since_clear: (byte & 0x20) != 0,
            test_not_completed_this_cycle: (byte & 0x40) != 0,
            warning_indicator_requested: (byte & 0x80) != 0,
            raw: byte,
        }
    }
}

impl Dtc {
    /// Parse DTC from raw bytes (3 bytes: DTC high, DTC low, status)
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 3 {
            return None;
        }

        let dtc_high = bytes[0];
        let dtc_low = bytes[1];
        let status = bytes[2];

        // Convert to standard DTC format (P0XXX, C0XXX, B0XXX, U0XXX)
        let code = Self::bytes_to_code(dtc_high, dtc_low);

        Some(Self {
            code,
            status: DtcStatus::from_byte(status),
            description: None,
            raw_bytes: bytes[..3].to_vec(),
        })
    }

    /// Convert two bytes to DTC code string
    fn bytes_to_code(high: u8, low: u8) -> String {
        // First 2 bits determine category
        let category = match (high >> 6) & 0x03 {
            0 => 'P', // Powertrain
            1 => 'C', // Chassis
            2 => 'B', // Body
            3 => 'U', // Network
            _ => '?',
        };

        // Remaining bits form the number
        let number = ((high as u16 & 0x3F) << 8) | (low as u16);

        format!("{}{:04X}", category, number)
    }
}

/// Live data PID
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pid {
    pub id: u16,
    pub name: String,
    pub description: String,
    pub unit: String,
    pub formula: String,
    pub min: f64,
    pub max: f64,
}

/// Live data value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveValue {
    pub pid: u16,
    pub name: String,
    pub value: f64,
    pub unit: String,
    pub raw_bytes: Vec<u8>,
}

/// UDS (Unified Diagnostic Services) service IDs
pub mod uds {
    pub const DIAGNOSTIC_SESSION_CONTROL: u8 = 0x10;
    pub const ECU_RESET: u8 = 0x11;
    pub const CLEAR_DIAGNOSTIC_INFO: u8 = 0x14;
    pub const READ_DTC_INFO: u8 = 0x19;
    pub const READ_DATA_BY_ID: u8 = 0x22;
    pub const READ_MEMORY_BY_ADDRESS: u8 = 0x23;
    pub const SECURITY_ACCESS: u8 = 0x27;
    pub const COMMUNICATION_CONTROL: u8 = 0x28;
    pub const WRITE_DATA_BY_ID: u8 = 0x2E;
    pub const IO_CONTROL: u8 = 0x2F;
    pub const ROUTINE_CONTROL: u8 = 0x31;
    pub const REQUEST_DOWNLOAD: u8 = 0x34;
    pub const REQUEST_UPLOAD: u8 = 0x35;
    pub const TRANSFER_DATA: u8 = 0x36;
    pub const REQUEST_TRANSFER_EXIT: u8 = 0x37;
    pub const WRITE_MEMORY_BY_ADDRESS: u8 = 0x3D;
    pub const TESTER_PRESENT: u8 = 0x3E;
    pub const CONTROL_DTC_SETTING: u8 = 0x85;

    // Positive response = service ID + 0x40
    pub const POSITIVE_RESPONSE_OFFSET: u8 = 0x40;
    pub const NEGATIVE_RESPONSE: u8 = 0x7F;

    // Diagnostic session types
    pub const SESSION_DEFAULT: u8 = 0x01;
    pub const SESSION_PROGRAMMING: u8 = 0x02;
    pub const SESSION_EXTENDED: u8 = 0x03;

    // Read DTC sub-functions
    pub const REPORT_DTC_BY_STATUS_MASK: u8 = 0x02;
    pub const REPORT_DTC_SNAPSHOT_BY_DTC: u8 = 0x04;
    pub const REPORT_SUPPORTED_DTC: u8 = 0x0A;
}

/// KWP2000 service IDs (for K-Line)
pub mod kwp {
    pub const START_COMMUNICATION: u8 = 0x81;
    pub const STOP_COMMUNICATION: u8 = 0x82;
    pub const ACCESS_TIMING_PARAMETER: u8 = 0x83;
    pub const TESTER_PRESENT: u8 = 0x3E;
    pub const START_DIAGNOSTIC_SESSION: u8 = 0x10;
    pub const ECU_RESET: u8 = 0x11;
    pub const CLEAR_DIAGNOSTIC_INFO: u8 = 0x14;
    pub const READ_STATUS_OF_DTC: u8 = 0x17;
    pub const READ_DTC_BY_STATUS: u8 = 0x18;
    pub const READ_ECU_ID: u8 = 0x1A;
    pub const READ_DATA_BY_LOCAL_ID: u8 = 0x21;
    pub const READ_DATA_BY_COMMON_ID: u8 = 0x22;
    pub const READ_MEMORY_BY_ADDRESS: u8 = 0x23;
    pub const SECURITY_ACCESS: u8 = 0x27;
    pub const DISABLE_NORMAL_TRANSMISSION: u8 = 0x28;
    pub const ENABLE_NORMAL_TRANSMISSION: u8 = 0x29;
    pub const DYNAMICALLY_DEFINE_LOCAL_ID: u8 = 0x2C;
    pub const WRITE_DATA_BY_LOCAL_ID: u8 = 0x3B;
    pub const WRITE_MEMORY_BY_ADDRESS: u8 = 0x3D;
    pub const INPUT_OUTPUT_CONTROL: u8 = 0x30;
    pub const START_ROUTINE_BY_LOCAL_ID: u8 = 0x31;
    pub const STOP_ROUTINE_BY_LOCAL_ID: u8 = 0x32;
    pub const REQUEST_ROUTINE_RESULTS: u8 = 0x33;
    pub const REQUEST_DOWNLOAD: u8 = 0x34;
    pub const REQUEST_UPLOAD: u8 = 0x35;
    pub const TRANSFER_DATA: u8 = 0x36;
    pub const REQUEST_TRANSFER_EXIT: u8 = 0x37;

    // Positive response = service ID + 0x40
    pub const POSITIVE_RESPONSE_OFFSET: u8 = 0x40;
    pub const NEGATIVE_RESPONSE: u8 = 0x7F;
}

/// Negative Response Codes (NRC)
pub mod nrc {
    pub const GENERAL_REJECT: u8 = 0x10;
    pub const SERVICE_NOT_SUPPORTED: u8 = 0x11;
    pub const SUB_FUNCTION_NOT_SUPPORTED: u8 = 0x12;
    pub const INCORRECT_MESSAGE_LENGTH: u8 = 0x13;
    pub const BUSY_REPEAT_REQUEST: u8 = 0x21;
    pub const CONDITIONS_NOT_CORRECT: u8 = 0x22;
    pub const REQUEST_SEQUENCE_ERROR: u8 = 0x24;
    pub const REQUEST_OUT_OF_RANGE: u8 = 0x31;
    pub const SECURITY_ACCESS_DENIED: u8 = 0x33;
    pub const INVALID_KEY: u8 = 0x35;
    pub const EXCEEDED_NUMBER_OF_ATTEMPTS: u8 = 0x36;
    pub const REQUIRED_TIME_DELAY_NOT_EXPIRED: u8 = 0x37;
    pub const UPLOAD_DOWNLOAD_NOT_ACCEPTED: u8 = 0x70;
    pub const TRANSFER_DATA_SUSPENDED: u8 = 0x71;
    pub const GENERAL_PROGRAMMING_FAILURE: u8 = 0x72;
    pub const SERVICE_NOT_SUPPORTED_IN_ACTIVE_SESSION: u8 = 0x7F;

    pub fn description(code: u8) -> &'static str {
        match code {
            0x10 => "General reject",
            0x11 => "Service not supported",
            0x12 => "Sub-function not supported",
            0x13 => "Incorrect message length or invalid format",
            0x21 => "Busy - repeat request",
            0x22 => "Conditions not correct",
            0x24 => "Request sequence error",
            0x31 => "Request out of range",
            0x33 => "Security access denied",
            0x35 => "Invalid key",
            0x36 => "Exceeded number of attempts",
            0x37 => "Required time delay not expired",
            0x70 => "Upload/download not accepted",
            0x71 => "Transfer data suspended",
            0x72 => "General programming failure",
            0x7F => "Service not supported in active session",
            _ => "Unknown error",
        }
    }
}

/// BMW E60 ECU definitions
pub fn e60_ecus() -> Vec<EcuInfo> {
    vec![
        EcuInfo {
            id: "DME".to_string(),
            name: "Digital Motor Electronics".to_string(),
            description: "Engine control unit (petrol)".to_string(),
            kline_address: Some(0x12),
            can_tx_id: Some(0x612),
            can_rx_id: Some(0x612),
            protocol: Protocol::Both,
        },
        EcuInfo {
            id: "DDE".to_string(),
            name: "Digital Diesel Electronics".to_string(),
            description: "Engine control unit (diesel) - DPF control".to_string(),
            kline_address: Some(0x12),
            can_tx_id: Some(0x612),
            can_rx_id: Some(0x612),
            protocol: Protocol::Both,
        },
        EcuInfo {
            id: "EGS".to_string(),
            name: "Electronic Transmission Control".to_string(),
            description: "Automatic transmission".to_string(),
            kline_address: Some(0x32),
            can_tx_id: Some(0x618),
            can_rx_id: Some(0x618),
            protocol: Protocol::Both,
        },
        EcuInfo {
            id: "DSC".to_string(),
            name: "Dynamic Stability Control".to_string(),
            description: "ABS/Traction control".to_string(),
            kline_address: Some(0x44),
            can_tx_id: Some(0x6D8),
            can_rx_id: Some(0x6D8),
            protocol: Protocol::Both,
        },
        EcuInfo {
            id: "ACSM".to_string(),
            name: "Airbag Control Module".to_string(),
            description: "Crash safety module".to_string(),
            kline_address: Some(0x4A),
            can_tx_id: Some(0x6B8),
            can_rx_id: Some(0x6B8),
            protocol: Protocol::Both,
        },
        EcuInfo {
            id: "IHKA".to_string(),
            name: "Integrated Heating/Climate Control".to_string(),
            description: "Climate control".to_string(),
            kline_address: Some(0x5B),
            can_tx_id: None,
            can_rx_id: None,
            protocol: Protocol::KLine,
        },
        EcuInfo {
            id: "KOMBI".to_string(),
            name: "Instrument Cluster".to_string(),
            description: "Dashboard/gauges".to_string(),
            kline_address: Some(0x60),
            can_tx_id: Some(0x6F1),
            can_rx_id: Some(0x660),
            protocol: Protocol::Both,
        },
        EcuInfo {
            id: "CAS".to_string(),
            name: "Car Access System".to_string(),
            description: "Immobilizer/key".to_string(),
            kline_address: None,
            can_tx_id: Some(0x6F1),
            can_rx_id: Some(0x640),
            protocol: Protocol::DCan,
        },
        EcuInfo {
            id: "FRM".to_string(),
            name: "Footwell Module".to_string(),
            description: "Lights/switches".to_string(),
            kline_address: Some(0x68),
            can_tx_id: Some(0x6F1),
            can_rx_id: Some(0x668),
            protocol: Protocol::Both,
        },
        EcuInfo {
            id: "CCC".to_string(),
            name: "Car Communication Computer".to_string(),
            description: "iDrive/navigation".to_string(),
            kline_address: None,
            can_tx_id: Some(0x6F1),
            can_rx_id: Some(0x663),
            protocol: Protocol::DCan,
        },
        EcuInfo {
            id: "PDC".to_string(),
            name: "Park Distance Control".to_string(),
            description: "Parking sensors".to_string(),
            kline_address: None,
            can_tx_id: Some(0x6F1),
            can_rx_id: Some(0x672),
            protocol: Protocol::DCan,
        },
    ]
}

/// Common OBD-II PIDs
pub fn common_pids() -> Vec<Pid> {
    vec![
        Pid {
            id: 0x05,
            name: "Engine Coolant Temp".to_string(),
            description: "Engine coolant temperature".to_string(),
            unit: "°C".to_string(),
            formula: "A - 40".to_string(),
            min: -40.0,
            max: 215.0,
        },
        Pid {
            id: 0x0C,
            name: "Engine RPM".to_string(),
            description: "Engine speed".to_string(),
            unit: "rpm".to_string(),
            formula: "(256*A + B) / 4".to_string(),
            min: 0.0,
            max: 16383.75,
        },
        Pid {
            id: 0x0D,
            name: "Vehicle Speed".to_string(),
            description: "Vehicle speed".to_string(),
            unit: "km/h".to_string(),
            formula: "A".to_string(),
            min: 0.0,
            max: 255.0,
        },
        Pid {
            id: 0x0F,
            name: "Intake Air Temp".to_string(),
            description: "Intake air temperature".to_string(),
            unit: "°C".to_string(),
            formula: "A - 40".to_string(),
            min: -40.0,
            max: 215.0,
        },
        Pid {
            id: 0x10,
            name: "MAF Air Flow".to_string(),
            description: "Mass air flow rate".to_string(),
            unit: "g/s".to_string(),
            formula: "(256*A + B) / 100".to_string(),
            min: 0.0,
            max: 655.35,
        },
        Pid {
            id: 0x11,
            name: "Throttle Position".to_string(),
            description: "Throttle position".to_string(),
            unit: "%".to_string(),
            formula: "A * 100 / 255".to_string(),
            min: 0.0,
            max: 100.0,
        },
        Pid {
            id: 0x2F,
            name: "Fuel Level".to_string(),
            description: "Fuel tank level".to_string(),
            unit: "%".to_string(),
            formula: "A * 100 / 255".to_string(),
            min: 0.0,
            max: 100.0,
        },
        Pid {
            id: 0x46,
            name: "Ambient Air Temp".to_string(),
            description: "Ambient air temperature".to_string(),
            unit: "°C".to_string(),
            formula: "A - 40".to_string(),
            min: -40.0,
            max: 215.0,
        },
    ]
}

/// DPF (Diesel Particulate Filter) routine IDs for BMW DDE
/// These are used with RoutineControl service (0x31)
pub mod dpf_routines {
    /// Reset DPF ash/soot counter (Rußbeladung zurücksetzen)
    /// This resets the calculated soot loading value
    pub const RESET_SOOT_LOADING: u16 = 0xA090;

    /// Reset DPF ash accumulation counter (Aschebeladung zurücksetzen)
    /// This resets the ash accumulation counter after DPF cleaning
    pub const RESET_ASH_LOADING: u16 = 0xA091;

    /// Reset DPF learned values / adaptation (Anlernung zurücksetzen)
    /// Resets the DPF model adaptation values
    pub const RESET_LEARNED_VALUES: u16 = 0xA092;

    /// Register new DPF installed (Neuen DPF anlernen)
    /// Call this after installing a new DPF
    pub const NEW_DPF_INSTALLED: u16 = 0xA093;

    /// Start forced DPF regeneration (Zwangsregeneration starten)
    /// WARNING: Vehicle must be stationary, engine running
    pub const START_FORCED_REGEN: u16 = 0xA094;

    /// Stop forced DPF regeneration
    pub const STOP_FORCED_REGEN: u16 = 0xA095;

    /// Alternative routine IDs (some DDE versions use these)
    pub mod alt {
        pub const RESET_SOOT: u16 = 0x0060;
        pub const RESET_ASH: u16 = 0x0061;
        pub const RESET_ADAPTATION: u16 = 0x0062;
        pub const NEW_DPF: u16 = 0x0063;
        pub const FORCED_REGEN: u16 = 0x0064;
    }
}

/// DPF-related Data Identifiers for ReadDataByIdentifier (0x22)
pub mod dpf_dids {
    /// DPF soot loading percentage (0-100%)
    pub const SOOT_LOADING: u16 = 0xAB10;

    /// DPF ash loading in grams
    pub const ASH_LOADING: u16 = 0xAB11;

    /// DPF differential pressure (mbar)
    pub const DIFFERENTIAL_PRESSURE: u16 = 0xAB12;

    /// Exhaust gas temperature before DPF (°C)
    pub const TEMP_BEFORE_DPF: u16 = 0xAB13;

    /// Exhaust gas temperature after DPF (°C)
    pub const TEMP_AFTER_DPF: u16 = 0xAB14;

    /// Distance since last regeneration (km)
    pub const DISTANCE_SINCE_REGEN: u16 = 0xAB15;

    /// Total regeneration count
    pub const REGEN_COUNT: u16 = 0xAB16;

    /// DPF regeneration status (0=inactive, 1=active)
    pub const REGEN_STATUS: u16 = 0xAB17;

    /// Time since last regeneration (minutes)
    pub const TIME_SINCE_REGEN: u16 = 0xAB18;

    /// DPF oil ash mass (grams)
    pub const OIL_ASH_MASS: u16 = 0xAB19;

    /// Total distance with DPF (km)
    pub const TOTAL_DPF_DISTANCE: u16 = 0xAB1A;
}

/// Diagnostic session types
pub mod session {
    pub const DEFAULT: u8 = 0x01;
    pub const PROGRAMMING: u8 = 0x02;
    pub const EXTENDED: u8 = 0x03;
    pub const SAFETY_SYSTEM: u8 = 0x04;

    /// BMW-specific extended diagnostic session
    pub const BMW_EXTENDED: u8 = 0x86;
}

/// Security access levels for BMW
pub mod security {
    /// Standard diagnostic level
    pub const LEVEL_STANDARD: u8 = 0x01;
    /// Programming level
    pub const LEVEL_PROGRAMMING: u8 = 0x03;
    /// Development level (usually locked)
    pub const LEVEL_DEVELOPMENT: u8 = 0x11;

    /// Simple seed-key algorithm for standard level
    /// Note: Real BMW security uses more complex algorithms
    pub fn calculate_key_simple(seed: &[u8]) -> Vec<u8> {
        // Simple XOR-based key calculation (for demonstration)
        // Real BMW ECUs use proprietary algorithms
        seed.iter().map(|&b| b ^ 0xCA).collect()
    }
}

/// RoutineControl sub-functions
pub mod routine {
    pub const START: u8 = 0x01;
    pub const STOP: u8 = 0x02;
    pub const REQUEST_RESULTS: u8 = 0x03;
}

/// DPF routine result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DpfRoutineResult {
    pub success: bool,
    pub routine_id: u16,
    pub status: String,
    pub data: Vec<u8>,
}

/// DPF status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DpfStatus {
    pub soot_loading_percent: Option<f32>,
    pub ash_loading_grams: Option<f32>,
    pub differential_pressure_mbar: Option<f32>,
    pub temp_before_dpf: Option<f32>,
    pub temp_after_dpf: Option<f32>,
    pub distance_since_regen_km: Option<f32>,
    pub regen_count: Option<u32>,
    pub regen_active: bool,
}
