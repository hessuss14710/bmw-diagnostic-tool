//! Centralized constants for BMW diagnostic communication
//!
//! This module contains all magic numbers, addresses, and protocol constants
//! used throughout the application.

// ============================================================================
// ECU ADDRESSES
// ============================================================================

/// Valid ECU addresses for K-Line/D-CAN communication
pub mod addresses {
    /// Diagnostic tester address (ISO 14230)
    pub const TESTER: u8 = 0xF1;

    /// Default target ECU (DME/DDE - Engine Control)
    pub const DEFAULT_ECU: u8 = 0x12;

    /// All valid ECU addresses
    pub const VALID_ECUS: &[u8] = &[
        DME_DDE, EGS, DSC, AIRBAG, IHKA, KOMBI, CAS, FRM, PDC, ACC,
    ];

    // Individual ECU addresses
    pub const DME_DDE: u8 = 0x12;  // Digital Motor Electronics / Digital Diesel Electronics
    pub const EGS: u8 = 0x32;      // Electronic Gearbox Control
    pub const DSC: u8 = 0x44;      // Dynamic Stability Control
    pub const AIRBAG: u8 = 0x4A;   // Airbag (MRS - Multiple Restraint System)
    pub const IHKA: u8 = 0x5B;     // Integrated Automatic Heating/Air Conditioning
    pub const KOMBI: u8 = 0x60;    // Instrument Cluster
    pub const CAS: u8 = 0x40;      // Car Access System
    pub const FRM: u8 = 0x68;      // Footwell Module (Lighting)
    pub const PDC: u8 = 0x66;      // Park Distance Control
    pub const ACC: u8 = 0x34;      // Active Cruise Control
}

// ============================================================================
// BAUD RATES
// ============================================================================

pub mod baud {
    /// K-Line standard baud rate (ISO 14230)
    pub const KLINE_DEFAULT: u32 = 10_400;

    /// K-Line fast init baud rate
    pub const KLINE_FAST: u32 = 10_400;

    /// D-CAN standard baud rate (500 kbaud)
    pub const DCAN_DEFAULT: u32 = 500_000;

    /// 5-baud init rate for slow init
    pub const FIVE_BAUD: u32 = 5;

    /// Valid baud rate range
    pub const MIN_BAUD: u32 = 5;
    pub const MAX_BAUD: u32 = 3_000_000;
}

// ============================================================================
// UDS SERVICES (ISO 14229)
// ============================================================================

pub mod uds {
    // Diagnostic and Communication Management
    pub const DIAGNOSTIC_SESSION_CONTROL: u8 = 0x10;
    pub const ECU_RESET: u8 = 0x11;
    pub const SECURITY_ACCESS: u8 = 0x27;
    pub const COMMUNICATION_CONTROL: u8 = 0x28;
    pub const TESTER_PRESENT: u8 = 0x3E;
    pub const CONTROL_DTC_SETTING: u8 = 0x85;

    // Data Transmission
    pub const READ_DATA_BY_ID: u8 = 0x22;
    pub const READ_MEMORY_BY_ADDRESS: u8 = 0x23;
    pub const WRITE_DATA_BY_ID: u8 = 0x2E;
    pub const WRITE_MEMORY_BY_ADDRESS: u8 = 0x3D;

    // DTC Management
    pub const CLEAR_DIAGNOSTIC_INFO: u8 = 0x14;
    pub const READ_DTC_INFO: u8 = 0x19;

    // Input/Output Control
    pub const IO_CONTROL: u8 = 0x2F;
    pub const ROUTINE_CONTROL: u8 = 0x31;

    // Upload/Download
    pub const REQUEST_DOWNLOAD: u8 = 0x34;
    pub const REQUEST_UPLOAD: u8 = 0x35;
    pub const TRANSFER_DATA: u8 = 0x36;
    pub const REQUEST_TRANSFER_EXIT: u8 = 0x37;

    // Response codes
    pub const POSITIVE_RESPONSE_OFFSET: u8 = 0x40;
    pub const NEGATIVE_RESPONSE: u8 = 0x7F;

    // Session types
    pub mod session {
        pub const DEFAULT: u8 = 0x01;
        pub const PROGRAMMING: u8 = 0x02;
        pub const EXTENDED: u8 = 0x03;
    }

    // DTC sub-functions
    pub mod dtc {
        pub const REPORT_BY_STATUS_MASK: u8 = 0x02;
        pub const REPORT_SNAPSHOT_BY_DTC: u8 = 0x04;
        pub const REPORT_SUPPORTED: u8 = 0x0A;
        pub const STATUS_MASK_ALL: u8 = 0xFF;
    }

    // Routine control sub-functions
    pub mod routine {
        pub const START: u8 = 0x01;
        pub const STOP: u8 = 0x02;
        pub const REQUEST_RESULTS: u8 = 0x03;
    }

    // Security access sub-functions
    pub mod security {
        pub const REQUEST_SEED_L1: u8 = 0x01;
        pub const SEND_KEY_L1: u8 = 0x02;
        pub const REQUEST_SEED_L2: u8 = 0x03;
        pub const SEND_KEY_L2: u8 = 0x04;
    }
}

// ============================================================================
// KWP2000 SERVICES (ISO 14230)
// ============================================================================

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
    pub const SECURITY_ACCESS: u8 = 0x27;
    pub const INPUT_OUTPUT_CONTROL: u8 = 0x30;
    pub const START_ROUTINE: u8 = 0x31;
    pub const STOP_ROUTINE: u8 = 0x32;
    pub const REQUEST_ROUTINE_RESULTS: u8 = 0x33;

    pub const POSITIVE_RESPONSE_OFFSET: u8 = 0x40;
    pub const NEGATIVE_RESPONSE: u8 = 0x7F;
}

// ============================================================================
// DPF ROUTINE IDS
// ============================================================================

pub mod dpf_routines {
    // Primary routine IDs (UDS format)
    pub const RESET_ASH_LOADING: u16 = 0xA091;
    pub const RESET_LEARNED_VALUES: u16 = 0xA092;
    pub const NEW_DPF_INSTALLED: u16 = 0xA093;
    pub const START_REGENERATION: u16 = 0xA094;

    // Alternative routine IDs (some ECUs use these)
    pub mod alt {
        pub const RESET_ASH: u16 = 0x0061;
        pub const RESET_ADAPTATION: u16 = 0x0062;
        pub const NEW_DPF: u16 = 0x0063;
        pub const REGENERATION: u16 = 0x0064;
    }

    /// All valid DPF routine IDs
    pub const VALID_ROUTINES: &[u16] = &[
        RESET_ASH_LOADING,
        RESET_LEARNED_VALUES,
        NEW_DPF_INSTALLED,
        START_REGENERATION,
        alt::RESET_ASH,
        alt::RESET_ADAPTATION,
        alt::NEW_DPF,
        alt::REGENERATION,
    ];
}

// ============================================================================
// TIMING CONSTANTS
// ============================================================================

pub mod timing {
    use std::time::Duration;

    /// P1 max - inter-byte time for ECU response (20ms)
    pub const P1_MAX_MS: u64 = 20;

    /// P2 max - time between request and response (50ms)
    pub const P2_MAX_MS: u64 = 50;

    /// P3 min - minimum time between responses and new request (55ms)
    pub const P3_MIN_MS: u64 = 55;

    /// P4 min - inter-byte time for tester request (5ms)
    pub const P4_MIN_MS: u64 = 5;

    /// Init timeout for K-Line initialization
    pub const INIT_TIMEOUT_MS: u64 = 400;

    /// Message receive timeout
    pub const MESSAGE_TIMEOUT_MS: u64 = 100;

    /// Tester present interval
    pub const TESTER_PRESENT_INTERVAL_MS: u64 = 2000;

    /// As Duration for convenience
    pub const P3_MIN: Duration = Duration::from_millis(P3_MIN_MS);
    pub const MESSAGE_TIMEOUT: Duration = Duration::from_millis(MESSAGE_TIMEOUT_MS);
    pub const INIT_TIMEOUT: Duration = Duration::from_millis(INIT_TIMEOUT_MS);
}

// ============================================================================
// LIMITS
// ============================================================================

pub mod limits {
    /// Maximum PIDs per single request
    pub const MAX_PIDS_PER_REQUEST: usize = 16;

    /// Maximum DIDs per single request
    pub const MAX_DIDS_PER_REQUEST: usize = 16;

    /// Maximum routine data size in bytes
    pub const MAX_ROUTINE_DATA_SIZE: usize = 256;

    /// Maximum hex string length for serial_send_hex
    pub const MAX_HEX_STRING_LENGTH: usize = 10_000;

    /// Maximum DTCs to read at once
    pub const MAX_DTCS_PER_READ: usize = 100;
}

// ============================================================================
// PID RANGES
// ============================================================================

pub mod pid_ranges {
    /// Standard OBD-II PIDs (Mode 01)
    pub const OBD2_STANDARD: (u16, u16) = (0x00, 0xFF);

    /// Extended manufacturer PIDs
    pub const EXTENDED: (u16, u16) = (0x0100, 0x01FF);

    /// BMW-specific DIDs
    pub const BMW_DID: (u16, u16) = (0x1000, 0xFFFF);

    /// All valid ranges
    pub const VALID_RANGES: &[(u16, u16)] = &[
        OBD2_STANDARD,
        EXTENDED,
        BMW_DID,
    ];

    /// Reserved/dangerous PIDs that should not be accessed
    pub const RESTRICTED: &[u16] = &[0x0000, 0xFFFF];
}

// ============================================================================
// DIESEL PID CATEGORIES
// ============================================================================

pub mod diesel_categories {
    pub const FUEL_SYSTEM: &str = "fuel_system";
    pub const TURBO: &str = "turbo";
    pub const EGR: &str = "egr";
    pub const TEMPERATURES: &str = "temperatures";
    pub const DPF: &str = "dpf";
    pub const GLOW_PLUGS: &str = "glow_plugs";
    pub const ENGINE: &str = "engine";
    pub const ELECTRICAL: &str = "electrical";

    pub const ALL_CATEGORIES: &[&str] = &[
        FUEL_SYSTEM,
        TURBO,
        EGR,
        TEMPERATURES,
        DPF,
        GLOW_PLUGS,
        ENGINE,
        ELECTRICAL,
    ];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_ecus_contains_default() {
        assert!(addresses::VALID_ECUS.contains(&addresses::DEFAULT_ECU));
    }

    #[test]
    fn test_baud_range() {
        assert!(baud::KLINE_DEFAULT >= baud::MIN_BAUD);
        assert!(baud::KLINE_DEFAULT <= baud::MAX_BAUD);
    }

    #[test]
    fn test_dpf_routines_valid() {
        assert!(dpf_routines::VALID_ROUTINES.contains(&dpf_routines::RESET_ASH_LOADING));
    }
}
