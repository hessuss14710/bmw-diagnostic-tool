//! D-CAN Protocol Implementation (BMW Diagnostic CAN)
//!
//! D-CAN uses CAN bus at 500 kbaud with ISO-TP (ISO 15765) transport layer.
//! The K+DCAN cable uses FTDI chip with special firmware that bridges
//! CAN to serial communication.
//!
//! Note: The K+DCAN cable requires switching between K-Line and D-CAN modes
//! using the DTR/RTS pins:
//! - K-Line mode: RTS=0, DTR=0 (default after power-on)
//! - D-CAN mode: RTS=1, DTR=0

// Allow unused items as they are part of the public API but not all are used internally
#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::thread;
use std::time::{Duration, Instant};

/// D-CAN frame types (ISO-TP)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FrameType {
    Single,        // SF - Single Frame (data <= 7 bytes)
    First,         // FF - First Frame (start of multi-frame)
    Consecutive,   // CF - Consecutive Frame
    FlowControl,   // FC - Flow Control
}

/// ISO-TP frame
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsoTpFrame {
    pub frame_type: u8,
    pub data: Vec<u8>,
    pub sequence: Option<u8>,
    pub total_length: Option<u16>,
}

impl IsoTpFrame {
    /// Create a single frame (data up to 7 bytes)
    pub fn single(data: Vec<u8>) -> Result<Self, String> {
        if data.len() > 7 {
            return Err("Data too long for single frame".to_string());
        }
        Ok(Self {
            frame_type: 0x00,
            data,
            sequence: None,
            total_length: None,
        })
    }

    /// Create a first frame (for multi-frame messages)
    pub fn first(data: &[u8], total_length: u16) -> Self {
        let frame_data = data[..6.min(data.len())].to_vec();
        Self {
            frame_type: 0x10,
            data: frame_data,
            sequence: None,
            total_length: Some(total_length),
        }
    }

    /// Create a consecutive frame
    pub fn consecutive(data: Vec<u8>, sequence: u8) -> Self {
        Self {
            frame_type: 0x20,
            data,
            sequence: Some(sequence & 0x0F),
            total_length: None,
        }
    }

    /// Create a flow control frame
    pub fn flow_control(flag: u8, block_size: u8, separation_time: u8) -> Self {
        Self {
            frame_type: 0x30,
            data: vec![flag, block_size, separation_time],
            sequence: None,
            total_length: None,
        }
    }

    /// Serialize frame to CAN data bytes (8 bytes)
    pub fn to_can_data(&self) -> [u8; 8] {
        let mut data = [0x00u8; 8];

        match self.frame_type & 0xF0 {
            0x00 => {
                // Single frame: [0L DDDDDD] where L = length
                data[0] = self.data.len() as u8;
                for (i, &byte) in self.data.iter().enumerate() {
                    if i < 7 {
                        data[i + 1] = byte;
                    }
                }
            }
            0x10 => {
                // First frame: [1H HL DDDDDD] where HHL = total length
                let len = self.total_length.unwrap_or(0);
                data[0] = 0x10 | ((len >> 8) as u8 & 0x0F);
                data[1] = (len & 0xFF) as u8;
                for (i, &byte) in self.data.iter().enumerate() {
                    if i < 6 {
                        data[i + 2] = byte;
                    }
                }
            }
            0x20 => {
                // Consecutive frame: [2N DDDDDDD] where N = sequence
                data[0] = 0x20 | (self.sequence.unwrap_or(0) & 0x0F);
                for (i, &byte) in self.data.iter().enumerate() {
                    if i < 7 {
                        data[i + 1] = byte;
                    }
                }
            }
            0x30 => {
                // Flow control: [3F BS ST] where F=flag, BS=block size, ST=sep time
                data[0] = 0x30 | (self.data.first().copied().unwrap_or(0) & 0x0F);
                data[1] = self.data.get(1).copied().unwrap_or(0);
                data[2] = self.data.get(2).copied().unwrap_or(0);
            }
            _ => {}
        }

        data
    }

    /// Parse frame from CAN data bytes
    pub fn from_can_data(data: &[u8]) -> Result<Self, String> {
        if data.is_empty() {
            return Err("Empty data".to_string());
        }

        let pci = data[0];
        let frame_type = pci & 0xF0;

        match frame_type {
            0x00 => {
                // Single frame
                let len = (pci & 0x0F) as usize;
                if data.len() < len + 1 {
                    return Err("Data too short for single frame".to_string());
                }
                Ok(Self {
                    frame_type: 0x00,
                    data: data[1..=len].to_vec(),
                    sequence: None,
                    total_length: None,
                })
            }
            0x10 => {
                // First frame
                if data.len() < 8 {
                    return Err("Data too short for first frame".to_string());
                }
                let len = (((pci & 0x0F) as u16) << 8) | (data[1] as u16);
                Ok(Self {
                    frame_type: 0x10,
                    data: data[2..8].to_vec(),
                    sequence: None,
                    total_length: Some(len),
                })
            }
            0x20 => {
                // Consecutive frame
                let seq = pci & 0x0F;
                Ok(Self {
                    frame_type: 0x20,
                    data: data[1..].to_vec(),
                    sequence: Some(seq),
                    total_length: None,
                })
            }
            0x30 => {
                // Flow control
                Ok(Self {
                    frame_type: 0x30,
                    data: vec![
                        pci & 0x0F,
                        data.get(1).copied().unwrap_or(0),
                        data.get(2).copied().unwrap_or(0),
                    ],
                    sequence: None,
                    total_length: None,
                })
            }
            _ => Err(format!("Unknown frame type: 0x{:02X}", frame_type)),
        }
    }
}

/// D-CAN protocol handler
pub struct DCanHandler {
    /// Transmit CAN ID (tester -> ECU)
    pub tx_id: u32,
    /// Receive CAN ID (ECU -> tester)
    pub rx_id: u32,
    /// Block size for flow control
    pub block_size: u8,
    /// Separation time in milliseconds
    pub separation_time: u8,
}

impl Default for DCanHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl DCanHandler {
    pub fn new() -> Self {
        Self {
            tx_id: 0x6F1,  // Default tester ID for BMW
            rx_id: 0x612,  // Default DME response ID
            block_size: 0,
            separation_time: 0,
        }
    }

    /// Create handler for specific ECU
    pub fn for_ecu(ecu_id: u8) -> Self {
        // BMW D-CAN addressing:
        // Tester TX: 0x6F1 (to all) or 0x600 + ecu_id
        // ECU RX: 0x600 + ecu_id + 8
        Self {
            tx_id: 0x6F1,
            rx_id: 0x600 + (ecu_id as u32) + 8,
            block_size: 0,
            separation_time: 0,
        }
    }

    /// Switch K+DCAN cable to D-CAN mode
    ///
    /// The K+DCAN cable uses RTS line to switch modes:
    /// - RTS=0: K-Line mode (default)
    /// - RTS=1: D-CAN mode
    pub fn switch_to_dcan_mode(port: &mut Box<dyn serialport::SerialPort>) -> Result<(), String> {
        log::info!("Switching to D-CAN mode");

        // Set RTS high to enable D-CAN mode
        port.write_request_to_send(true)
            .map_err(|e| format!("Failed to set RTS: {}", e))?;

        // Set baud rate to 500000 for D-CAN
        port.set_baud_rate(500000)
            .map_err(|e| format!("Failed to set baud rate: {}", e))?;

        // Clear buffers
        port.clear(serialport::ClearBuffer::All)
            .map_err(|e| format!("Failed to clear buffers: {}", e))?;

        thread::sleep(Duration::from_millis(100));

        log::info!("D-CAN mode enabled at 500 kbaud");
        Ok(())
    }

    /// Switch K+DCAN cable to K-Line mode
    pub fn switch_to_kline_mode(port: &mut Box<dyn serialport::SerialPort>) -> Result<(), String> {
        log::info!("Switching to K-Line mode");

        // Set RTS low to enable K-Line mode
        port.write_request_to_send(false)
            .map_err(|e| format!("Failed to set RTS: {}", e))?;

        // Set baud rate to 10400 for K-Line
        port.set_baud_rate(10400)
            .map_err(|e| format!("Failed to set baud rate: {}", e))?;

        // Clear buffers
        port.clear(serialport::ClearBuffer::All)
            .map_err(|e| format!("Failed to clear buffers: {}", e))?;

        thread::sleep(Duration::from_millis(100));

        log::info!("K-Line mode enabled at 10400 baud");
        Ok(())
    }

    /// Send ISO-TP message and receive response
    ///
    /// This handles segmentation for messages > 7 bytes
    pub fn send_message(
        port: &mut Box<dyn serialport::SerialPort>,
        tx_id: u32,
        rx_id: u32,
        data: &[u8],
    ) -> Result<Vec<u8>, String> {
        if data.is_empty() {
            return Err("Empty data".to_string());
        }

        // For K+DCAN cable, we send CAN frames as serial data
        // Format: [ID_HI] [ID_LO] [LEN] [DATA...]
        // Where ID is 11-bit CAN ID, LEN is always 8

        if data.len() <= 7 {
            // Single frame
            let frame = IsoTpFrame::single(data.to_vec())?;
            Self::send_can_frame(port, tx_id, &frame.to_can_data())?;
        } else {
            // Multi-frame: First frame + consecutive frames
            let total_len = data.len();

            // Send first frame (contains first 6 bytes)
            let first = IsoTpFrame::first(data, total_len as u16);
            Self::send_can_frame(port, tx_id, &first.to_can_data())?;

            // Wait for flow control
            let fc = Self::receive_can_frame(port, rx_id, Duration::from_millis(100))?;
            let fc_frame = IsoTpFrame::from_can_data(&fc)?;

            if fc_frame.frame_type != 0x30 {
                return Err("Expected flow control frame".to_string());
            }

            let fc_flag = fc_frame.data.first().copied().unwrap_or(0);
            if fc_flag != 0 {
                return Err(format!("Flow control: wait or overflow ({})", fc_flag));
            }

            // Send consecutive frames
            let mut offset = 6;
            let mut sequence = 1u8;

            while offset < data.len() {
                let chunk_end = (offset + 7).min(data.len());
                let chunk = data[offset..chunk_end].to_vec();

                let cf = IsoTpFrame::consecutive(chunk, sequence);
                Self::send_can_frame(port, tx_id, &cf.to_can_data())?;

                offset = chunk_end;
                sequence = (sequence + 1) & 0x0F;

                // Small delay between frames
                thread::sleep(Duration::from_millis(1));
            }
        }

        // Receive response
        Self::receive_isotp_message(port, rx_id, Duration::from_millis(1000))
    }

    /// Send a single CAN frame via K+DCAN cable
    fn send_can_frame(
        port: &mut Box<dyn serialport::SerialPort>,
        can_id: u32,
        data: &[u8; 8],
    ) -> Result<(), String> {
        // K+DCAN cable protocol for D-CAN:
        // The FTDI chip with custom firmware expects raw CAN frames
        // Format varies by cable manufacturer, common format:
        // [LEN] [ID_HI] [ID_LO] [DATA x 8]

        let mut frame = Vec::with_capacity(12);
        frame.push(12); // Total frame length
        frame.push(((can_id >> 8) & 0xFF) as u8);
        frame.push((can_id & 0xFF) as u8);
        frame.extend_from_slice(data);

        log::debug!("Sending CAN frame ID=0x{:03X}: {:02X?}", can_id, data);

        port.write(&frame)
            .map_err(|e| format!("Failed to send CAN frame: {}", e))?;

        Ok(())
    }

    /// Receive a single CAN frame
    fn receive_can_frame(
        port: &mut Box<dyn serialport::SerialPort>,
        expected_id: u32,
        timeout: Duration,
    ) -> Result<Vec<u8>, String> {
        let mut buffer = [0u8; 64];
        let start = Instant::now();

        while start.elapsed() < timeout {
            match port.read(&mut buffer) {
                Ok(n) if n >= 11 => {
                    // Parse frame: [LEN] [ID_HI] [ID_LO] [DATA x 8]
                    let id = ((buffer[1] as u32) << 8) | (buffer[2] as u32);

                    if id == expected_id {
                        log::debug!("Received CAN frame ID=0x{:03X}: {:02X?}", id, &buffer[3..11]);
                        return Ok(buffer[3..11].to_vec());
                    }
                }
                Ok(_) => {}
                Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {}
                Err(e) => return Err(format!("Read error: {}", e)),
            }
            thread::sleep(Duration::from_millis(1));
        }

        Err("Timeout waiting for CAN frame".to_string())
    }

    /// Receive a complete ISO-TP message (handles multi-frame)
    fn receive_isotp_message(
        port: &mut Box<dyn serialport::SerialPort>,
        rx_id: u32,
        timeout: Duration,
    ) -> Result<Vec<u8>, String> {
        let start = Instant::now();

        // Get first frame
        let first_data = Self::receive_can_frame(port, rx_id, timeout)?;
        let first = IsoTpFrame::from_can_data(&first_data)?;

        match first.frame_type {
            0x00 => {
                // Single frame - return data directly
                Ok(first.data)
            }
            0x10 => {
                // First frame of multi-frame message
                let total_len = first.total_length.unwrap_or(0) as usize;
                let mut result = first.data.clone();

                // Send flow control (CTS = Clear To Send)
                // Note: For receiving, we don't actually send FC in this simple implementation
                // The K+DCAN cable handles this at firmware level
                let _fc = IsoTpFrame::flow_control(0, 0, 0);

                // Receive consecutive frames
                let mut expected_seq = 1u8;

                while result.len() < total_len {
                    let remaining_timeout = timeout
                        .checked_sub(start.elapsed())
                        .unwrap_or(Duration::ZERO);

                    if remaining_timeout.is_zero() {
                        return Err("Timeout receiving multi-frame message".to_string());
                    }

                    let cf_data = Self::receive_can_frame(port, rx_id, remaining_timeout)?;
                    let cf = IsoTpFrame::from_can_data(&cf_data)?;

                    if cf.frame_type != 0x20 {
                        return Err(format!(
                            "Expected consecutive frame, got type 0x{:02X}",
                            cf.frame_type
                        ));
                    }

                    let seq = cf.sequence.unwrap_or(0);
                    if seq != expected_seq {
                        return Err(format!(
                            "Sequence error: expected {}, got {}",
                            expected_seq, seq
                        ));
                    }

                    result.extend_from_slice(&cf.data);
                    expected_seq = (expected_seq + 1) & 0x0F;
                }

                // Trim to exact length
                result.truncate(total_len);
                Ok(result)
            }
            _ => Err(format!(
                "Unexpected frame type: 0x{:02X}",
                first.frame_type
            )),
        }
    }
}

/// BMW ECU CAN IDs for D-CAN
pub mod can_ids {
    // Functional (broadcast) addresses
    pub const FUNCTIONAL_REQ: u32 = 0x6F1;  // Request to all ECUs

    // Physical addresses (ECU specific)
    pub const DME_TX: u32 = 0x612;   // DME/DDE request
    pub const DME_RX: u32 = 0x612;   // DME/DDE response
    pub const EGS_TX: u32 = 0x618;   // Transmission request
    pub const EGS_RX: u32 = 0x618;   // Transmission response
    pub const DSC_TX: u32 = 0x6D8;   // DSC/ABS request
    pub const DSC_RX: u32 = 0x6D8;   // DSC/ABS response

    // Common response offset
    pub const RESPONSE_OFFSET: u32 = 8;  // ECU responds on TX_ID + 8

    /// Get CAN IDs for a given ECU
    /// Returns (tx_id, rx_id) tuple
    pub fn for_ecu(ecu_name: &str) -> Option<(u32, u32)> {
        match ecu_name.to_uppercase().as_str() {
            "DDE" | "DME" => Some((0x612, 0x612 + 8)),
            "EGS" => Some((0x618, 0x618 + 8)),
            "DSC" => Some((0x6D8, 0x6D8 + 8)),
            "KOMBI" => Some((0x660, 0x660 + 8)),
            "CAS" => Some((0x640, 0x640 + 8)),
            "FRM" => Some((0x668, 0x668 + 8)),
            "ACSM" => Some((0x6C0, 0x6C0 + 8)),
            "CCC" | "CIC" => Some((0x6F1, 0x63F)),  // Head unit uses functional addressing
            _ => None,
        }
    }
}

// =============================================================================
// High-Level D-CAN UDS Functions
// =============================================================================

use crate::bmw::Dtc;

impl DCanHandler {
    /// Send UDS request and receive response via D-CAN
    pub fn send_uds_request(
        port: &mut Box<dyn serialport::SerialPort>,
        tx_id: u32,
        rx_id: u32,
        service_data: &[u8],
    ) -> Result<Vec<u8>, String> {
        Self::send_message(port, tx_id, rx_id, service_data)
    }

    /// Read DTCs from ECU via D-CAN
    pub fn read_dtcs(
        port: &mut Box<dyn serialport::SerialPort>,
        tx_id: u32,
        rx_id: u32,
    ) -> Result<Vec<Dtc>, String> {
        // UDS ReadDTCInformation (0x19) with sub-function 0x02 (reportDTCByStatusMask)
        let request = vec![0x19, 0x02, 0xFF];

        let response = Self::send_message(port, tx_id, rx_id, &request)?;

        // Parse response
        if response.first() != Some(&0x59) {
            if response.first() == Some(&0x7F) {
                let nrc = response.get(2).copied().unwrap_or(0);
                return Err(format!("Negative response: 0x{:02X}", nrc));
            }
            return Err(format!("Unexpected response: {:02X?}", response));
        }

        // Response format: [0x59] [sub-function] [status_mask] [DTC1_HI] [DTC1_LO] [STATUS1] ...
        let mut dtcs = Vec::new();
        if response.len() >= 3 {
            let data = &response[3..];
            for chunk in data.chunks(3) {
                if chunk.len() >= 3 {
                    if let Some(dtc) = Dtc::from_bytes(chunk) {
                        dtcs.push(dtc);
                    }
                }
            }
        }

        Ok(dtcs)
    }

    /// Clear DTCs from ECU via D-CAN
    pub fn clear_dtcs(
        port: &mut Box<dyn serialport::SerialPort>,
        tx_id: u32,
        rx_id: u32,
    ) -> Result<(), String> {
        // UDS ClearDiagnosticInformation (0x14) with group = all (0xFFFFFF)
        let request = vec![0x14, 0xFF, 0xFF, 0xFF];

        let response = Self::send_message(port, tx_id, rx_id, &request)?;

        if response.first() == Some(&0x54) {
            Ok(())
        } else if response.first() == Some(&0x7F) {
            let nrc = response.get(2).copied().unwrap_or(0);
            Err(format!("Clear DTCs failed: NRC 0x{:02X}", nrc))
        } else {
            Err(format!("Unexpected response: {:02X?}", response))
        }
    }

    /// Read data by identifier via D-CAN
    pub fn read_data_by_id(
        port: &mut Box<dyn serialport::SerialPort>,
        tx_id: u32,
        rx_id: u32,
        did: u16,
    ) -> Result<Vec<u8>, String> {
        let request = vec![0x22, (did >> 8) as u8, (did & 0xFF) as u8];

        let response = Self::send_message(port, tx_id, rx_id, &request)?;

        if response.first() == Some(&0x62) && response.len() >= 3 {
            // Verify DID matches
            let resp_did = ((response[1] as u16) << 8) | (response[2] as u16);
            if resp_did == did {
                return Ok(response[3..].to_vec());
            }
            return Err(format!("DID mismatch: expected 0x{:04X}, got 0x{:04X}", did, resp_did));
        }

        if response.first() == Some(&0x7F) {
            let nrc = response.get(2).copied().unwrap_or(0);
            return Err(format!("Read DID 0x{:04X} failed: NRC 0x{:02X}", did, nrc));
        }

        Err(format!("Unexpected response: {:02X?}", response))
    }

    /// Start diagnostic session via D-CAN
    pub fn start_session(
        port: &mut Box<dyn serialport::SerialPort>,
        tx_id: u32,
        rx_id: u32,
        session_type: u8,
    ) -> Result<(), String> {
        let request = vec![0x10, session_type];

        let response = Self::send_message(port, tx_id, rx_id, &request)?;

        if response.first() == Some(&0x50) {
            Ok(())
        } else if response.first() == Some(&0x7F) {
            let nrc = response.get(2).copied().unwrap_or(0);
            Err(format!("Session 0x{:02X} rejected: NRC 0x{:02X}", session_type, nrc))
        } else {
            Err(format!("Unexpected response: {:02X?}", response))
        }
    }

    /// Send TesterPresent via D-CAN
    pub fn tester_present(
        port: &mut Box<dyn serialport::SerialPort>,
        tx_id: u32,
        rx_id: u32,
    ) -> Result<(), String> {
        let request = vec![0x3E, 0x00]; // TesterPresent with response expected

        let response = Self::send_message(port, tx_id, rx_id, &request)?;

        if response.first() == Some(&0x7E) {
            Ok(())
        } else if response.first() == Some(&0x7F) {
            let nrc = response.get(2).copied().unwrap_or(0);
            Err(format!("TesterPresent rejected: NRC 0x{:02X}", nrc))
        } else {
            Err(format!("Unexpected response: {:02X?}", response))
        }
    }

    /// Execute routine control via D-CAN
    pub fn routine_control(
        port: &mut Box<dyn serialport::SerialPort>,
        tx_id: u32,
        rx_id: u32,
        routine_id: u16,
        sub_function: u8,
        data: Option<&[u8]>,
    ) -> Result<Vec<u8>, String> {
        let mut request = vec![
            0x31,
            sub_function,
            (routine_id >> 8) as u8,
            (routine_id & 0xFF) as u8,
        ];
        if let Some(extra) = data {
            request.extend_from_slice(extra);
        }

        let response = Self::send_message(port, tx_id, rx_id, &request)?;

        if response.first() == Some(&0x71) {
            // Return routine result data (skip service ID, sub-function, routine ID)
            Ok(response.get(4..).unwrap_or(&[]).to_vec())
        } else if response.first() == Some(&0x7F) {
            let nrc = response.get(2).copied().unwrap_or(0);
            Err(format!("Routine 0x{:04X} failed: NRC 0x{:02X}", routine_id, nrc))
        } else {
            Err(format!("Unexpected response: {:02X?}", response))
        }
    }
}

// =============================================================================
// Protocol Auto-Detection
// =============================================================================

/// Detect which protocol (K-Line or D-CAN) an ECU supports
pub fn detect_ecu_protocol(
    port: &mut Box<dyn serialport::SerialPort>,
    ecu_name: &str,
) -> Result<String, String> {
    use crate::kline::KLineHandler;

    // First try D-CAN if ECU has known CAN IDs
    if let Some((tx_id, rx_id)) = can_ids::for_ecu(ecu_name) {
        // Switch to D-CAN mode
        DCanHandler::switch_to_dcan_mode(port)?;

        // Try TesterPresent
        let tp_request = vec![0x3E, 0x00];
        match DCanHandler::send_message(port, tx_id, rx_id, &tp_request) {
            Ok(response) if response.first() == Some(&0x7E) => {
                log::info!("ECU {} responds on D-CAN", ecu_name);
                return Ok("D-CAN".to_string());
            }
            _ => {
                log::debug!("ECU {} did not respond on D-CAN, trying K-Line", ecu_name);
            }
        }
    }

    // Try K-Line
    DCanHandler::switch_to_kline_mode(port)?;

    // Get K-Line address for ECU
    let kline_addr = match ecu_name.to_uppercase().as_str() {
        "DDE" | "DME" => 0x12,
        "EGS" => 0x32,
        "DSC" => 0x44,
        "KOMBI" => 0x60,
        "FRM" => 0x68,
        "ACSM" => 0x6C,
        "CAS" => 0x40,
        _ => return Err(format!("Unknown ECU: {}", ecu_name)),
    };

    // Try fast init
    let source = 0xF1;
    match KLineHandler::init_fast(port, kline_addr, source) {
        Ok(_) => {
            log::info!("ECU {} responds on K-Line at 0x{:02X}", ecu_name, kline_addr);
            Ok("K-Line".to_string())
        }
        Err(e) => {
            log::warn!("ECU {} not responding: {}", ecu_name, e);
            Err(format!("ECU {} not responding on K-Line or D-CAN", ecu_name))
        }
    }
}
