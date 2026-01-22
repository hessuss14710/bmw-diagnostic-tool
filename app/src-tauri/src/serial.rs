use serde::{Deserialize, Serialize};
use serialport::{available_ports, SerialPortType};
use std::sync::Mutex;
use std::time::Duration;

/// Information about a serial port
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortInfo {
    pub name: String,
    pub port_type: String,
    pub vid: Option<u16>,
    pub pid: Option<u16>,
    pub manufacturer: Option<String>,
    pub product: Option<String>,
    pub serial_number: Option<String>,
    pub is_ftdi: bool,
}

/// Connection state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Error(String),
}

/// Serial connection manager
pub struct SerialManager {
    port: Option<Box<dyn serialport::SerialPort>>,
    state: ConnectionState,
    current_port: Option<String>,
    baud_rate: u32,
}

impl SerialManager {
    pub fn new() -> Self {
        Self {
            port: None,
            state: ConnectionState::Disconnected,
            current_port: None,
            baud_rate: 10400, // K-Line default baud rate
        }
    }

    /// List all available serial ports
    pub fn list_ports() -> Result<Vec<PortInfo>, String> {
        let ports = available_ports().map_err(|e| format!("Failed to list ports: {}", e))?;

        let port_infos: Vec<PortInfo> = ports
            .into_iter()
            .map(|p| {
                let (port_type, vid, pid, manufacturer, product, serial_number, is_ftdi) =
                    match &p.port_type {
                        SerialPortType::UsbPort(usb) => {
                            // FTDI VID is 0x0403
                            let is_ftdi = usb.vid == 0x0403;
                            (
                                "USB".to_string(),
                                Some(usb.vid),
                                Some(usb.pid),
                                usb.manufacturer.clone(),
                                usb.product.clone(),
                                usb.serial_number.clone(),
                                is_ftdi,
                            )
                        }
                        SerialPortType::PciPort => {
                            ("PCI".to_string(), None, None, None, None, None, false)
                        }
                        SerialPortType::BluetoothPort => {
                            ("Bluetooth".to_string(), None, None, None, None, None, false)
                        }
                        SerialPortType::Unknown => {
                            ("Unknown".to_string(), None, None, None, None, None, false)
                        }
                    };

                PortInfo {
                    name: p.port_name,
                    port_type,
                    vid,
                    pid,
                    manufacturer,
                    product,
                    serial_number,
                    is_ftdi,
                }
            })
            .collect();

        Ok(port_infos)
    }

    /// Connect to a serial port
    pub fn connect(&mut self, port_name: &str, baud_rate: u32) -> Result<(), String> {
        // Disconnect if already connected
        if self.port.is_some() {
            self.disconnect()?;
        }

        self.state = ConnectionState::Connecting;
        self.baud_rate = baud_rate;

        let port = serialport::new(port_name, baud_rate)
            .timeout(Duration::from_millis(1000))
            .data_bits(serialport::DataBits::Eight)
            .parity(serialport::Parity::None)
            .stop_bits(serialport::StopBits::One)
            .flow_control(serialport::FlowControl::None)
            .open()
            .map_err(|e| {
                self.state = ConnectionState::Error(e.to_string());
                format!("Failed to open port {}: {}", port_name, e)
            })?;

        self.port = Some(port);
        self.current_port = Some(port_name.to_string());
        self.state = ConnectionState::Connected;

        log::info!("Connected to {} at {} baud", port_name, baud_rate);
        Ok(())
    }

    /// Disconnect from the current port
    pub fn disconnect(&mut self) -> Result<(), String> {
        if let Some(port) = self.port.take() {
            drop(port);
            log::info!("Disconnected from {:?}", self.current_port);
        }
        self.current_port = None;
        self.state = ConnectionState::Disconnected;
        Ok(())
    }

    /// Get current connection state
    pub fn get_state(&self) -> ConnectionState {
        self.state.clone()
    }

    /// Get current port name
    pub fn get_current_port(&self) -> Option<String> {
        self.current_port.clone()
    }

    /// Send data to the serial port
    pub fn write(&mut self, data: &[u8]) -> Result<usize, String> {
        let port = self
            .port
            .as_mut()
            .ok_or_else(|| "Not connected".to_string())?;

        port.write(data)
            .map_err(|e| format!("Write error: {}", e))
    }

    /// Read data from the serial port
    pub fn read(&mut self, buffer: &mut [u8]) -> Result<usize, String> {
        let port = self
            .port
            .as_mut()
            .ok_or_else(|| "Not connected".to_string())?;

        port.read(buffer)
            .map_err(|e| format!("Read error: {}", e))
    }

    /// Read with timeout (non-blocking)
    pub fn read_available(&mut self) -> Result<Vec<u8>, String> {
        let port = self
            .port
            .as_mut()
            .ok_or_else(|| "Not connected".to_string())?;

        let bytes_to_read = port
            .bytes_to_read()
            .map_err(|e| format!("Error checking available bytes: {}", e))?;

        if bytes_to_read == 0 {
            return Ok(Vec::new());
        }

        let mut buffer = vec![0u8; bytes_to_read as usize];
        let bytes_read = port
            .read(&mut buffer)
            .map_err(|e| format!("Read error: {}", e))?;

        buffer.truncate(bytes_read);
        Ok(buffer)
    }

    /// Set DTR (Data Terminal Ready) line
    pub fn set_dtr(&mut self, level: bool) -> Result<(), String> {
        let port = self
            .port
            .as_mut()
            .ok_or_else(|| "Not connected".to_string())?;

        port.write_data_terminal_ready(level)
            .map_err(|e| format!("Failed to set DTR: {}", e))
    }

    /// Set RTS (Request To Send) line
    pub fn set_rts(&mut self, level: bool) -> Result<(), String> {
        let port = self
            .port
            .as_mut()
            .ok_or_else(|| "Not connected".to_string())?;

        port.write_request_to_send(level)
            .map_err(|e| format!("Failed to set RTS: {}", e))
    }

    /// Set baud rate
    pub fn set_baud_rate(&mut self, baud_rate: u32) -> Result<(), String> {
        let port = self
            .port
            .as_mut()
            .ok_or_else(|| "Not connected".to_string())?;

        port.set_baud_rate(baud_rate)
            .map_err(|e| format!("Failed to set baud rate: {}", e))?;

        self.baud_rate = baud_rate;
        Ok(())
    }

    /// Clear buffers
    pub fn clear_buffers(&mut self) -> Result<(), String> {
        let port = self
            .port
            .as_mut()
            .ok_or_else(|| "Not connected".to_string())?;

        port.clear(serialport::ClearBuffer::All)
            .map_err(|e| format!("Failed to clear buffers: {}", e))
    }

    /// Get mutable reference to the port for protocol handlers
    pub fn get_port_mut(&mut self) -> Option<&mut Box<dyn serialport::SerialPort>> {
        self.port.as_mut()
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.port.is_some()
    }
}

/// Thread-safe wrapper for SerialManager
pub struct SerialState(pub Mutex<SerialManager>);

impl SerialState {
    pub fn new() -> Self {
        Self(Mutex::new(SerialManager::new()))
    }
}
