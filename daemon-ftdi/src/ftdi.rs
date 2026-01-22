//! FTDI D2XX Direct Control Module
//!
//! Provides low-level access to FTDI chips for precise timing control.
//! Uses D2XX drivers instead of VCP for microsecond-level timing.

use anyhow::{anyhow, Result};
use libftd2xx::{Ftdi, FtdiCommon, list_devices as ftdi_list, BitMode};
use std::time::{Duration, Instant};
use std::thread;
use tracing::{debug, info, warn};

/// FTDI device information
#[derive(Debug, Clone)]
pub struct FtdiDevice {
    pub index: usize,
    pub description: String,
    pub serial_number: String,
}

/// FTDI connection handle with precise timing
pub struct FtdiConnection {
    device: Ftdi,
    baud_rate: u32,
    connected: bool,
}

/// List all available FTDI devices
pub fn list_devices() -> Result<Vec<FtdiDevice>> {
    let devices = ftdi_list()?;

    Ok(devices
        .into_iter()
        .enumerate()
        .map(|(i, info)| FtdiDevice {
            index: i,
            description: info.description,
            serial_number: info.serial_number,
        })
        .collect())
}

impl FtdiConnection {
    /// Open FTDI device by index
    pub fn open(index: i32) -> Result<Self> {
        info!("Opening FTDI device index {}...", index);

        // Use with_index to open by index (0 = first device)
        let mut device = Ftdi::with_index(index)?;

        // Reset device
        device.reset()?;

        // Set timeouts (read, write)
        device.set_timeouts(Duration::from_millis(1000), Duration::from_millis(1000))?;

        // Purge buffers
        device.purge_all()?;

        info!("FTDI device opened successfully");

        Ok(Self {
            device,
            baud_rate: 10400,
            connected: true,
        })
    }

    /// Open FTDI device by serial number
    pub fn open_by_serial(serial: &str) -> Result<Self> {
        info!("Opening FTDI device with serial {}...", serial);

        let mut device = Ftdi::with_serial_number(serial)?;

        device.reset()?;
        device.set_timeouts(Duration::from_millis(1000), Duration::from_millis(1000))?;
        device.purge_all()?;

        Ok(Self {
            device,
            baud_rate: 10400,
            connected: true,
        })
    }

    /// Set baud rate with high precision
    pub fn set_baud_rate(&mut self, baud: u32) -> Result<()> {
        debug!("Setting baud rate to {}", baud);
        self.device.set_baud_rate(baud)?;
        self.baud_rate = baud;
        Ok(())
    }

    /// Configure for K-Line communication (10400 baud, 8N1)
    pub fn configure_kline(&mut self) -> Result<()> {
        info!("Configuring for K-Line (10400 baud, 8N1)");

        self.set_baud_rate(10400)?;

        // 8 data bits, 1 stop bit, no parity
        self.device.set_data_characteristics(
            libftd2xx::BitsPerWord::Bits8,
            libftd2xx::StopBits::Bits1,
            libftd2xx::Parity::No,
        )?;

        // No flow control - use the specific method
        self.device.set_flow_control_none()?;

        // Set latency timer to minimum (1ms) for fastest response
        self.device.set_latency_timer(Duration::from_millis(1))?;

        Ok(())
    }

    /// Configure for D-CAN communication (500 kbaud)
    ///
    /// **WARNING: D-CAN is NOT fully implemented!**
    ///
    /// This function only sets the baud rate to 500 kbaud. Real D-CAN communication
    /// requires the CAN protocol (ISO 11898), not UART serial. This would need:
    /// - A CAN controller (not just FTDI serial)
    /// - CAN frame format with IDs (11-bit or 29-bit)
    /// - ISO-TP (ISO 15765-2) for messages > 8 bytes
    ///
    /// For BMW E60 vehicles built after March 2007, you need a proper K+DCAN cable
    /// with CAN controller hardware, not just FTDI.
    ///
    /// This function is provided for future expansion but will NOT work with D-CAN ECUs.
    pub fn configure_dcan(&mut self) -> Result<()> {
        warn!("D-CAN mode selected but NOT fully implemented! Only K-Line is supported.");
        warn!("BMW E60 after 03/2007 requires real CAN hardware, not UART serial.");
        info!("Configuring for D-CAN (500000 baud) - LIMITED FUNCTIONALITY");
        self.set_baud_rate(500000)?;

        self.device.set_data_characteristics(
            libftd2xx::BitsPerWord::Bits8,
            libftd2xx::StopBits::Bits1,
            libftd2xx::Parity::No,
        )?;

        self.device.set_flow_control_none()?;
        self.device.set_latency_timer(Duration::from_millis(1))?;

        Ok(())
    }

    /// Write bytes with precise timing
    pub fn write(&mut self, data: &[u8]) -> Result<usize> {
        debug!("TX: {:02X?}", data);
        let written = self.device.write(data)?;
        Ok(written)
    }

    /// Read bytes with timeout
    pub fn read(&mut self, buffer: &mut [u8], timeout_ms: u64) -> Result<usize> {
        let start = Instant::now();
        let timeout = Duration::from_millis(timeout_ms);
        let mut total_read = 0;

        while start.elapsed() < timeout && total_read < buffer.len() {
            let queue_status = self.device.queue_status()?;

            if queue_status > 0 {
                let to_read = std::cmp::min(queue_status, buffer.len() - total_read);
                let read = self.device.read(&mut buffer[total_read..total_read + to_read])?;
                total_read += read;
            } else {
                // Small sleep to avoid busy waiting, but keep it minimal
                thread::sleep(Duration::from_micros(100));
            }
        }

        if total_read > 0 {
            debug!("RX: {:02X?}", &buffer[..total_read]);
        }

        Ok(total_read)
    }

    /// Read exact number of bytes with timeout
    pub fn read_exact(&mut self, length: usize, timeout_ms: u64) -> Result<Vec<u8>> {
        let mut buffer = vec![0u8; length];
        let read = self.read(&mut buffer, timeout_ms)?;

        if read < length {
            return Err(anyhow!(
                "Timeout: expected {} bytes, got {}",
                length,
                read
            ));
        }

        Ok(buffer)
    }

    /// Purge RX and TX buffers
    pub fn purge(&mut self) -> Result<()> {
        self.device.purge_all()?;
        Ok(())
    }

    /// High-precision delay for microseconds (Linux version)
    /// Uses thread::sleep for bulk of delay, spin-wait only for final precision
    #[cfg(not(target_os = "windows"))]
    pub fn delay_us(us: u64) {
        let start = Instant::now();
        let target = Duration::from_micros(us);

        // For delays > 2ms, use sleep for most of it to save CPU
        if us > 2000 {
            let sleep_time = Duration::from_micros(us.saturating_sub(1000));
            std::thread::sleep(sleep_time);
        }

        // Spin-wait for final precision (last ~1ms or less)
        while start.elapsed() < target {
            std::hint::spin_loop();
        }
    }

    /// High-precision delay for microseconds (Windows version)
    /// Windows has ~15.6ms timer resolution, so we use different thresholds
    #[cfg(target_os = "windows")]
    pub fn delay_us(us: u64) {
        let start = Instant::now();
        let target = Duration::from_micros(us);

        // Windows sleep() has ~15ms resolution, so:
        // - For < 15ms: pure spin-wait (no sleep, it would overshoot)
        // - For >= 15ms: sleep with 5ms safety margin, then spin-wait
        if us >= 15000 {
            let sleep_time = Duration::from_micros(us.saturating_sub(5000));
            std::thread::sleep(sleep_time);
        }

        // Spin-wait for remaining time (or all of it if < 15ms)
        while start.elapsed() < target {
            std::hint::spin_loop();
        }
    }

    /// High-precision delay for milliseconds (Linux version)
    /// Uses thread::sleep for bulk of delay, spin-wait only for final precision
    #[cfg(not(target_os = "windows"))]
    pub fn delay_ms(ms: u64) {
        let start = Instant::now();
        let target = Duration::from_millis(ms);

        // For delays > 2ms, sleep for most of it
        if ms > 2 {
            let sleep_time = Duration::from_millis(ms.saturating_sub(1));
            std::thread::sleep(sleep_time);
        }

        // Spin-wait for final millisecond precision
        while start.elapsed() < target {
            std::hint::spin_loop();
        }
    }

    /// High-precision delay for milliseconds (Windows version)
    /// Windows has ~15.6ms timer resolution, requires larger safety margins
    #[cfg(target_os = "windows")]
    pub fn delay_ms(ms: u64) {
        let start = Instant::now();
        let target = Duration::from_millis(ms);

        // Windows sleep() has ~15ms resolution, so:
        // - For < 20ms: pure spin-wait to avoid overshoot
        // - For >= 20ms: sleep with 5ms safety margin, then spin-wait
        if ms >= 20 {
            let sleep_time = Duration::from_millis(ms.saturating_sub(5));
            std::thread::sleep(sleep_time);
        }

        // Spin-wait for remaining time
        while start.elapsed() < target {
            std::hint::spin_loop();
        }
    }

    /// Send byte at 5 baud (for K-Line slow init per ISO 9141-2)
    /// Format: START + 7 data bits + ODD PARITY + STOP = 10 bits at 200ms each
    pub fn send_5baud(&mut self, byte: u8) -> Result<()> {
        info!("Sending 0x{:02X} at 5 baud (ISO 9141-2 format)...", byte);

        // At 5 baud, each bit takes 200ms (1/5 = 0.2s = 200ms)
        // ISO 9141-2 format: start + 7 data bits + odd parity + stop = 10 bits = 2 seconds

        // Calculate odd parity for bits 0-6
        let data_bits = byte & 0x7F; // Only lower 7 bits
        let ones_count = data_bits.count_ones();
        let parity_bit = if ones_count % 2 == 0 { 1u8 } else { 0u8 }; // Odd parity

        debug!(
            "5-baud: data=0x{:02X}, 7-bit=0x{:02X}, ones={}, parity={}",
            byte, data_bits, ones_count, parity_bit
        );

        // Set to asynchronous bit-bang mode
        // Mask 0x01 = only TXD pin is output
        self.device.set_bit_mode(0x01, BitMode::AsyncBitbang)?;

        // Start bit (LOW)
        self.device.write(&[0x00])?;
        Self::delay_ms(200);

        // 7 Data bits (LSB first) - bits 0-6 only
        for i in 0..7 {
            let bit = (data_bits >> i) & 0x01;
            self.device.write(&[bit])?;
            Self::delay_ms(200);
        }

        // Parity bit (odd parity)
        self.device.write(&[parity_bit])?;
        Self::delay_ms(200);

        // Stop bit (HIGH)
        self.device.write(&[0x01])?;
        Self::delay_ms(200);

        // Return to normal UART mode (reset bit mode)
        self.device.set_bit_mode(0x00, BitMode::Reset)?;

        // Reconfigure for 10400 baud
        self.configure_kline()?;

        info!("5-baud transmission complete");
        Ok(())
    }

    /// Break signal for fast init
    pub fn send_break(&mut self, duration_ms: u64) -> Result<()> {
        debug!("Sending break signal for {}ms", duration_ms);
        self.device.set_break_on()?;
        Self::delay_ms(duration_ms);
        self.device.set_break_off()?;
        Ok(())
    }

    /// Close the connection
    pub fn close(&mut self) -> Result<()> {
        if self.connected {
            info!("Closing FTDI connection");
            self.device.close()?;
            self.connected = false;
        }
        Ok(())
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.connected
    }

    /// Get current baud rate
    pub fn baud_rate(&self) -> u32 {
        self.baud_rate
    }
}

impl Drop for FtdiConnection {
    fn drop(&mut self) {
        let _ = self.close();
    }
}
