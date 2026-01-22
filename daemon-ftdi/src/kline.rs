//! K-Line Protocol Implementation
//!
//! Implements ISO 9141-2 and ISO 14230 (KWP2000) initialization
//! with microsecond-level timing precision.

use crate::ftdi::FtdiConnection;
use crate::kwp2000::{KwpMessage, KwpResponse};
use anyhow::{anyhow, Result};
use std::time::Instant;
use tracing::{debug, info, warn};

/// K-Line protocol handler
pub struct KLine {
    ftdi: FtdiConnection,
    ecu_address: u8,
    tester_address: u8,
    initialized: bool,
    key_bytes: Option<[u8; 2]>,
    /// Timestamp of last request completion (for P3min timing)
    last_request_time: Option<Instant>,
    /// P3min timing in milliseconds (default 55ms per ISO 14230)
    p3_min_ms: u64,
}

/// ECU addresses for BMW E60 K-Line (KWP2000)
///
/// Note: On E60, body electronics (ZKE/FRM) are on PT-CAN/K-CAN, not K-Line.
/// The old ZKE address 0x00 (DS2 protocol at 9600 baud) is NOT compatible
/// with KWP2000 at 10400 baud. Only DME/EGS are on K-Line pin 7,
/// other modules use K-Line pin 8.
#[derive(Debug, Clone, Copy)]
pub enum EcuAddress {
    DME = 0x12,    // Engine (Digital Motor Electronics) - K-Line pin 7
    EGS = 0x18,    // Transmission - K-Line pin 7
    DSC = 0x44,    // Stability Control - K-Line pin 8
    KOMBI = 0x60,  // Instrument Cluster - K-Line pin 8
    IHKA = 0x5B,   // Climate Control - K-Line pin 8
}

impl From<EcuAddress> for u8 {
    fn from(addr: EcuAddress) -> u8 {
        addr as u8
    }
}

/// Initialization result
#[derive(Debug)]
pub struct InitResult {
    pub success: bool,
    pub key_bytes: Option<[u8; 2]>,
    pub timing_p2_max: Option<u16>,
    pub timing_p3_min: Option<u16>,
}

impl KLine {
    /// Create new K-Line handler
    pub fn new(ftdi: FtdiConnection) -> Self {
        Self {
            ftdi,
            ecu_address: 0x12, // Default to DME
            tester_address: 0xF1,
            initialized: false,
            key_bytes: None,
            last_request_time: None,
            p3_min_ms: 55, // ISO 14230 default P3min
        }
    }

    /// Set ECU address
    pub fn set_ecu_address(&mut self, address: u8) {
        self.ecu_address = address;
        self.initialized = false;
    }

    /// 5-Baud Initialization (ISO 9141-2)
    ///
    /// This is the slow initialization method that requires precise timing:
    /// 1. Send ECU address at 5 baud (200ms per bit, 2 seconds total)
    /// 2. Wait for sync byte (0x55)
    /// 3. Receive key bytes (KB1, KB2)
    /// 4. Send inverted KB2
    /// 5. Receive inverted address (0xCC = ~0x33)
    pub fn init_5baud(&mut self, address: u8) -> Result<InitResult> {
        info!("Starting 5-baud initialization for ECU 0x{:02X}", address);
        self.ecu_address = address;

        // Ensure K-Line configuration
        self.ftdi.configure_kline()?;
        self.ftdi.purge()?;

        // Step 1: Send functional address 0x33 at 5 baud
        // ISO 14230-2 slow init ALWAYS uses 0x33, not the physical ECU address
        const INIT_ADDRESS: u8 = 0x33;
        self.ftdi.send_5baud(INIT_ADDRESS)?;

        // Step 2: Wait for sync byte (0x55) - ECU sends this at 10400 baud
        // Wait up to 400ms for response (extended for slower ECUs)
        let start = Instant::now();
        let mut sync_received = false;

        while start.elapsed().as_millis() < 400 {
            let mut buf = [0u8; 1];
            if self.ftdi.read(&mut buf, 50)? > 0 {
                if buf[0] == 0x55 {
                    sync_received = true;
                    debug!("Received sync byte 0x55");
                    break;
                }
            }
        }

        if !sync_received {
            warn!("No sync byte received from ECU");
            return Ok(InitResult {
                success: false,
                key_bytes: None,
                timing_p2_max: None,
                timing_p3_min: None,
            });
        }

        // Step 3: Receive key bytes (KB1, KB2)
        // W1 timing: 5-20ms between bytes
        FtdiConnection::delay_ms(5);

        let kb1 = self.ftdi.read_exact(1, 50)?[0];
        debug!("Received KB1: 0x{:02X}", kb1);

        FtdiConnection::delay_ms(5);

        let kb2 = self.ftdi.read_exact(1, 50)?[0];
        debug!("Received KB2: 0x{:02X}", kb2);

        self.key_bytes = Some([kb1, kb2]);

        // Step 4: Send inverted KB2
        // W4 timing: 25-50ms after receiving KB2
        FtdiConnection::delay_ms(25);

        let inverted_kb2 = !kb2;
        debug!("Sending inverted KB2: 0x{:02X}", inverted_kb2);
        self.ftdi.write(&[inverted_kb2])?;

        // Read back our own echo (K-Line is half-duplex)
        let mut echo = [0u8; 1];
        if let Ok(n) = self.ftdi.read(&mut echo, 20) {
            if n > 0 && echo[0] != inverted_kb2 {
                warn!("Echo mismatch: sent 0x{:02X}, got 0x{:02X}", inverted_kb2, echo[0]);
            }
        }

        // Step 5: Receive inverted init address (~0x33 = 0xCC)
        // W4 timing: 25-50ms
        FtdiConnection::delay_ms(25);

        let response = self.ftdi.read_exact(1, 100)?[0];
        let expected = !INIT_ADDRESS; // 0xCC

        if response != expected {
            warn!(
                "Unexpected response: got 0x{:02X}, expected 0x{:02X} (~0x33)",
                response, expected
            );
            return Ok(InitResult {
                success: false,
                key_bytes: Some([kb1, kb2]),
                timing_p2_max: None,
                timing_p3_min: None,
            });
        }

        info!("5-baud initialization successful!");
        self.initialized = true;

        Ok(InitResult {
            success: true,
            key_bytes: Some([kb1, kb2]),
            timing_p2_max: Some(50),  // Default P2 max
            timing_p3_min: Some(55),  // Default P3 min
        })
    }

    /// Fast Initialization (ISO 14230 / KWP2000)
    ///
    /// This is the faster initialization method:
    /// 1. Send 25ms break signal (TiniL)
    /// 2. Wait 25ms (TWup)
    /// 3. Send StartCommunication request (0x81)
    /// 4. Receive positive response (0xC1)
    pub fn init_fast(&mut self, address: u8) -> Result<InitResult> {
        info!("Starting fast initialization for ECU 0x{:02X}", address);
        self.ecu_address = address;

        // Ensure K-Line configuration
        self.ftdi.configure_kline()?;
        self.ftdi.purge()?;

        // Step 1: Send 30ms break (TiniL)
        // ISO 14230 specifies TiniL = 25-50ms, using 30ms for better compatibility
        self.ftdi.send_break(30)?;

        // Step 2: Wait 25ms (TWup - Wake-up time)
        FtdiConnection::delay_ms(25);

        // Step 3: Send StartCommunication (0x81)
        let start_comm = KwpMessage::new(self.tester_address, address, vec![0x81]);
        let bytes = start_comm.to_bytes();

        debug!("TX StartCommunication: {:02X?}", bytes);
        self.ftdi.write(&bytes)?;

        // Read back our own transmission (K-Line is half-duplex)
        let mut echo = vec![0u8; bytes.len()];
        if let Ok(n) = self.ftdi.read(&mut echo, 100) {
            if n > 0 && echo[..n] != bytes[..n.min(bytes.len())] {
                warn!("Echo mismatch in fast init");
            }
        }

        // P2 timing: ECU will respond within 25-50ms, no artificial delay needed
        // The read timeout handles waiting for response

        // Read response
        let mut response_buf = vec![0u8; 32];
        let read = self.ftdi.read(&mut response_buf, 200)?;

        if read == 0 {
            warn!("No response to StartCommunication");
            return Ok(InitResult {
                success: false,
                key_bytes: None,
                timing_p2_max: None,
                timing_p3_min: None,
            });
        }

        let response_data = &response_buf[..read];
        debug!("RX: {:02X?}", response_data);

        // Parse response
        if let Some(response) = KwpResponse::parse(response_data) {
            if response.service == 0xC1 {
                // Positive response to StartCommunication
                info!("Fast initialization successful!");
                self.initialized = true;

                // Extract key bytes (KB1, KB2) if present
                let (p2_max, p3_min) = if response.data.len() >= 2 {
                    // KB1, KB2 are always first 2 bytes of positive response
                    let kb1 = response.data[0];
                    let kb2 = response.data[1];
                    self.key_bytes = Some([kb1, kb2]);

                    // Default timing (P2max=50ms, P3min=55ms per ISO 14230)
                    (Some(50), Some(55))
                } else {
                    // No key bytes returned, use defaults
                    self.key_bytes = Some([0x8F, 0xEA]); // Common defaults
                    (Some(50), Some(55))
                };

                return Ok(InitResult {
                    success: true,
                    key_bytes: self.key_bytes,
                    timing_p2_max: p2_max,
                    timing_p3_min: p3_min,
                });
            } else if response.service == 0x7F {
                // Negative response
                let error_code = response.data.get(1).copied().unwrap_or(0x00);
                warn!("Negative response: error code 0x{:02X}", error_code);
            }
        }

        Ok(InitResult {
            success: false,
            key_bytes: None,
            timing_p2_max: None,
            timing_p3_min: None,
        })
    }

    /// Send KWP2000 request and receive response
    /// Automatically enforces P3min timing between consecutive requests
    pub fn send_request(&mut self, service: u8, data: &[u8]) -> Result<KwpResponse> {
        if !self.initialized {
            return Err(anyhow!("K-Line not initialized"));
        }

        // Enforce P3min timing: minimum time between end of last response and new request
        // Windows tolerance: allow 3ms slack due to ~15ms timer resolution
        #[cfg(target_os = "windows")]
        const P3_TOLERANCE_MS: u64 = 3;
        #[cfg(not(target_os = "windows"))]
        const P3_TOLERANCE_MS: u64 = 0;

        if let Some(last_time) = self.last_request_time {
            let elapsed = last_time.elapsed().as_millis() as u64;
            let effective_p3min = self.p3_min_ms.saturating_sub(P3_TOLERANCE_MS);
            if elapsed < effective_p3min {
                let wait_time = effective_p3min - elapsed;
                debug!("P3min: waiting {}ms before next request", wait_time);
                FtdiConnection::delay_ms(wait_time);
            }
        }

        // Build request
        let mut request_data = vec![service];
        request_data.extend_from_slice(data);

        let request = KwpMessage::new(self.tester_address, self.ecu_address, request_data);
        let bytes = request.to_bytes();

        debug!("TX: {:02X?}", bytes);

        // Purge any stale data
        self.ftdi.purge()?;

        // Send request
        let start = Instant::now();
        self.ftdi.write(&bytes)?;

        // Read back echo (K-Line half-duplex)
        let mut echo = vec![0u8; bytes.len()];
        if let Ok(n) = self.ftdi.read(&mut echo, 50) {
            if n > 0 && echo[..n] != bytes[..n] {
                warn!("Echo mismatch in send_request");
            }
        }

        // Read response with timeout (P2 timing handled by ECU)
        let mut response_buf = vec![0u8; 256];
        let read = self.ftdi.read(&mut response_buf, 500)?;

        // Record completion time for P3min calculation
        self.last_request_time = Some(Instant::now());

        let latency = start.elapsed().as_millis();
        debug!("Response received in {}ms", latency);

        if read == 0 {
            return Err(anyhow!("No response from ECU (timeout)"));
        }

        let response_data = &response_buf[..read];
        debug!("RX: {:02X?}", response_data);

        KwpResponse::parse(response_data)
            .ok_or_else(|| anyhow!("Failed to parse response"))
    }

    /// Send TesterPresent to keep connection alive
    /// Returns Ok(true) if ECU responds positively, Ok(false) if no response,
    /// or Err if communication error occurs
    pub fn tester_present(&mut self) -> Result<bool> {
        if !self.initialized {
            debug!("TesterPresent skipped: not initialized");
            return Ok(false);
        }

        // Service 0x3E = TesterPresent
        match self.send_request(0x3E, &[]) {
            Ok(response) => {
                let success = response.service == 0x7E; // Positive response
                if !success {
                    warn!("TesterPresent: unexpected response 0x{:02X}", response.service);
                }
                Ok(success)
            }
            Err(e) => {
                warn!("TesterPresent failed: {}", e);
                // Return error instead of silently converting to Ok(false)
                // This allows caller to detect communication problems
                Err(e)
            }
        }
    }

    /// Read DTCs (Diagnostic Trouble Codes)
    pub fn read_dtcs(&mut self) -> Result<Vec<(u16, u8)>> {
        // Service 0x18 = ReadDTCByStatus
        // Sub-function 0x00 = Report DTC by status mask
        // Status mask 0xFF = All DTCs
        let response = self.send_request(0x18, &[0x00, 0xFF])?;

        if response.service != 0x58 {
            if response.service == 0x7F {
                return Err(anyhow!(
                    "Negative response: 0x{:02X}",
                    response.data.get(1).copied().unwrap_or(0)
                ));
            }
            return Err(anyhow!("Unexpected response service: 0x{:02X}", response.service));
        }

        // Parse DTCs from response
        // Format: [count, high1, low1, status1, high2, low2, status2, ...]
        let mut dtcs = Vec::new();
        let data = &response.data;

        if data.len() >= 2 {
            let mut i = 1; // Skip count byte
            while i + 2 < data.len() {
                let high = data[i];
                let low = data[i + 1];
                let status = data[i + 2];

                if high != 0 || low != 0 {
                    let dtc_code = ((high as u16) << 8) | (low as u16);
                    dtcs.push((dtc_code, status));
                }

                i += 3;
            }
        }

        Ok(dtcs)
    }

    /// Clear DTCs
    pub fn clear_dtcs(&mut self) -> Result<bool> {
        // Service 0x14 = ClearDiagnosticInformation
        let response = self.send_request(0x14, &[0xFF, 0x00])?;
        Ok(response.service == 0x54)
    }

    /// Read OBD-II standard PID (Service 0x01 - Request Current Powertrain Data)
    /// Use this for standard PIDs like RPM (0x0C), Speed (0x0D), Coolant Temp (0x05)
    pub fn read_obd_pid(&mut self, pid: u8) -> Result<Vec<u8>> {
        // Service 0x01 = OBD-II Request Current Powertrain Diagnostic Data
        let response = self.send_request(0x01, &[pid])?;

        // Positive response is 0x41 (0x01 + 0x40)
        if response.service != 0x41 {
            if response.service == 0x7F {
                // Negative response - PID not supported, try manufacturer-specific
                return self.read_manufacturer_pid(pid);
            }
            return Err(anyhow!("Unexpected response: 0x{:02X}", response.service));
        }

        // Response data format: [pid_echo, data...]
        if response.data.len() > 1 {
            Ok(response.data[1..].to_vec())
        } else {
            Ok(vec![])
        }
    }

    /// Read manufacturer-specific PID (Service 0x21 - ReadDataByLocalIdentifier)
    /// Use this for BMW-specific PIDs not in the OBD-II standard
    pub fn read_manufacturer_pid(&mut self, pid: u8) -> Result<Vec<u8>> {
        // Service 0x21 = KWP2000 ReadDataByLocalIdentifier
        let response = self.send_request(0x21, &[pid])?;

        // Positive response is 0x61 (0x21 + 0x40)
        if response.service != 0x61 {
            return Err(anyhow!("Unexpected response: 0x{:02X}", response.service));
        }

        // Response data format: [pid_echo, data...]
        if response.data.len() > 1 {
            Ok(response.data[1..].to_vec())
        } else {
            Ok(vec![])
        }
    }

    /// Read PID value - tries OBD-II standard first, then manufacturer-specific
    /// Standard OBD-II PIDs (0x00-0x65): RPM, Speed, Temp, etc.
    pub fn read_pid(&mut self, pid: u8) -> Result<Vec<u8>> {
        // Try OBD-II standard service first for common PIDs
        self.read_obd_pid(pid)
    }

    /// Check if initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Get connection reference
    pub fn ftdi(&mut self) -> &mut FtdiConnection {
        &mut self.ftdi
    }
}

/// Decode DTC code to standard format (P0123, C0456, etc.)
pub fn decode_dtc(code: u16) -> String {
    let first_char = match (code >> 14) & 0x03 {
        0 => 'P', // Powertrain
        1 => 'C', // Chassis
        2 => 'B', // Body
        3 => 'U', // Network
        _ => '?',
    };

    let second_digit = (code >> 12) & 0x03;
    let third_digit = (code >> 8) & 0x0F;
    let fourth_digit = (code >> 4) & 0x0F;
    let fifth_digit = code & 0x0F;

    format!(
        "{}{}{}{}{}",
        first_char,
        second_digit,
        format!("{:X}", third_digit),
        format!("{:X}", fourth_digit),
        format!("{:X}", fifth_digit)
    )
}

/// DTC status flags
#[derive(Debug, Clone)]
pub struct DtcStatus {
    pub test_failed: bool,
    pub test_failed_this_cycle: bool,
    pub pending: bool,
    pub confirmed: bool,
    pub test_not_completed_since_clear: bool,
    pub test_failed_since_clear: bool,
    pub test_not_completed_this_cycle: bool,
    pub warning_indicator: bool,
}

impl From<u8> for DtcStatus {
    fn from(status: u8) -> Self {
        Self {
            test_failed: (status & 0x01) != 0,
            test_failed_this_cycle: (status & 0x02) != 0,
            pending: (status & 0x04) != 0,
            confirmed: (status & 0x08) != 0,
            test_not_completed_since_clear: (status & 0x10) != 0,
            test_failed_since_clear: (status & 0x20) != 0,
            test_not_completed_this_cycle: (status & 0x40) != 0,
            warning_indicator: (status & 0x80) != 0,
        }
    }
}
