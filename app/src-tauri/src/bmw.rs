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

// ============================================================================
// DIESEL-SPECIFIC PIDs for BMW E60 520d (M47N2/N47)
// ============================================================================

/// Diesel-specific Data Identifiers (DIDs) for ReadDataByIdentifier (0x22)
/// These are BMW manufacturer-specific DIDs for the DDE (Digital Diesel Electronics)
pub mod diesel_dids {
    // === COMMON RAIL / FUEL SYSTEM ===

    /// Fuel rail pressure actual (bar) - Formula: (A*256+B) * 0.1
    pub const FUEL_RAIL_PRESSURE: u16 = 0x394A;

    /// Fuel rail pressure desired/target (bar)
    pub const FUEL_RAIL_PRESSURE_DESIRED: u16 = 0x394B;

    /// Fuel injection quantity total (mg/stroke) - Formula: (A*256+B) * 0.01
    pub const INJECTION_QUANTITY: u16 = 0x394C;

    /// Fuel injection quantity pilot (mg/stroke)
    pub const INJECTION_QUANTITY_PILOT: u16 = 0x394D;

    /// Fuel injection timing (°BTDC)
    pub const INJECTION_TIMING: u16 = 0x394E;

    // === INJECTOR CORRECTIONS ===

    /// Injector correction cylinder 1 (mg) - deviation from nominal
    pub const INJECTOR_CORRECTION_CYL1: u16 = 0x3950;
    /// Injector correction cylinder 2 (mg)
    pub const INJECTOR_CORRECTION_CYL2: u16 = 0x3951;
    /// Injector correction cylinder 3 (mg)
    pub const INJECTOR_CORRECTION_CYL3: u16 = 0x3952;
    /// Injector correction cylinder 4 (mg)
    pub const INJECTOR_CORRECTION_CYL4: u16 = 0x3953;

    /// Injector IMA code cylinder 1-4 (calibration values)
    pub const INJECTOR_IMA_CYL1: u16 = 0x3954;
    pub const INJECTOR_IMA_CYL2: u16 = 0x3955;
    pub const INJECTOR_IMA_CYL3: u16 = 0x3956;
    pub const INJECTOR_IMA_CYL4: u16 = 0x3957;

    // === EGR (Exhaust Gas Recirculation) ===

    /// EGR valve position actual (%) - Formula: A * 100 / 255
    pub const EGR_POSITION_ACTUAL: u16 = 0x3960;
    /// EGR valve position desired (%)
    pub const EGR_POSITION_DESIRED: u16 = 0x3961;
    /// EGR mass flow actual (kg/h)
    pub const EGR_MASS_FLOW: u16 = 0x3962;
    /// EGR cooler bypass position (%)
    pub const EGR_COOLER_BYPASS: u16 = 0x3963;

    // === TURBOCHARGER ===

    /// Boost pressure actual (mbar) - Formula: (A*256+B)
    pub const BOOST_PRESSURE_ACTUAL: u16 = 0x3970;
    /// Boost pressure desired (mbar)
    pub const BOOST_PRESSURE_DESIRED: u16 = 0x3971;
    /// VNT/VGT actuator position actual (%)
    pub const VNT_POSITION_ACTUAL: u16 = 0x3972;
    /// VNT/VGT actuator position desired (%)
    pub const VNT_POSITION_DESIRED: u16 = 0x3973;
    /// Wastegate duty cycle (%)
    pub const WASTEGATE_DUTY: u16 = 0x3974;
    /// Charge air pressure after intercooler (mbar)
    pub const CHARGE_AIR_PRESSURE: u16 = 0x3975;

    // === AIR SYSTEM ===

    /// Mass air flow (kg/h) - Formula: (A*256+B) * 0.1
    pub const AIR_MASS_FLOW: u16 = 0x3980;
    /// Air mass per stroke (mg)
    pub const AIR_MASS_PER_STROKE: u16 = 0x3981;
    /// Intake manifold pressure (mbar)
    pub const INTAKE_MANIFOLD_PRESSURE: u16 = 0x3982;
    /// Atmospheric pressure (mbar)
    pub const ATMOSPHERIC_PRESSURE: u16 = 0x3983;
    /// Swirl flap position (%)
    pub const SWIRL_FLAP_POSITION: u16 = 0x3984;
    /// Throttle valve position (%)
    pub const THROTTLE_POSITION_DIESEL: u16 = 0x3985;

    // === EXHAUST TEMPERATURES ===

    /// Exhaust gas temperature pre-turbo (°C) - Formula: (A*256+B) * 0.1 - 40
    pub const EXHAUST_TEMP_PRE_TURBO: u16 = 0x3990;
    /// Exhaust gas temperature post-turbo (°C)
    pub const EXHAUST_TEMP_POST_TURBO: u16 = 0x3991;
    /// Exhaust gas temperature DPF inlet (°C)
    pub const EXHAUST_TEMP_DPF_INLET: u16 = 0x3992;
    /// Exhaust gas temperature DPF outlet (°C)
    pub const EXHAUST_TEMP_DPF_OUTLET: u16 = 0x3993;
    /// Exhaust gas temperature pre-cat (°C)
    pub const EXHAUST_TEMP_PRE_CAT: u16 = 0x3994;

    // === DPF (Diesel Particulate Filter) ===

    /// DPF differential pressure (mbar)
    pub const DPF_DIFFERENTIAL_PRESSURE: u16 = 0x39A0;
    /// DPF soot loading calculated (%)
    pub const DPF_SOOT_LOADING: u16 = 0x39A1;
    /// DPF ash loading (grams)
    pub const DPF_ASH_LOADING: u16 = 0x39A2;
    /// DPF regeneration status (0=off, 1=active)
    pub const DPF_REGEN_STATUS: u16 = 0x39A3;
    /// Distance since last DPF regeneration (km)
    pub const DPF_DISTANCE_SINCE_REGEN: u16 = 0x39A4;
    /// Total DPF regeneration count
    pub const DPF_REGEN_COUNT: u16 = 0x39A5;
    /// DPF average regeneration interval (km)
    pub const DPF_AVG_REGEN_INTERVAL: u16 = 0x39A6;
    /// Time since last regeneration (minutes)
    pub const DPF_TIME_SINCE_REGEN: u16 = 0x39A7;

    // === GLOW PLUGS ===

    /// Glow plug status (bitmask: bit0=cyl1, bit1=cyl2, etc)
    pub const GLOW_PLUG_STATUS: u16 = 0x39B0;
    /// Glow plug on-time remaining (seconds)
    pub const GLOW_PLUG_TIME_REMAINING: u16 = 0x39B1;
    /// Glow plug current cylinder 1 (A)
    pub const GLOW_PLUG_CURRENT_CYL1: u16 = 0x39B2;
    /// Glow plug current cylinder 2 (A)
    pub const GLOW_PLUG_CURRENT_CYL2: u16 = 0x39B3;
    /// Glow plug current cylinder 3 (A)
    pub const GLOW_PLUG_CURRENT_CYL3: u16 = 0x39B4;
    /// Glow plug current cylinder 4 (A)
    pub const GLOW_PLUG_CURRENT_CYL4: u16 = 0x39B5;

    // === PEDAL / DRIVER DEMAND ===

    /// Accelerator pedal position 1 (%)
    pub const ACCELERATOR_PEDAL_POS1: u16 = 0x39C0;
    /// Accelerator pedal position 2 (redundant sensor) (%)
    pub const ACCELERATOR_PEDAL_POS2: u16 = 0x39C1;
    /// Driver torque demand (Nm)
    pub const DRIVER_TORQUE_DEMAND: u16 = 0x39C2;
    /// Actual engine torque (Nm)
    pub const ENGINE_TORQUE_ACTUAL: u16 = 0x39C3;
    /// Engine load (%)
    pub const ENGINE_LOAD: u16 = 0x39C4;

    // === ELECTRICAL / MISC ===

    /// Battery voltage (V) - Formula: (A*256+B) * 0.001
    pub const BATTERY_VOLTAGE: u16 = 0x39D0;
    /// Alternator load (%)
    pub const ALTERNATOR_LOAD: u16 = 0x39D1;
    /// Fuel temperature (°C)
    pub const FUEL_TEMPERATURE: u16 = 0x39D2;
    /// Coolant temperature (°C)
    pub const COOLANT_TEMPERATURE: u16 = 0x39D3;
    /// Oil temperature (°C)
    pub const OIL_TEMPERATURE: u16 = 0x39D4;
    /// Oil pressure (bar)
    pub const OIL_PRESSURE: u16 = 0x39D5;

    // === ENGINE OPERATION ===

    /// Engine speed (RPM)
    pub const ENGINE_RPM: u16 = 0x39E0;
    /// Vehicle speed from ECU (km/h)
    pub const VEHICLE_SPEED: u16 = 0x39E1;
    /// Engine running time (seconds)
    pub const ENGINE_RUNTIME: u16 = 0x39E2;
    /// Total fuel consumption (liters)
    pub const TOTAL_FUEL_CONSUMPTION: u16 = 0x39E3;
    /// Instant fuel consumption (L/h)
    pub const INSTANT_FUEL_CONSUMPTION: u16 = 0x39E4;
    /// Distance to empty (km)
    pub const DISTANCE_TO_EMPTY: u16 = 0x39E5;
}

/// DID value result from ReadDataByIdentifier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DidValue {
    pub did: u16,
    pub name: String,
    pub value: f64,
    pub unit: String,
    pub raw: Vec<u8>,
    pub timestamp: u64,
}

/// Diesel live data status (comprehensive snapshot)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DieselLiveData {
    // Fuel system
    pub rail_pressure_actual: Option<f64>,
    pub rail_pressure_desired: Option<f64>,
    pub injection_quantity: Option<f64>,

    // Turbo
    pub boost_pressure_actual: Option<f64>,
    pub boost_pressure_desired: Option<f64>,
    pub vnt_position: Option<f64>,

    // EGR
    pub egr_position_actual: Option<f64>,
    pub egr_position_desired: Option<f64>,

    // Exhaust temps
    pub exhaust_temp_pre_turbo: Option<f64>,
    pub exhaust_temp_dpf_inlet: Option<f64>,
    pub exhaust_temp_dpf_outlet: Option<f64>,

    // DPF
    pub dpf_soot_loading: Option<f64>,
    pub dpf_differential_pressure: Option<f64>,
    pub dpf_regen_active: bool,

    // Engine
    pub engine_rpm: Option<f64>,
    pub coolant_temp: Option<f64>,
    pub oil_temp: Option<f64>,
    pub battery_voltage: Option<f64>,

    // Corrections
    pub injector_corrections: Vec<f64>,

    pub timestamp: u64,
}

/// Categories for diesel PIDs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DieselPidCategory {
    FuelSystem,
    Turbo,
    Egr,
    Temperatures,
    Dpf,
    GlowPlugs,
    Engine,
    Electrical,
}

/// Full diesel PID definition with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DieselPidDefinition {
    pub did: u16,
    pub name: String,
    pub short_name: String,
    pub description: String,
    pub unit: String,
    pub min: f64,
    pub max: f64,
    pub category: String,
    pub formula: String,
    pub warning_low: Option<f64>,
    pub warning_high: Option<f64>,
    pub critical_low: Option<f64>,
    pub critical_high: Option<f64>,
}

/// Get all diesel PID definitions for E60 520d
pub fn get_diesel_pid_definitions() -> Vec<DieselPidDefinition> {
    vec![
        // === FUEL SYSTEM ===
        DieselPidDefinition {
            did: diesel_dids::FUEL_RAIL_PRESSURE,
            name: "Presion Rail Combustible".to_string(),
            short_name: "Rail".to_string(),
            description: "Presion del common rail en bar".to_string(),
            unit: "bar".to_string(),
            min: 0.0,
            max: 2000.0,
            category: "fuel_system".to_string(),
            formula: "(A*256+B) * 0.1".to_string(),
            warning_low: Some(200.0),
            warning_high: Some(1800.0),
            critical_low: Some(150.0),
            critical_high: Some(1900.0),
        },
        DieselPidDefinition {
            did: diesel_dids::FUEL_RAIL_PRESSURE_DESIRED,
            name: "Presion Rail Deseada".to_string(),
            short_name: "Rail Obj".to_string(),
            description: "Presion objetivo del rail".to_string(),
            unit: "bar".to_string(),
            min: 0.0,
            max: 2000.0,
            category: "fuel_system".to_string(),
            formula: "(A*256+B) * 0.1".to_string(),
            warning_low: None,
            warning_high: None,
            critical_low: None,
            critical_high: None,
        },
        DieselPidDefinition {
            did: diesel_dids::INJECTION_QUANTITY,
            name: "Caudal Inyeccion".to_string(),
            short_name: "Inyeccion".to_string(),
            description: "Cantidad de combustible inyectado por carrera".to_string(),
            unit: "mg/str".to_string(),
            min: 0.0,
            max: 100.0,
            category: "fuel_system".to_string(),
            formula: "(A*256+B) * 0.01".to_string(),
            warning_low: None,
            warning_high: Some(80.0),
            critical_low: None,
            critical_high: Some(90.0),
        },
        DieselPidDefinition {
            did: diesel_dids::INJECTOR_CORRECTION_CYL1,
            name: "Correccion Inyector Cil.1".to_string(),
            short_name: "Inj1".to_string(),
            description: "Desviacion del inyector cilindro 1".to_string(),
            unit: "mg".to_string(),
            min: -5.0,
            max: 5.0,
            category: "fuel_system".to_string(),
            formula: "(A-128) * 0.1".to_string(),
            warning_low: Some(-3.0),
            warning_high: Some(3.0),
            critical_low: Some(-4.0),
            critical_high: Some(4.0),
        },
        DieselPidDefinition {
            did: diesel_dids::INJECTOR_CORRECTION_CYL2,
            name: "Correccion Inyector Cil.2".to_string(),
            short_name: "Inj2".to_string(),
            description: "Desviacion del inyector cilindro 2".to_string(),
            unit: "mg".to_string(),
            min: -5.0,
            max: 5.0,
            category: "fuel_system".to_string(),
            formula: "(A-128) * 0.1".to_string(),
            warning_low: Some(-3.0),
            warning_high: Some(3.0),
            critical_low: Some(-4.0),
            critical_high: Some(4.0),
        },
        DieselPidDefinition {
            did: diesel_dids::INJECTOR_CORRECTION_CYL3,
            name: "Correccion Inyector Cil.3".to_string(),
            short_name: "Inj3".to_string(),
            description: "Desviacion del inyector cilindro 3".to_string(),
            unit: "mg".to_string(),
            min: -5.0,
            max: 5.0,
            category: "fuel_system".to_string(),
            formula: "(A-128) * 0.1".to_string(),
            warning_low: Some(-3.0),
            warning_high: Some(3.0),
            critical_low: Some(-4.0),
            critical_high: Some(4.0),
        },
        DieselPidDefinition {
            did: diesel_dids::INJECTOR_CORRECTION_CYL4,
            name: "Correccion Inyector Cil.4".to_string(),
            short_name: "Inj4".to_string(),
            description: "Desviacion del inyector cilindro 4".to_string(),
            unit: "mg".to_string(),
            min: -5.0,
            max: 5.0,
            category: "fuel_system".to_string(),
            formula: "(A-128) * 0.1".to_string(),
            warning_low: Some(-3.0),
            warning_high: Some(3.0),
            critical_low: Some(-4.0),
            critical_high: Some(4.0),
        },

        // === TURBO ===
        DieselPidDefinition {
            did: diesel_dids::BOOST_PRESSURE_ACTUAL,
            name: "Presion Turbo Actual".to_string(),
            short_name: "Boost".to_string(),
            description: "Presion de sobrealimentacion actual".to_string(),
            unit: "mbar".to_string(),
            min: 0.0,
            max: 2500.0,
            category: "turbo".to_string(),
            formula: "A*256+B".to_string(),
            warning_low: None,
            warning_high: Some(2200.0),
            critical_low: None,
            critical_high: Some(2400.0),
        },
        DieselPidDefinition {
            did: diesel_dids::BOOST_PRESSURE_DESIRED,
            name: "Presion Turbo Objetivo".to_string(),
            short_name: "Boost Obj".to_string(),
            description: "Presion de sobrealimentacion deseada".to_string(),
            unit: "mbar".to_string(),
            min: 0.0,
            max: 2500.0,
            category: "turbo".to_string(),
            formula: "A*256+B".to_string(),
            warning_low: None,
            warning_high: None,
            critical_low: None,
            critical_high: None,
        },
        DieselPidDefinition {
            did: diesel_dids::VNT_POSITION_ACTUAL,
            name: "Posicion VNT Actual".to_string(),
            short_name: "VNT".to_string(),
            description: "Posicion del actuador de geometria variable".to_string(),
            unit: "%".to_string(),
            min: 0.0,
            max: 100.0,
            category: "turbo".to_string(),
            formula: "A * 100 / 255".to_string(),
            warning_low: None,
            warning_high: None,
            critical_low: None,
            critical_high: None,
        },

        // === EGR ===
        DieselPidDefinition {
            did: diesel_dids::EGR_POSITION_ACTUAL,
            name: "Posicion EGR Actual".to_string(),
            short_name: "EGR".to_string(),
            description: "Apertura de la valvula EGR".to_string(),
            unit: "%".to_string(),
            min: 0.0,
            max: 100.0,
            category: "egr".to_string(),
            formula: "A * 100 / 255".to_string(),
            warning_low: None,
            warning_high: None,
            critical_low: None,
            critical_high: None,
        },
        DieselPidDefinition {
            did: diesel_dids::EGR_POSITION_DESIRED,
            name: "Posicion EGR Objetivo".to_string(),
            short_name: "EGR Obj".to_string(),
            description: "Posicion objetivo de la valvula EGR".to_string(),
            unit: "%".to_string(),
            min: 0.0,
            max: 100.0,
            category: "egr".to_string(),
            formula: "A * 100 / 255".to_string(),
            warning_low: None,
            warning_high: None,
            critical_low: None,
            critical_high: None,
        },
        DieselPidDefinition {
            did: diesel_dids::EGR_MASS_FLOW,
            name: "Caudal Masico EGR".to_string(),
            short_name: "EGR Flow".to_string(),
            description: "Flujo de gases recirculados".to_string(),
            unit: "kg/h".to_string(),
            min: 0.0,
            max: 500.0,
            category: "egr".to_string(),
            formula: "(A*256+B) * 0.1".to_string(),
            warning_low: None,
            warning_high: None,
            critical_low: None,
            critical_high: None,
        },

        // === TEMPERATURAS ESCAPE ===
        DieselPidDefinition {
            did: diesel_dids::EXHAUST_TEMP_PRE_TURBO,
            name: "Temp Escape Pre-Turbo".to_string(),
            short_name: "T.PreTurbo".to_string(),
            description: "Temperatura gases antes del turbo".to_string(),
            unit: "°C".to_string(),
            min: -40.0,
            max: 900.0,
            category: "temperatures".to_string(),
            formula: "(A*256+B) * 0.1 - 40".to_string(),
            warning_low: None,
            warning_high: Some(750.0),
            critical_low: None,
            critical_high: Some(850.0),
        },
        DieselPidDefinition {
            did: diesel_dids::EXHAUST_TEMP_DPF_INLET,
            name: "Temp Entrada DPF".to_string(),
            short_name: "T.DPF In".to_string(),
            description: "Temperatura gases entrada filtro particulas".to_string(),
            unit: "°C".to_string(),
            min: -40.0,
            max: 800.0,
            category: "temperatures".to_string(),
            formula: "(A*256+B) * 0.1 - 40".to_string(),
            warning_low: None,
            warning_high: Some(650.0),
            critical_low: None,
            critical_high: Some(700.0),
        },
        DieselPidDefinition {
            did: diesel_dids::EXHAUST_TEMP_DPF_OUTLET,
            name: "Temp Salida DPF".to_string(),
            short_name: "T.DPF Out".to_string(),
            description: "Temperatura gases salida filtro particulas".to_string(),
            unit: "°C".to_string(),
            min: -40.0,
            max: 800.0,
            category: "temperatures".to_string(),
            formula: "(A*256+B) * 0.1 - 40".to_string(),
            warning_low: None,
            warning_high: Some(600.0),
            critical_low: None,
            critical_high: Some(650.0),
        },

        // === DPF ===
        DieselPidDefinition {
            did: diesel_dids::DPF_SOOT_LOADING,
            name: "Carga Hollin DPF".to_string(),
            short_name: "Hollin".to_string(),
            description: "Porcentaje de carga de hollin en el DPF".to_string(),
            unit: "%".to_string(),
            min: 0.0,
            max: 100.0,
            category: "dpf".to_string(),
            formula: "A * 100 / 255".to_string(),
            warning_low: None,
            warning_high: Some(70.0),
            critical_low: None,
            critical_high: Some(85.0),
        },
        DieselPidDefinition {
            did: diesel_dids::DPF_DIFFERENTIAL_PRESSURE,
            name: "Presion Diferencial DPF".to_string(),
            short_name: "dP DPF".to_string(),
            description: "Diferencia de presion a traves del DPF".to_string(),
            unit: "mbar".to_string(),
            min: 0.0,
            max: 500.0,
            category: "dpf".to_string(),
            formula: "A*256+B".to_string(),
            warning_low: None,
            warning_high: Some(300.0),
            critical_low: None,
            critical_high: Some(400.0),
        },
        DieselPidDefinition {
            did: diesel_dids::DPF_ASH_LOADING,
            name: "Carga Cenizas DPF".to_string(),
            short_name: "Cenizas".to_string(),
            description: "Gramos de ceniza acumulada en el DPF".to_string(),
            unit: "g".to_string(),
            min: 0.0,
            max: 200.0,
            category: "dpf".to_string(),
            formula: "(A*256+B) * 0.1".to_string(),
            warning_low: None,
            warning_high: Some(100.0),
            critical_low: None,
            critical_high: Some(150.0),
        },
        DieselPidDefinition {
            did: diesel_dids::DPF_DISTANCE_SINCE_REGEN,
            name: "Distancia desde Regen".to_string(),
            short_name: "Km Regen".to_string(),
            description: "Kilometros recorridos desde ultima regeneracion".to_string(),
            unit: "km".to_string(),
            min: 0.0,
            max: 1000.0,
            category: "dpf".to_string(),
            formula: "A*256+B".to_string(),
            warning_low: None,
            warning_high: Some(500.0),
            critical_low: None,
            critical_high: Some(700.0),
        },
        DieselPidDefinition {
            did: diesel_dids::DPF_REGEN_COUNT,
            name: "Contador Regeneraciones".to_string(),
            short_name: "Regens".to_string(),
            description: "Numero total de regeneraciones realizadas".to_string(),
            unit: "".to_string(),
            min: 0.0,
            max: 10000.0,
            category: "dpf".to_string(),
            formula: "A*256+B".to_string(),
            warning_low: None,
            warning_high: None,
            critical_low: None,
            critical_high: None,
        },

        // === BUJIAS CALENTAMIENTO ===
        DieselPidDefinition {
            did: diesel_dids::GLOW_PLUG_STATUS,
            name: "Estado Bujias".to_string(),
            short_name: "Bujias".to_string(),
            description: "Estado de las bujias de calentamiento (bitmask)".to_string(),
            unit: "".to_string(),
            min: 0.0,
            max: 255.0,
            category: "glow_plugs".to_string(),
            formula: "A".to_string(),
            warning_low: None,
            warning_high: None,
            critical_low: None,
            critical_high: None,
        },

        // === MOTOR ===
        DieselPidDefinition {
            did: diesel_dids::ENGINE_RPM,
            name: "RPM Motor".to_string(),
            short_name: "RPM".to_string(),
            description: "Velocidad del motor".to_string(),
            unit: "rpm".to_string(),
            min: 0.0,
            max: 6000.0,
            category: "engine".to_string(),
            formula: "A*256+B".to_string(),
            warning_low: None,
            warning_high: Some(5000.0),
            critical_low: None,
            critical_high: Some(5500.0),
        },
        DieselPidDefinition {
            did: diesel_dids::ENGINE_LOAD,
            name: "Carga Motor".to_string(),
            short_name: "Carga".to_string(),
            description: "Porcentaje de carga del motor".to_string(),
            unit: "%".to_string(),
            min: 0.0,
            max: 100.0,
            category: "engine".to_string(),
            formula: "A * 100 / 255".to_string(),
            warning_low: None,
            warning_high: None,
            critical_low: None,
            critical_high: None,
        },
        DieselPidDefinition {
            did: diesel_dids::ACCELERATOR_PEDAL_POS1,
            name: "Posicion Acelerador".to_string(),
            short_name: "Acel".to_string(),
            description: "Posicion del pedal acelerador".to_string(),
            unit: "%".to_string(),
            min: 0.0,
            max: 100.0,
            category: "engine".to_string(),
            formula: "A * 100 / 255".to_string(),
            warning_low: None,
            warning_high: None,
            critical_low: None,
            critical_high: None,
        },

        // === ELECTRICO ===
        DieselPidDefinition {
            did: diesel_dids::BATTERY_VOLTAGE,
            name: "Tension Bateria".to_string(),
            short_name: "Bateria".to_string(),
            description: "Voltaje de la bateria".to_string(),
            unit: "V".to_string(),
            min: 0.0,
            max: 20.0,
            category: "electrical".to_string(),
            formula: "(A*256+B) * 0.001".to_string(),
            warning_low: Some(11.5),
            warning_high: Some(15.0),
            critical_low: Some(10.5),
            critical_high: Some(16.0),
        },
        DieselPidDefinition {
            did: diesel_dids::COOLANT_TEMPERATURE,
            name: "Temperatura Refrigerante".to_string(),
            short_name: "T.Refrig".to_string(),
            description: "Temperatura del liquido refrigerante".to_string(),
            unit: "°C".to_string(),
            min: -40.0,
            max: 150.0,
            category: "engine".to_string(),
            formula: "A - 40".to_string(),
            warning_low: Some(60.0),
            warning_high: Some(105.0),
            critical_low: Some(40.0),
            critical_high: Some(115.0),
        },
        DieselPidDefinition {
            did: diesel_dids::OIL_TEMPERATURE,
            name: "Temperatura Aceite".to_string(),
            short_name: "T.Aceite".to_string(),
            description: "Temperatura del aceite motor".to_string(),
            unit: "°C".to_string(),
            min: -40.0,
            max: 180.0,
            category: "engine".to_string(),
            formula: "A - 40".to_string(),
            warning_low: Some(60.0),
            warning_high: Some(130.0),
            critical_low: Some(40.0),
            critical_high: Some(150.0),
        },
        DieselPidDefinition {
            did: diesel_dids::FUEL_TEMPERATURE,
            name: "Temperatura Combustible".to_string(),
            short_name: "T.Fuel".to_string(),
            description: "Temperatura del diesel".to_string(),
            unit: "°C".to_string(),
            min: -40.0,
            max: 100.0,
            category: "fuel_system".to_string(),
            formula: "A - 40".to_string(),
            warning_low: None,
            warning_high: Some(60.0),
            critical_low: None,
            critical_high: Some(70.0),
        },
        DieselPidDefinition {
            did: diesel_dids::AIR_MASS_FLOW,
            name: "Caudal Masico Aire".to_string(),
            short_name: "MAF".to_string(),
            description: "Flujo de aire medido por el sensor MAF".to_string(),
            unit: "kg/h".to_string(),
            min: 0.0,
            max: 1000.0,
            category: "engine".to_string(),
            formula: "(A*256+B) * 0.1".to_string(),
            warning_low: None,
            warning_high: None,
            critical_low: None,
            critical_high: None,
        },
    ]
}

/// Calculate value from raw DID response bytes
pub fn calculate_diesel_did_value(did: u16, data: &[u8]) -> Option<(f64, String, String)> {
    if data.is_empty() {
        return None;
    }

    let a = data[0] as f64;
    let b = data.get(1).copied().unwrap_or(0) as f64;
    let ab = a * 256.0 + b;

    let (value, unit, name) = match did {
        // Fuel rail pressure (bar)
        0x394A => (ab * 0.1, "bar".to_string(), "Rail Pressure".to_string()),
        0x394B => (ab * 0.1, "bar".to_string(), "Rail Pressure Desired".to_string()),

        // Injection quantity (mg/stroke)
        0x394C => (ab * 0.01, "mg/str".to_string(), "Injection Qty".to_string()),
        0x394D => (ab * 0.01, "mg/str".to_string(), "Pilot Injection".to_string()),

        // Injector corrections (signed, mg)
        0x3950..=0x3953 => {
            let cyl = (did - 0x394F) as u8;
            ((a - 128.0) * 0.1, "mg".to_string(), format!("Inj Corr Cyl{}", cyl))
        },

        // EGR position (%)
        0x3960 => (a * 100.0 / 255.0, "%".to_string(), "EGR Position".to_string()),
        0x3961 => (a * 100.0 / 255.0, "%".to_string(), "EGR Desired".to_string()),
        0x3962 => (ab * 0.1, "kg/h".to_string(), "EGR Mass Flow".to_string()),

        // Boost pressure (mbar)
        0x3970 => (ab, "mbar".to_string(), "Boost Actual".to_string()),
        0x3971 => (ab, "mbar".to_string(), "Boost Desired".to_string()),

        // VNT position (%)
        0x3972 => (a * 100.0 / 255.0, "%".to_string(), "VNT Position".to_string()),
        0x3973 => (a * 100.0 / 255.0, "%".to_string(), "VNT Desired".to_string()),

        // Air mass (kg/h)
        0x3980 => (ab * 0.1, "kg/h".to_string(), "Air Mass Flow".to_string()),

        // Exhaust temperatures (°C)
        0x3990..=0x3994 => {
            let names = ["Pre-Turbo", "Post-Turbo", "DPF Inlet", "DPF Outlet", "Pre-Cat"];
            let idx = (did - 0x3990) as usize;
            (ab * 0.1 - 40.0, "°C".to_string(), format!("Exhaust {}", names.get(idx).unwrap_or(&"Temp")))
        },

        // DPF values
        0x39A0 => (ab, "mbar".to_string(), "DPF Diff Pressure".to_string()),
        0x39A1 => (a * 100.0 / 255.0, "%".to_string(), "DPF Soot Loading".to_string()),
        0x39A2 => (ab * 0.1, "g".to_string(), "DPF Ash Loading".to_string()),
        0x39A3 => (a, "".to_string(), "DPF Regen Status".to_string()),
        0x39A4 => (ab, "km".to_string(), "Dist Since Regen".to_string()),
        0x39A5 => (ab, "".to_string(), "Regen Count".to_string()),

        // Glow plugs
        0x39B0 => (a, "".to_string(), "Glow Plug Status".to_string()),
        0x39B1 => (a, "s".to_string(), "Glow Time Remain".to_string()),

        // Pedal/load
        0x39C0 => (a * 100.0 / 255.0, "%".to_string(), "Accel Pedal".to_string()),
        0x39C4 => (a * 100.0 / 255.0, "%".to_string(), "Engine Load".to_string()),

        // Electrical
        0x39D0 => (ab * 0.001, "V".to_string(), "Battery Voltage".to_string()),
        0x39D2 => (a - 40.0, "°C".to_string(), "Fuel Temp".to_string()),
        0x39D3 => (a - 40.0, "°C".to_string(), "Coolant Temp".to_string()),
        0x39D4 => (a - 40.0, "°C".to_string(), "Oil Temp".to_string()),
        0x39D5 => (ab * 0.01, "bar".to_string(), "Oil Pressure".to_string()),

        // Engine
        0x39E0 => (ab, "rpm".to_string(), "Engine RPM".to_string()),
        0x39E1 => (a, "km/h".to_string(), "Vehicle Speed".to_string()),
        0x39E4 => (ab * 0.01, "L/h".to_string(), "Fuel Consumption".to_string()),

        // Unknown DID
        _ => (a, "raw".to_string(), format!("DID 0x{:04X}", did)),
    };

    Some((value, unit, name))
}
