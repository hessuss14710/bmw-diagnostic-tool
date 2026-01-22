//! KWP2000 Protocol Message Handling
//!
//! Implements message building and parsing for ISO 14230 (KWP2000).

use tracing::debug;

/// KWP2000 message structure
#[derive(Debug, Clone)]
pub struct KwpMessage {
    pub source: u8,
    pub target: u8,
    pub data: Vec<u8>,
}

/// KWP2000 response structure
#[derive(Debug, Clone)]
pub struct KwpResponse {
    pub source: u8,
    pub target: u8,
    pub service: u8,
    pub data: Vec<u8>,
}

impl KwpMessage {
    /// Create a new KWP2000 message
    pub fn new(source: u8, target: u8, data: Vec<u8>) -> Self {
        Self { source, target, data }
    }

    /// Convert message to bytes for transmission
    ///
    /// Format: FMT TGT SRC [LEN] DATA... CHK
    ///
    /// FMT byte:
    /// - Bit 7: 1 = length in FMT byte
    /// - Bit 6: Address mode (0 = physical, 1 = functional)
    /// - Bits 5-0: Length (if bit 7 = 1)
    ///
    /// Note: KWP2000 single-frame messages support max 255 bytes of data.
    /// Longer data will be truncated with a warning.
    pub fn to_bytes(&self) -> Vec<u8> {
        let length = self.data.len();

        // KWP2000 single frame max is 255 bytes
        if length > 255 {
            debug!("WARNING: Data length {} exceeds KWP2000 max (255), truncating", length);
        }
        let effective_length = length.min(255);

        let mut bytes = Vec::with_capacity(effective_length + 5);

        if effective_length <= 63 {
            // Length in format byte
            let fmt = 0x80 | (effective_length as u8);
            bytes.push(fmt);
            bytes.push(self.target);
            bytes.push(self.source);
        } else {
            // Length in separate byte (format 0xC0 = with address, length in next byte)
            bytes.push(0xC0);
            bytes.push(self.target);
            bytes.push(self.source);
            bytes.push(effective_length as u8);
        }

        // Add data (truncate if necessary)
        bytes.extend_from_slice(&self.data[..effective_length]);

        // Calculate checksum (sum of all bytes mod 256)
        let checksum = bytes.iter().fold(0u8, |acc, &b| acc.wrapping_add(b));
        bytes.push(checksum);

        bytes
    }

    /// Create StartCommunication request (0x81)
    pub fn start_communication(source: u8, target: u8) -> Self {
        Self::new(source, target, vec![0x81])
    }

    /// Create StopCommunication request (0x82)
    pub fn stop_communication(source: u8, target: u8) -> Self {
        Self::new(source, target, vec![0x82])
    }

    /// Create TesterPresent request (0x3E)
    pub fn tester_present(source: u8, target: u8) -> Self {
        Self::new(source, target, vec![0x3E])
    }

    /// Create ReadDTCByStatus request (0x18)
    pub fn read_dtc(source: u8, target: u8, status_mask: u8) -> Self {
        Self::new(source, target, vec![0x18, 0x00, status_mask])
    }

    /// Create ClearDTC request (0x14)
    pub fn clear_dtc(source: u8, target: u8) -> Self {
        Self::new(source, target, vec![0x14, 0xFF, 0x00])
    }

    /// Create ReadDataByLocalId request (0x21)
    pub fn read_data_local(source: u8, target: u8, pid: u8) -> Self {
        Self::new(source, target, vec![0x21, pid])
    }
}

impl KwpResponse {
    /// Parse response from raw bytes
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 4 {
            debug!("Response too short: {} bytes", data.len());
            return None;
        }

        let fmt = data[0];
        let target = data[1];
        let source = data[2];

        let (data_length, data_start) = if fmt >= 0xC0 {
            // Format 0xC0-0xFF: Length in separate byte, with address
            if data.len() < 5 {
                debug!("Response too short for extended format");
                return None;
            }
            let len = data[3] as usize;
            (len, 4)
        } else if fmt >= 0x80 {
            // Format 0x80-0xBF: Length in format byte bits 0-5, with address
            let len = (fmt & 0x3F) as usize;
            (len, 3)
        } else {
            // Format 0x00-0x7F: Without address - not used in our protocol
            debug!("Unsupported format byte (no address): 0x{:02X}", fmt);
            return None;
        };

        let total_length = data_start + data_length + 1; // +1 for checksum

        if data.len() < total_length {
            debug!(
                "Response incomplete: expected {} bytes, got {}",
                total_length,
                data.len()
            );
            return None;
        }

        // Verify checksum
        let calc_checksum = data[..total_length - 1]
            .iter()
            .fold(0u8, |acc, &b| acc.wrapping_add(b));

        let recv_checksum = data[total_length - 1];

        if calc_checksum != recv_checksum {
            debug!(
                "Checksum mismatch: calculated 0x{:02X}, received 0x{:02X}",
                calc_checksum, recv_checksum
            );
            return None;
        }

        // Extract service ID and data
        let response_data = &data[data_start..data_start + data_length];

        if response_data.is_empty() {
            return None;
        }

        let service = response_data[0];
        let payload = response_data[1..].to_vec();

        Some(Self {
            source,
            target,
            service,
            data: payload,
        })
    }

    /// Check if this is a positive response
    pub fn is_positive(&self) -> bool {
        // Positive response = request service + 0x40
        self.service >= 0x40 && self.service != 0x7F
    }

    /// Check if this is a negative response
    pub fn is_negative(&self) -> bool {
        self.service == 0x7F
    }

    /// Get error code if negative response
    pub fn error_code(&self) -> Option<u8> {
        if self.is_negative() && self.data.len() >= 2 {
            Some(self.data[1])
        } else {
            None
        }
    }

    /// Get error description
    pub fn error_description(&self) -> Option<&'static str> {
        self.error_code().map(|code| match code {
            0x10 => "General reject",
            0x11 => "Service not supported",
            0x12 => "Sub-function not supported",
            0x13 => "Message length incorrect",
            0x14 => "Response too long",
            0x21 => "Busy - repeat request",
            0x22 => "Conditions not correct",
            0x23 => "Routine not complete",
            0x24 => "Request sequence error",
            0x25 => "No response from subnet",
            0x26 => "Failure prevents execution",
            0x31 => "Request out of range",
            0x33 => "Security access denied",
            0x35 => "Invalid key",
            0x36 => "Exceed number of attempts",
            0x37 => "Required time delay not expired",
            0x40 => "Download not accepted",
            0x41 => "Improper download type",
            0x42 => "Can not download to specified address",
            0x43 => "Can not download number of bytes requested",
            0x50 => "Upload not accepted",
            0x51 => "Improper upload type",
            0x52 => "Can not upload from specified address",
            0x53 => "Can not upload number of bytes requested",
            0x71 => "Transfer suspended",
            0x72 => "Transfer aborted",
            0x74 => "Illegal address in block transfer",
            0x75 => "Illegal byte count in block transfer",
            0x76 => "Illegal block transfer type",
            0x77 => "Block transfer data checksum error",
            0x78 => "Request correctly received, response pending",
            0x79 => "Incorrect byte count during block transfer",
            0x80 => "Service not supported in active diagnostic session",
            _ => "Unknown error",
        })
    }
}

/// KWP2000 Service IDs
#[allow(dead_code)]
pub mod services {
    // Diagnostic Management
    pub const START_DIAGNOSTIC_SESSION: u8 = 0x10;
    pub const ECU_RESET: u8 = 0x11;
    pub const CLEAR_DIAGNOSTIC_INFO: u8 = 0x14;
    pub const READ_DTC_BY_STATUS: u8 = 0x18;
    pub const READ_DTC_BY_NUMBER: u8 = 0x19;
    pub const READ_ECU_IDENTIFICATION: u8 = 0x1A;
    pub const STOP_DIAGNOSTIC_SESSION: u8 = 0x20;
    pub const READ_DATA_BY_LOCAL_ID: u8 = 0x21;
    pub const READ_DATA_BY_COMMON_ID: u8 = 0x22;
    pub const READ_MEMORY_BY_ADDRESS: u8 = 0x23;
    pub const SECURITY_ACCESS: u8 = 0x27;
    pub const DISABLE_NORMAL_MESSAGE_TX: u8 = 0x28;
    pub const ENABLE_NORMAL_MESSAGE_TX: u8 = 0x29;
    pub const DYNAMICALLY_DEFINE_LOCAL_ID: u8 = 0x2C;
    pub const WRITE_DATA_BY_COMMON_ID: u8 = 0x2E;
    pub const INPUT_OUTPUT_CONTROL: u8 = 0x30;
    pub const START_ROUTINE_BY_LOCAL_ID: u8 = 0x31;
    pub const STOP_ROUTINE_BY_LOCAL_ID: u8 = 0x32;
    pub const REQUEST_ROUTINE_RESULTS: u8 = 0x33;
    pub const REQUEST_DOWNLOAD: u8 = 0x34;
    pub const REQUEST_UPLOAD: u8 = 0x35;
    pub const TRANSFER_DATA: u8 = 0x36;
    pub const REQUEST_TRANSFER_EXIT: u8 = 0x37;
    pub const WRITE_DATA_BY_LOCAL_ID: u8 = 0x3B;
    pub const WRITE_MEMORY_BY_ADDRESS: u8 = 0x3D;
    pub const TESTER_PRESENT: u8 = 0x3E;

    // Communication Control
    pub const START_COMMUNICATION: u8 = 0x81;
    pub const STOP_COMMUNICATION: u8 = 0x82;
    pub const ACCESS_TIMING_PARAMETERS: u8 = 0x83;

    // Negative Response
    pub const NEGATIVE_RESPONSE: u8 = 0x7F;
}

/// Standard OBD-II PIDs (Service 0x01) - Work with all OBD-II vehicles
#[allow(dead_code)]
pub mod obd_pids {
    // Engine/Vehicle Status
    pub const MONITOR_STATUS: u8 = 0x01;           // Monitor status since DTCs cleared
    pub const FREEZE_DTC: u8 = 0x02;               // Freeze DTC
    pub const FUEL_SYSTEM_STATUS: u8 = 0x03;       // Fuel system status
    pub const ENGINE_LOAD: u8 = 0x04;              // Calculated engine load (%)
    pub const COOLANT_TEMP: u8 = 0x05;             // Engine coolant temperature (°C)

    // Fuel Trims
    pub const SHORT_FUEL_TRIM_B1: u8 = 0x06;       // Short term fuel trim Bank 1 (%)
    pub const LONG_FUEL_TRIM_B1: u8 = 0x07;        // Long term fuel trim Bank 1 (%)
    pub const SHORT_FUEL_TRIM_B2: u8 = 0x08;       // Short term fuel trim Bank 2 (%)
    pub const LONG_FUEL_TRIM_B2: u8 = 0x09;        // Long term fuel trim Bank 2 (%)

    // Fuel Pressure
    pub const FUEL_PRESSURE: u8 = 0x0A;            // Fuel pressure (kPa gauge)
    pub const INTAKE_MAP: u8 = 0x0B;               // Intake manifold absolute pressure (kPa)

    // Core Engine Data
    pub const ENGINE_RPM: u8 = 0x0C;               // Engine RPM
    pub const VEHICLE_SPEED: u8 = 0x0D;            // Vehicle speed (km/h)
    pub const TIMING_ADVANCE: u8 = 0x0E;           // Timing advance (° before TDC)
    pub const INTAKE_AIR_TEMP: u8 = 0x0F;          // Intake air temperature (°C)
    pub const MAF_RATE: u8 = 0x10;                 // MAF air flow rate (g/s)
    pub const THROTTLE_POSITION: u8 = 0x11;        // Throttle position (%)

    // Secondary Air Status
    pub const COMMANDED_SEC_AIR: u8 = 0x12;        // Commanded secondary air status
    pub const O2_SENSORS_PRESENT: u8 = 0x13;       // Oxygen sensors present (2 banks)

    // Oxygen Sensors
    pub const O2_B1S1_VOLTAGE: u8 = 0x14;          // O2 Bank 1, Sensor 1 voltage & trim
    pub const O2_B1S2_VOLTAGE: u8 = 0x15;          // O2 Bank 1, Sensor 2 voltage & trim
    pub const O2_B1S3_VOLTAGE: u8 = 0x16;          // O2 Bank 1, Sensor 3 voltage & trim
    pub const O2_B1S4_VOLTAGE: u8 = 0x17;          // O2 Bank 1, Sensor 4 voltage & trim
    pub const O2_B2S1_VOLTAGE: u8 = 0x18;          // O2 Bank 2, Sensor 1 voltage & trim
    pub const O2_B2S2_VOLTAGE: u8 = 0x19;          // O2 Bank 2, Sensor 2 voltage & trim

    // OBD Standards
    pub const OBD_STANDARDS: u8 = 0x1C;            // OBD standards this vehicle conforms to

    // Run Time
    pub const RUN_TIME: u8 = 0x1F;                 // Run time since engine start (seconds)

    // Distance
    pub const DISTANCE_WITH_MIL: u8 = 0x21;        // Distance traveled with MIL on (km)

    // Fuel Rail Pressure
    pub const FUEL_RAIL_PRESSURE_REL: u8 = 0x22;   // Fuel rail pressure relative to manifold
    pub const FUEL_RAIL_PRESSURE_DIRECT: u8 = 0x23;// Fuel rail gauge pressure (diesel/GDI)

    // Wide-band O2 Sensors
    pub const O2_B1S1_WR_LAMBDA: u8 = 0x24;        // O2 Bank 1 Sensor 1 WR lambda
    pub const O2_B1S2_WR_LAMBDA: u8 = 0x25;        // O2 Bank 1 Sensor 2 WR lambda

    // EGR
    pub const COMMANDED_EGR: u8 = 0x2C;            // Commanded EGR (%)
    pub const EGR_ERROR: u8 = 0x2D;                // EGR error (%)

    // Evaporative System
    pub const COMMANDED_EVAP_PURGE: u8 = 0x2E;     // Commanded evaporative purge (%)
    pub const FUEL_TANK_LEVEL: u8 = 0x2F;          // Fuel tank level input (%)

    // Warm-ups & Distance
    pub const WARMUPS_SINCE_CLEAR: u8 = 0x30;      // Warm-ups since codes cleared
    pub const DISTANCE_SINCE_CLEAR: u8 = 0x31;     // Distance traveled since codes cleared (km)

    // EVAP System Vapor Pressure
    pub const EVAP_VAPOR_PRESSURE: u8 = 0x32;      // Evap system vapor pressure (Pa)

    // Barometric Pressure
    pub const BAROMETRIC_PRESSURE: u8 = 0x33;      // Absolute barometric pressure (kPa)

    // Catalyst Temperature
    pub const CAT_TEMP_B1S1: u8 = 0x3C;            // Catalyst temperature Bank 1, Sensor 1 (°C)
    pub const CAT_TEMP_B2S1: u8 = 0x3D;            // Catalyst temperature Bank 2, Sensor 1 (°C)
    pub const CAT_TEMP_B1S2: u8 = 0x3E;            // Catalyst temperature Bank 1, Sensor 2 (°C)
    pub const CAT_TEMP_B2S2: u8 = 0x3F;            // Catalyst temperature Bank 2, Sensor 2 (°C)

    // Control Module Voltage
    pub const CONTROL_MODULE_VOLTAGE: u8 = 0x42;   // Control module voltage (V)

    // Absolute Load
    pub const ABSOLUTE_LOAD: u8 = 0x43;            // Absolute load value (%)

    // Fuel/Air Commanded
    pub const FUEL_AIR_COMMANDED: u8 = 0x44;       // Commanded equivalence ratio (lambda)

    // Throttle Position B/C/D
    pub const REL_THROTTLE_POS: u8 = 0x45;         // Relative throttle position (%)
    pub const AMBIENT_AIR_TEMP: u8 = 0x46;         // Ambient air temperature (°C)
    pub const ABS_THROTTLE_POS_B: u8 = 0x47;       // Absolute throttle position B (%)
    pub const ABS_THROTTLE_POS_C: u8 = 0x48;       // Absolute throttle position C (%)
    pub const ACCEL_PEDAL_POS_D: u8 = 0x49;        // Accelerator pedal position D (%)
    pub const ACCEL_PEDAL_POS_E: u8 = 0x4A;        // Accelerator pedal position E (%)
    pub const ACCEL_PEDAL_POS_F: u8 = 0x4B;        // Accelerator pedal position F (%)
    pub const COMMANDED_THROTTLE: u8 = 0x4C;       // Commanded throttle actuator (%)

    // Time with MIL
    pub const TIME_WITH_MIL: u8 = 0x4D;            // Time run with MIL on (minutes)
    pub const TIME_SINCE_CLEAR: u8 = 0x4E;         // Time since codes cleared (minutes)

    // Fuel Type
    pub const FUEL_TYPE: u8 = 0x51;                // Fuel type

    // Ethanol
    pub const ETHANOL_PERCENT: u8 = 0x52;          // Ethanol fuel percentage (%)

    // Fuel Rail Absolute
    pub const FUEL_RAIL_PRESSURE_ABS: u8 = 0x59;   // Fuel rail absolute pressure (kPa)

    // Relative Pedal Position
    pub const REL_PEDAL_POS: u8 = 0x5A;            // Relative accelerator pedal position (%)

    // Hybrid Battery Pack
    pub const HYBRID_BATTERY_LIFE: u8 = 0x5B;      // Hybrid battery pack remaining life (%)

    // Engine Oil Temperature
    pub const ENGINE_OIL_TEMP: u8 = 0x5C;          // Engine oil temperature (°C)

    // Fuel Injection Timing
    pub const FUEL_INJECTION_TIMING: u8 = 0x5D;    // Fuel injection timing (°)

    // Engine Fuel Rate
    pub const ENGINE_FUEL_RATE: u8 = 0x5E;         // Engine fuel rate (L/h)

    // Emissions Requirements
    pub const EMISSIONS_REQUIREMENTS: u8 = 0x5F;   // Emission requirements

    // Engine Torque
    pub const DEMANDED_TORQUE: u8 = 0x61;          // Driver's demand engine torque (%)
    pub const ACTUAL_TORQUE: u8 = 0x62;            // Actual engine torque (%)
    pub const ENGINE_REF_TORQUE: u8 = 0x63;        // Engine reference torque (Nm)
}

/// BMW-specific PIDs for DME (Service 0x21 - ReadDataByLocalIdentifier)
/// Note: These are reverse-engineered and may vary by DME version (MS45, MSV70, etc.)
#[allow(dead_code)]
pub mod bmw_dme_pids {
    // Engine Status
    pub const STATUS_MOTOR: u8 = 0x02;             // Engine running status
    pub const STATUS_LAUFRUHIG: u8 = 0x05;         // Engine smoothness/rough running

    // Temperatures
    pub const MOTORTEMPERATUR: u8 = 0x10;          // Engine/coolant temperature
    pub const OELTEMPERATUR: u8 = 0x11;            // Oil temperature
    pub const ANSAUGLUFT_TEMP: u8 = 0x12;          // Intake air temperature
    pub const ABGASTEMPERATUR: u8 = 0x13;          // Exhaust gas temperature

    // Pressures
    pub const SAUGROHRDRUCK: u8 = 0x14;            // Intake manifold pressure
    pub const LADEDRUCK: u8 = 0x15;                // Boost pressure (turbo)
    pub const UMGEBUNGSDRUCK: u8 = 0x16;           // Ambient pressure
    pub const KRAFTSTOFFDRUCK: u8 = 0x17;          // Fuel pressure

    // Engine Load/Speed
    pub const MOTORDREHZAHL: u8 = 0x20;            // Engine RPM
    pub const MOTORLAST: u8 = 0x21;                // Engine load
    pub const FAHRGESCHWINDIGKEIT: u8 = 0x22;      // Vehicle speed
    pub const WUNSCHDREHZAHL: u8 = 0x23;           // Target idle RPM

    // Throttle/Pedal
    pub const DROSSELKLAPPE: u8 = 0x30;            // Throttle valve position
    pub const FAHRPEDALSTELLUNG: u8 = 0x31;        // Accelerator pedal position
    pub const FAHRPEDAL_SPANNUNG: u8 = 0x32;       // Accelerator pedal voltage

    // Ignition
    pub const ZUENDWINKEL: u8 = 0x40;              // Ignition timing angle
    pub const ZUENDWINKEL_ZYL1: u8 = 0x41;         // Ignition angle cylinder 1
    pub const ZUENDWINKEL_ZYL2: u8 = 0x42;         // Ignition angle cylinder 2
    pub const ZUENDWINKEL_ZYL3: u8 = 0x43;         // Ignition angle cylinder 3
    pub const ZUENDWINKEL_ZYL4: u8 = 0x44;         // Ignition angle cylinder 4
    pub const ZUENDWINKEL_ZYL5: u8 = 0x45;         // Ignition angle cylinder 5
    pub const ZUENDWINKEL_ZYL6: u8 = 0x46;         // Ignition angle cylinder 6

    // Fuel Injection
    pub const EINSPRITZZEIT: u8 = 0x50;            // Injection time (ms)
    pub const EINSPRITZ_KORR_ADD: u8 = 0x51;       // Injection correction additive
    pub const EINSPRITZ_KORR_MULT: u8 = 0x52;      // Injection correction multiplicative

    // Lambda/O2
    pub const LAMBDA_SOLL: u8 = 0x60;              // Target lambda
    pub const LAMBDA_IST: u8 = 0x61;               // Actual lambda
    pub const LAMBDA_VORKAT: u8 = 0x62;            // Lambda pre-catalyst
    pub const LAMBDA_NACHKAT: u8 = 0x63;           // Lambda post-catalyst

    // Fuel Adaptations
    pub const ADAPT_ADD_ZYL1: u8 = 0x70;           // Additive adaptation cylinder 1
    pub const ADAPT_ADD_ZYL2: u8 = 0x71;           // Additive adaptation cylinder 2
    pub const ADAPT_ADD_ZYL3: u8 = 0x72;           // Additive adaptation cylinder 3
    pub const ADAPT_ADD_ZYL4: u8 = 0x73;           // Additive adaptation cylinder 4
    pub const ADAPT_ADD_ZYL5: u8 = 0x74;           // Additive adaptation cylinder 5
    pub const ADAPT_ADD_ZYL6: u8 = 0x75;           // Additive adaptation cylinder 6
    pub const ADAPT_MULT: u8 = 0x78;               // Multiplicative adaptation

    // VANOS
    pub const VANOS_EINLASS: u8 = 0x80;            // VANOS intake position
    pub const VANOS_AUSLASS: u8 = 0x81;            // VANOS exhaust position
    pub const VANOS_SOLL_EINL: u8 = 0x82;          // VANOS intake target
    pub const VANOS_SOLL_AUSL: u8 = 0x83;          // VANOS exhaust target

    // Knock Sensors
    pub const KLOPFSENSOR_1: u8 = 0x90;            // Knock sensor 1
    pub const KLOPFSENSOR_2: u8 = 0x91;            // Knock sensor 2
    pub const KLOPF_RETARD_ZYL1: u8 = 0x92;        // Knock retard cylinder 1
    pub const KLOPF_RETARD_ZYL2: u8 = 0x93;        // Knock retard cylinder 2
    pub const KLOPF_RETARD_ZYL3: u8 = 0x94;        // Knock retard cylinder 3
    pub const KLOPF_RETARD_ZYL4: u8 = 0x95;        // Knock retard cylinder 4
    pub const KLOPF_RETARD_ZYL5: u8 = 0x96;        // Knock retard cylinder 5
    pub const KLOPF_RETARD_ZYL6: u8 = 0x97;        // Knock retard cylinder 6

    // Voltages
    pub const BATTERIE_SPANNUNG: u8 = 0xA0;        // Battery voltage
    pub const GENERATOR_SPANNUNG: u8 = 0xA1;       // Alternator voltage

    // Idle Control
    pub const LEERLAUF_STELLER: u8 = 0xB0;         // Idle actuator position
    pub const LEERLAUF_DREHZAHL: u8 = 0xB1;        // Idle RPM

    // Valvetronic (N52 engines)
    pub const VALVETRONIC_HUB: u8 = 0xC0;          // Valvetronic lift
    pub const VALVETRONIC_SOLL: u8 = 0xC1;         // Valvetronic target lift
    pub const VALVETRONIC_MOTOR: u8 = 0xC2;        // Valvetronic motor current

    // Diagnostics
    pub const FEHLER_SPEICHER: u8 = 0xF0;          // Error memory status
    pub const ECU_IDENTIFICATION: u8 = 0xF1;       // ECU identification
}

/// BMW EGS (Transmission) PIDs (Service 0x21)
/// Note: Proprietary and may vary by EGS version (GS19, GS20, etc.)
#[allow(dead_code)]
pub mod bmw_egs_pids {
    // Gear Information
    pub const ISTGANG: u8 = 0x01;                  // Current gear (1-6, 0=N, 7=R)
    pub const SOLLGANG: u8 = 0x02;                 // Target gear
    pub const GANGWAHLHEBEL: u8 = 0x03;            // Gear selector position (PRND)
    pub const SCHALTVORGANG: u8 = 0x04;            // Shift operation active

    // Speeds
    pub const EINGANGSDREHZAHL: u8 = 0x10;         // Input shaft speed (RPM)
    pub const AUSGANGSDREHZAHL: u8 = 0x11;         // Output shaft speed (RPM)
    pub const TURBINEN_DREHZAHL: u8 = 0x12;        // Turbine speed (RPM)
    pub const WANDLER_SCHLUPF: u8 = 0x13;          // Torque converter slip (RPM)

    // Temperatures
    pub const GETRIEBEOEL_TEMP: u8 = 0x20;         // Transmission oil temperature
    pub const WANDLER_TEMP: u8 = 0x21;             // Torque converter temperature

    // Pressures
    pub const HAUPTDRUCK: u8 = 0x30;               // Main pressure (bar)
    pub const WANDLER_DRUCK: u8 = 0x31;            // Converter pressure (bar)
    pub const SCHALTDRUCK: u8 = 0x32;              // Shift pressure (bar)

    // Torque
    pub const MOTOR_MOMENT: u8 = 0x40;             // Engine torque (Nm)
    pub const GETRIEBE_MOMENT: u8 = 0x41;          // Transmission output torque (Nm)
    pub const MOMENT_REDUZIERUNG: u8 = 0x42;       // Torque reduction during shift

    // Clutch/Lock-up
    pub const WANDLERKUPPLUNG: u8 = 0x50;          // Torque converter lock-up status
    pub const WANDLER_SCHLUPF_SOLL: u8 = 0x51;     // Target converter slip
    pub const KUPPLUNG_SCHLUPF: u8 = 0x52;         // Clutch slip (%)

    // Solenoids
    pub const MAGNETVENTIL_A: u8 = 0x60;           // Solenoid A current (mA)
    pub const MAGNETVENTIL_B: u8 = 0x61;           // Solenoid B current (mA)
    pub const MAGNETVENTIL_C: u8 = 0x62;           // Solenoid C current (mA)
    pub const MAGNETVENTIL_D: u8 = 0x63;           // Solenoid D current (mA)
    pub const MAGNETVENTIL_E: u8 = 0x64;           // Solenoid E current (mA)

    // Driving Mode
    pub const FAHRPROGRAMM: u8 = 0x70;             // Driving program (Normal/Sport/Manual)
    pub const SPORT_MODUS: u8 = 0x71;              // Sport mode active
    pub const MANUELL_MODUS: u8 = 0x72;            // Manual mode active
    pub const KICK_DOWN: u8 = 0x73;                // Kickdown active

    // Adaptation
    pub const ADAPT_DRUCK_1_2: u8 = 0x80;          // Pressure adaptation 1-2 shift
    pub const ADAPT_DRUCK_2_3: u8 = 0x81;          // Pressure adaptation 2-3 shift
    pub const ADAPT_DRUCK_3_4: u8 = 0x82;          // Pressure adaptation 3-4 shift
    pub const ADAPT_DRUCK_4_5: u8 = 0x83;          // Pressure adaptation 4-5 shift
    pub const ADAPT_DRUCK_5_6: u8 = 0x84;          // Pressure adaptation 5-6 shift
    pub const SCHALTZEIT_ADAPT: u8 = 0x88;         // Shift time adaptation

    // Status
    pub const GETRIEBE_STATUS: u8 = 0x90;          // Transmission status word
    pub const FEHLER_STATUS: u8 = 0x91;            // Error status
    pub const NOTLAUF_AKTIV: u8 = 0x92;            // Limp mode active

    // Oil Level
    pub const OEL_LEVEL: u8 = 0xA0;                // Oil level
    pub const OEL_QUALITAET: u8 = 0xA1;            // Oil quality
}

/// Legacy alias for backwards compatibility
#[allow(dead_code)]
pub mod pids {
    pub use super::obd_pids::*;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_building() {
        let msg = KwpMessage::new(0xF1, 0x12, vec![0x3E]);
        let bytes = msg.to_bytes();

        // Expected: 81 12 F1 3E CS
        assert_eq!(bytes.len(), 5);
        assert_eq!(bytes[0], 0x81); // FMT: 0x80 | 1
        assert_eq!(bytes[1], 0x12); // Target
        assert_eq!(bytes[2], 0xF1); // Source
        assert_eq!(bytes[3], 0x3E); // Service

        // Checksum
        let expected_checksum = (0x81 + 0x12 + 0xF1 + 0x3E) & 0xFF;
        assert_eq!(bytes[4], expected_checksum as u8);
    }

    #[test]
    fn test_response_parsing() {
        // Example positive response to TesterPresent
        let data = vec![0x81, 0xF1, 0x12, 0x7E, 0x22]; // FMT TGT SRC SVC CHK

        let response = KwpResponse::parse(&data).unwrap();

        assert_eq!(response.source, 0x12);
        assert_eq!(response.target, 0xF1);
        assert_eq!(response.service, 0x7E);
        assert!(response.is_positive());
    }
}
