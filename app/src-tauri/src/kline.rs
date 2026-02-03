//! K-Line Protocol Implementation (ISO 9141-2 / ISO 14230 KWP2000)
//!
//! This module handles communication with BMW ECUs via K-Line.
//! K-Line is used for older BMW systems and some body modules.

// Allow unused items as they are part of the public API but not all are used internally
#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::thread;
use std::time::{Duration, Instant};

/// K-Line protocol variants
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum KLineProtocol {
    /// ISO 9141-2 with 5 baud init
    ISO9141,
    /// ISO 14230 KWP2000 with fast init
    KWP2000Fast,
    /// ISO 14230 KWP2000 with 5 baud init
    KWP2000Slow,
}

/// K-Line initialization result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KLineInitResult {
    pub success: bool,
    pub protocol: Option<String>,
    pub key_bytes: Vec<u8>,
    pub error: Option<String>,
}

/// K-Line message format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KLineMessage {
    pub format: u8,
    pub target: u8,
    pub source: u8,
    pub data: Vec<u8>,
    pub checksum: u8,
}

impl KLineMessage {
    /// Create a new K-Line message
    pub fn new(target: u8, source: u8, data: Vec<u8>) -> Self {
        let format = if data.len() < 64 {
            0x80 | (data.len() as u8)
        } else {
            0x80 // Length in separate byte
        };

        let mut msg = Self {
            format,
            target,
            source,
            data,
            checksum: 0,
        };
        msg.checksum = msg.calculate_checksum();
        msg
    }

    /// Calculate checksum (sum of all bytes mod 256)
    pub fn calculate_checksum(&self) -> u8 {
        let mut sum: u16 = self.format as u16 + self.target as u16 + self.source as u16;
        for byte in &self.data {
            sum += *byte as u16;
        }
        (sum & 0xFF) as u8
    }

    /// Serialize message to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(4 + self.data.len());
        bytes.push(self.format);
        bytes.push(self.target);
        bytes.push(self.source);
        bytes.extend_from_slice(&self.data);
        bytes.push(self.checksum);
        bytes
    }

    /// Parse message from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < 4 {
            return Err("Message too short".to_string());
        }

        let format = bytes[0];
        let target = bytes[1];
        let source = bytes[2];

        let data_len = if format & 0x3F == 0 {
            // Length in separate byte
            if bytes.len() < 5 {
                return Err("Message too short for extended length".to_string());
            }
            bytes[3] as usize
        } else {
            (format & 0x3F) as usize
        };

        let data_start = if format & 0x3F == 0 { 4 } else { 3 };
        let data_end = data_start + data_len;

        if bytes.len() < data_end + 1 {
            return Err(format!(
                "Message too short: need {} bytes, got {}",
                data_end + 1,
                bytes.len()
            ));
        }

        let data = bytes[data_start..data_end].to_vec();
        let checksum = bytes[data_end];

        let msg = Self {
            format,
            target,
            source,
            data,
            checksum,
        };

        // Verify checksum
        if msg.calculate_checksum() != checksum {
            return Err("Invalid checksum".to_string());
        }

        Ok(msg)
    }
}

/// K-Line protocol handler
pub struct KLineHandler {
    /// Default target ECU address (0x12 for DME/DDE on BMW)
    pub target_address: u8,
    /// Tester address (0xF1 standard diagnostic tester)
    pub source_address: u8,
    /// Current protocol
    pub protocol: Option<KLineProtocol>,
    /// Inter-byte timing (P4) in milliseconds
    pub p4_timing: u64,
}

impl Default for KLineHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl KLineHandler {
    pub fn new() -> Self {
        Self {
            target_address: 0x12,  // DME/DDE default
            source_address: 0xF1,  // Tester
            protocol: None,
            p4_timing: 5,
        }
    }

    /// Perform 5 baud initialization
    ///
    /// This sends the address byte at 5 baud (200ms per bit) using
    /// the serial port's break/mark signaling via DTR/RTS control.
    ///
    /// Returns the key bytes (KB1, KB2) on success.
    pub fn init_5baud(
        port: &mut Box<dyn serialport::SerialPort>,
        address: u8,
    ) -> Result<(u8, u8), String> {
        log::info!("Starting 5 baud init with address 0x{:02X}", address);

        // Set initial line state
        port.write_data_terminal_ready(false)
            .map_err(|e| format!("Failed to set DTR: {}", e))?;
        port.write_request_to_send(false)
            .map_err(|e| format!("Failed to set RTS: {}", e))?;

        thread::sleep(Duration::from_millis(300));

        // Send address byte at 5 baud
        // Each bit takes 200ms (5 baud = 5 bits per second)
        // Format: 1 start bit (low), 8 data bits (LSB first), 1 stop bit (high)

        // Start bit (low)
        port.write_data_terminal_ready(true)
            .map_err(|e| format!("Failed to set DTR for start bit: {}", e))?;
        thread::sleep(Duration::from_millis(200));

        // Data bits (LSB first)
        for i in 0..8 {
            let bit = (address >> i) & 0x01;
            port.write_data_terminal_ready(bit == 0)
                .map_err(|e| format!("Failed to set DTR for bit {}: {}", i, e))?;
            thread::sleep(Duration::from_millis(200));
        }

        // Stop bit (high)
        port.write_data_terminal_ready(false)
            .map_err(|e| format!("Failed to set DTR for stop bit: {}", e))?;
        thread::sleep(Duration::from_millis(200));

        // Now switch to 10400 baud to receive response
        port.set_baud_rate(10400)
            .map_err(|e| format!("Failed to set baud rate: {}", e))?;

        // Wait for sync byte (0x55)
        let mut sync = [0u8; 1];
        let start = Instant::now();
        loop {
            if start.elapsed() > Duration::from_millis(300) {
                return Err("Timeout waiting for sync byte".to_string());
            }
            if port.read(&mut sync).unwrap_or(0) == 1 {
                if sync[0] == 0x55 {
                    log::info!("Received sync byte 0x55");
                    break;
                }
            }
            thread::sleep(Duration::from_millis(1));
        }

        // Read key bytes (KB1, KB2)
        let mut key_bytes = [0u8; 2];
        let mut received = 0;
        let start = Instant::now();
        while received < 2 {
            if start.elapsed() > Duration::from_millis(100) {
                return Err(format!(
                    "Timeout waiting for key bytes, received {} of 2",
                    received
                ));
            }
            let n = port.read(&mut key_bytes[received..]).unwrap_or(0);
            received += n;
            if n == 0 {
                thread::sleep(Duration::from_millis(1));
            }
        }

        let kb1 = key_bytes[0];
        let kb2 = key_bytes[1];
        log::info!("Received key bytes: KB1=0x{:02X}, KB2=0x{:02X}", kb1, kb2);

        // Send inverted KB2 as acknowledgment
        let inv_kb2 = !kb2;
        thread::sleep(Duration::from_millis(25)); // W4 timing
        port.write(&[inv_kb2])
            .map_err(|e| format!("Failed to send inverted KB2: {}", e))?;

        // Wait for inverted address as final confirmation
        let mut inv_addr = [0u8; 1];
        let start = Instant::now();
        loop {
            if start.elapsed() > Duration::from_millis(100) {
                return Err("Timeout waiting for inverted address".to_string());
            }
            if port.read(&mut inv_addr).unwrap_or(0) == 1 {
                if inv_addr[0] == !address {
                    log::info!("5 baud init successful");
                    return Ok((kb1, kb2));
                } else {
                    return Err(format!(
                        "Invalid inverted address: expected 0x{:02X}, got 0x{:02X}",
                        !address,
                        inv_addr[0]
                    ));
                }
            }
            thread::sleep(Duration::from_millis(1));
        }
    }

    /// Perform fast initialization (ISO 14230 KWP2000)
    ///
    /// This uses a 25ms low pulse followed by a 25ms high pulse,
    /// then sends a StartCommunication request.
    pub fn init_fast(
        port: &mut Box<dyn serialport::SerialPort>,
        target: u8,
        source: u8,
    ) -> Result<Vec<u8>, String> {
        log::info!(
            "Starting fast init to target 0x{:02X} from 0x{:02X}",
            target,
            source
        );

        // Set baud rate
        port.set_baud_rate(10400)
            .map_err(|e| format!("Failed to set baud rate: {}", e))?;

        // Clear buffers
        port.clear(serialport::ClearBuffer::All)
            .map_err(|e| format!("Failed to clear buffers: {}", e))?;

        // Fast init sequence: 25ms low, 25ms high
        // Use break signal for the low pulse
        // Note: serialport-rs set_break() triggers break, clear_break() releases it

        // Pull line low for 25ms (TiniL) using break
        port.set_break()
            .map_err(|e| format!("Failed to set break: {}", e))?;
        thread::sleep(Duration::from_millis(25));

        // Release line high for 25ms (TiniH)
        port.clear_break()
            .map_err(|e| format!("Failed to clear break: {}", e))?;
        thread::sleep(Duration::from_millis(25));

        // Send StartCommunication request (service 0x81)
        let start_comm = KLineMessage::new(target, source, vec![0x81]);
        let request = start_comm.to_bytes();

        log::debug!("Sending StartCommunication: {:02X?}", request);
        port.write(&request)
            .map_err(|e| format!("Failed to send StartCommunication: {}", e))?;

        // Read echo (our own transmitted bytes)
        thread::sleep(Duration::from_millis(10));
        let mut echo = vec![0u8; request.len()];
        let _ = port.read(&mut echo);

        // Read response
        let mut response = Vec::new();
        let mut buffer = [0u8; 64];
        let start = Instant::now();

        while start.elapsed() < Duration::from_millis(300) {
            match port.read(&mut buffer) {
                Ok(n) if n > 0 => {
                    response.extend_from_slice(&buffer[..n]);
                    // Check if we have a complete message
                    if response.len() >= 4 {
                        let len = if response[0] & 0x3F == 0 {
                            response.get(3).copied().unwrap_or(0) as usize + 5
                        } else {
                            (response[0] & 0x3F) as usize + 4
                        };
                        if response.len() >= len {
                            break;
                        }
                    }
                }
                _ => thread::sleep(Duration::from_millis(5)),
            }
        }

        if response.is_empty() {
            return Err("No response to StartCommunication".to_string());
        }

        log::info!("Fast init response: {:02X?}", response);

        // Parse and validate response
        let msg = KLineMessage::from_bytes(&response)?;

        // Check for positive response (0xC1 = StartCommunication positive response)
        if msg.data.first() == Some(&0xC1) {
            log::info!("Fast init successful");
            Ok(msg.data)
        } else if msg.data.first() == Some(&0x7F) {
            // Negative response
            let nrc = msg.data.get(2).copied().unwrap_or(0);
            Err(format!("Negative response, NRC: 0x{:02X}", nrc))
        } else {
            Err(format!("Unexpected response: {:02X?}", msg.data))
        }
    }

    /// Send a KWP2000 service request and receive response
    pub fn send_request(
        port: &mut Box<dyn serialport::SerialPort>,
        target: u8,
        source: u8,
        service_data: &[u8],
    ) -> Result<Vec<u8>, String> {
        let msg = KLineMessage::new(target, source, service_data.to_vec());
        let request = msg.to_bytes();

        log::debug!("Sending request: {:02X?}", request);

        // Send request
        port.write(&request)
            .map_err(|e| format!("Failed to send request: {}", e))?;

        // Wait for echo
        thread::sleep(Duration::from_millis(10));
        let mut echo = vec![0u8; request.len()];
        let _ = port.read(&mut echo);

        // Read response with timeout
        let mut response = Vec::new();
        let mut buffer = [0u8; 128];
        let start = Instant::now();

        while start.elapsed() < Duration::from_millis(1000) {
            match port.read(&mut buffer) {
                Ok(n) if n > 0 => {
                    response.extend_from_slice(&buffer[..n]);
                    // Check if we have a complete message
                    if response.len() >= 4 {
                        let len = if response[0] & 0x3F == 0 {
                            response.get(3).copied().unwrap_or(0) as usize + 5
                        } else {
                            (response[0] & 0x3F) as usize + 4
                        };
                        if response.len() >= len {
                            break;
                        }
                    }
                }
                _ => thread::sleep(Duration::from_millis(5)),
            }
        }

        if response.is_empty() {
            return Err("No response received".to_string());
        }

        log::debug!("Received response: {:02X?}", response);

        // Parse response
        let msg = KLineMessage::from_bytes(&response)?;
        Ok(msg.data)
    }

    /// Send TesterPresent to keep session alive
    pub fn tester_present(
        port: &mut Box<dyn serialport::SerialPort>,
        target: u8,
        source: u8,
    ) -> Result<(), String> {
        // TesterPresent service (0x3E) with response suppressed (0x00)
        let response = Self::send_request(port, target, source, &[0x3E, 0x00])?;

        if response.first() == Some(&0x7E) {
            Ok(())
        } else if response.first() == Some(&0x7F) {
            let nrc = response.get(2).copied().unwrap_or(0);
            Err(format!("TesterPresent rejected, NRC: 0x{:02X}", nrc))
        } else {
            Ok(()) // Response might be suppressed
        }
    }

    /// Stop communication session
    pub fn stop_communication(
        port: &mut Box<dyn serialport::SerialPort>,
        target: u8,
        source: u8,
    ) -> Result<(), String> {
        let response = Self::send_request(port, target, source, &[0x82])?;

        if response.first() == Some(&0xC2) {
            log::info!("Communication stopped");
            Ok(())
        } else {
            Err(format!("Unexpected stop response: {:02X?}", response))
        }
    }
}

/// BMW ECU addresses for K-Line
pub mod ecu_addresses {
    pub const DME: u8 = 0x12;      // Engine control (DME/DDE)
    pub const EGS: u8 = 0x32;      // Transmission
    pub const ABS_DSC: u8 = 0x44;  // ABS/DSC
    pub const AIRBAG: u8 = 0x4A;   // Airbag (MRS)
    pub const IHKA: u8 = 0x5B;     // Climate control
    pub const KOMBI: u8 = 0x60;    // Instrument cluster
    pub const EWS: u8 = 0x44;      // Immobilizer
    pub const LCM: u8 = 0x68;      // Light control module
    pub const GM: u8 = 0x00;       // General module (ZKE)
    pub const TESTER: u8 = 0xF1;   // Diagnostic tester
}
