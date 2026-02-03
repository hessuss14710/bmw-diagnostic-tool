//! Input validation module for BMW diagnostic commands
//!
//! Provides centralized validation logic for all user inputs to prevent
//! invalid data from reaching ECU communication functions.

use crate::constants::{addresses, baud, diesel_categories, dpf_routines, limits, pid_ranges, uds};
use std::collections::HashSet;

// ============================================================================
// ERROR TYPES
// ============================================================================

/// Validation error with detailed message
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.field, self.message)
    }
}

impl std::error::Error for ValidationError {}

impl ValidationError {
    pub fn new(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
        }
    }
}

/// Result type for validation operations
pub type ValidationResult<T> = Result<T, ValidationError>;

// ============================================================================
// ECU ADDRESS VALIDATION
// ============================================================================

/// Validates an ECU address is in the allowed list
pub fn validate_ecu_address(addr: u8) -> ValidationResult<u8> {
    if addresses::VALID_ECUS.contains(&addr) {
        Ok(addr)
    } else {
        Err(ValidationError::new(
            "ecu_address",
            format!(
                "Invalid ECU address 0x{:02X}. Valid addresses: {:02X?}",
                addr, addresses::VALID_ECUS
            ),
        ))
    }
}

/// Validates ECU address with optional default
pub fn validate_ecu_address_or_default(addr: Option<u8>) -> ValidationResult<u8> {
    match addr {
        Some(a) => validate_ecu_address(a),
        None => Ok(addresses::DEFAULT_ECU),
    }
}

// ============================================================================
// BAUD RATE VALIDATION
// ============================================================================

/// Validates baud rate is within acceptable range
pub fn validate_baud_rate(rate: u32) -> ValidationResult<u32> {
    if rate >= baud::MIN_BAUD && rate <= baud::MAX_BAUD {
        Ok(rate)
    } else {
        Err(ValidationError::new(
            "baud_rate",
            format!(
                "Invalid baud rate {}. Must be between {} and {}",
                rate, baud::MIN_BAUD, baud::MAX_BAUD
            ),
        ))
    }
}

/// Validates baud rate with optional default
pub fn validate_baud_rate_or_default(rate: Option<u32>) -> ValidationResult<u32> {
    match rate {
        Some(r) => validate_baud_rate(r),
        None => Ok(baud::KLINE_DEFAULT),
    }
}

// ============================================================================
// PID/DID VALIDATION
// ============================================================================

/// Validates a single PID is within valid ranges
pub fn validate_pid(pid: u16) -> ValidationResult<u16> {
    // Check if restricted
    if pid_ranges::RESTRICTED.contains(&pid) {
        return Err(ValidationError::new(
            "pid",
            format!("PID 0x{:04X} is restricted", pid),
        ));
    }

    // Check if in any valid range
    let is_valid = pid_ranges::VALID_RANGES
        .iter()
        .any(|(start, end)| pid >= *start && pid <= *end);

    if is_valid {
        Ok(pid)
    } else {
        Err(ValidationError::new(
            "pid",
            format!(
                "PID 0x{:04X} is not in valid ranges: {:?}",
                pid, pid_ranges::VALID_RANGES
            ),
        ))
    }
}

/// Validates a DID (same rules as PID but different name for clarity)
pub fn validate_did(did: u16) -> ValidationResult<u16> {
    validate_pid(did).map_err(|e| ValidationError::new("did", e.message))
}

/// Validates a list of PIDs
pub fn validate_pids(pids: &[u16]) -> ValidationResult<()> {
    if pids.is_empty() {
        return Err(ValidationError::new("pids", "PID list cannot be empty"));
    }

    if pids.len() > limits::MAX_PIDS_PER_REQUEST {
        return Err(ValidationError::new(
            "pids",
            format!(
                "Too many PIDs: {} (max: {})",
                pids.len(),
                limits::MAX_PIDS_PER_REQUEST
            ),
        ));
    }

    // Check for duplicates
    let mut seen = HashSet::new();
    for (idx, pid) in pids.iter().enumerate() {
        if !seen.insert(pid) {
            return Err(ValidationError::new(
                "pids",
                format!("Duplicate PID at index {}: 0x{:04X}", idx, pid),
            ));
        }
        // Validate each PID
        validate_pid(*pid)?;
    }

    Ok(())
}

/// Validates a list of DIDs
pub fn validate_dids(dids: &[u16]) -> ValidationResult<()> {
    if dids.is_empty() {
        return Err(ValidationError::new("dids", "DID list cannot be empty"));
    }

    if dids.len() > limits::MAX_DIDS_PER_REQUEST {
        return Err(ValidationError::new(
            "dids",
            format!(
                "Too many DIDs: {} (max: {})",
                dids.len(),
                limits::MAX_DIDS_PER_REQUEST
            ),
        ));
    }

    let mut seen = HashSet::new();
    for (idx, did) in dids.iter().enumerate() {
        if !seen.insert(did) {
            return Err(ValidationError::new(
                "dids",
                format!("Duplicate DID at index {}: 0x{:04X}", idx, did),
            ));
        }
        validate_did(*did)?;
    }

    Ok(())
}

// ============================================================================
// ROUTINE VALIDATION
// ============================================================================

/// Validates a routine ID is in the allowed list
pub fn validate_routine_id(routine_id: u16) -> ValidationResult<u16> {
    if dpf_routines::VALID_ROUTINES.contains(&routine_id) {
        Ok(routine_id)
    } else {
        Err(ValidationError::new(
            "routine_id",
            format!(
                "Invalid routine ID 0x{:04X}. Valid IDs: {:04X?}",
                routine_id, dpf_routines::VALID_ROUTINES
            ),
        ))
    }
}

/// Validates a routine sub-function
pub fn validate_sub_function(sub_fn: u8) -> ValidationResult<u8> {
    const VALID: &[u8] = &[
        uds::routine::START,
        uds::routine::STOP,
        uds::routine::REQUEST_RESULTS,
    ];

    if VALID.contains(&sub_fn) {
        Ok(sub_fn)
    } else {
        Err(ValidationError::new(
            "sub_function",
            format!("Invalid sub-function 0x{:02X}. Valid: {:02X?}", sub_fn, VALID),
        ))
    }
}

/// Validates routine data
pub fn validate_routine_data(data: &[u8]) -> ValidationResult<()> {
    if data.len() > limits::MAX_ROUTINE_DATA_SIZE {
        return Err(ValidationError::new(
            "data",
            format!(
                "Data too large: {} bytes (max: {})",
                data.len(),
                limits::MAX_ROUTINE_DATA_SIZE
            ),
        ));
    }

    // Check for potentially dangerous bytes
    for (idx, byte) in data.iter().enumerate() {
        if *byte == 0x7F {
            return Err(ValidationError::new(
                "data",
                format!("Invalid byte at offset {}: 0x{:02X} (negative response marker)", idx, byte),
            ));
        }
    }

    Ok(())
}

// ============================================================================
// STRING VALIDATION
// ============================================================================

/// Validates a diesel category string
pub fn validate_diesel_category(category: &str) -> ValidationResult<String> {
    let normalized = category.trim().to_lowercase();

    if diesel_categories::ALL_CATEGORIES.contains(&normalized.as_str()) {
        Ok(normalized)
    } else {
        Err(ValidationError::new(
            "category",
            format!(
                "Invalid category '{}'. Valid: {:?}",
                normalized, diesel_categories::ALL_CATEGORIES
            ),
        ))
    }
}

/// Validates a hex string for serial_send_hex
pub fn validate_hex_string(hex: &str) -> ValidationResult<Vec<u8>> {
    if hex.is_empty() {
        return Err(ValidationError::new("hex_data", "Hex string cannot be empty"));
    }

    if hex.len() > limits::MAX_HEX_STRING_LENGTH {
        return Err(ValidationError::new(
            "hex_data",
            format!(
                "Hex string too long: {} chars (max: {})",
                hex.len(),
                limits::MAX_HEX_STRING_LENGTH
            ),
        ));
    }

    // Filter to only hex digits
    let hex_clean: String = hex.chars().filter(|c| c.is_ascii_hexdigit()).collect();

    if hex_clean.len() % 2 != 0 {
        return Err(ValidationError::new(
            "hex_data",
            "Hex string must have even number of digits",
        ));
    }

    // Parse to bytes
    let bytes: Result<Vec<u8>, _> = (0..hex_clean.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex_clean[i..i + 2], 16))
        .collect();

    bytes.map_err(|e| ValidationError::new("hex_data", format!("Invalid hex: {}", e)))
}

// ============================================================================
// SESSION VALIDATION
// ============================================================================

/// Validates a session type
pub fn validate_session_type(session_type: u8) -> ValidationResult<u8> {
    const VALID: &[u8] = &[
        uds::session::DEFAULT,
        uds::session::PROGRAMMING,
        uds::session::EXTENDED,
    ];

    if VALID.contains(&session_type) {
        Ok(session_type)
    } else {
        Err(ValidationError::new(
            "session_type",
            format!("Invalid session type 0x{:02X}. Valid: {:02X?}", session_type, VALID),
        ))
    }
}

/// Validates a security level
pub fn validate_security_level(level: u8) -> ValidationResult<u8> {
    // Odd numbers 0x01-0x41 are valid seed requests
    // Even numbers 0x02-0x42 are key responses
    if level >= 0x01 && level <= 0x42 && (level % 2 == 1 || level % 2 == 0) {
        Ok(level)
    } else {
        Err(ValidationError::new(
            "security_level",
            format!("Invalid security level 0x{:02X}", level),
        ))
    }
}

// ============================================================================
// DEVICE INDEX VALIDATION
// ============================================================================

/// Validates a device index
pub fn validate_device_index(index: i32, max_devices: usize) -> ValidationResult<usize> {
    if index < 0 {
        return Err(ValidationError::new(
            "device_index",
            "Device index must be non-negative",
        ));
    }

    let idx = index as usize;
    if idx >= max_devices {
        return Err(ValidationError::new(
            "device_index",
            format!(
                "Device index {} out of range (max: {})",
                idx,
                max_devices.saturating_sub(1)
            ),
        ));
    }

    Ok(idx)
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_ecu_address_valid() {
        assert!(validate_ecu_address(0x12).is_ok());
        assert!(validate_ecu_address(0x32).is_ok());
    }

    #[test]
    fn test_validate_ecu_address_invalid() {
        assert!(validate_ecu_address(0x99).is_err());
        assert!(validate_ecu_address(0x00).is_err());
    }

    #[test]
    fn test_validate_baud_rate_valid() {
        assert!(validate_baud_rate(10400).is_ok());
        assert!(validate_baud_rate(500000).is_ok());
    }

    #[test]
    fn test_validate_baud_rate_invalid() {
        assert!(validate_baud_rate(0).is_err());
        assert!(validate_baud_rate(10_000_000).is_err());
    }

    #[test]
    fn test_validate_pid_valid() {
        assert!(validate_pid(0x0C).is_ok()); // RPM
        assert!(validate_pid(0x05).is_ok()); // Coolant temp
    }

    #[test]
    fn test_validate_pid_restricted() {
        assert!(validate_pid(0x0000).is_err());
        assert!(validate_pid(0xFFFF).is_err());
    }

    #[test]
    fn test_validate_pids_empty() {
        assert!(validate_pids(&[]).is_err());
    }

    #[test]
    fn test_validate_pids_duplicates() {
        assert!(validate_pids(&[0x0C, 0x0D, 0x0C]).is_err());
    }

    #[test]
    fn test_validate_pids_too_many() {
        let pids: Vec<u16> = (0..20).collect();
        assert!(validate_pids(&pids).is_err());
    }

    #[test]
    fn test_validate_diesel_category_valid() {
        assert!(validate_diesel_category("fuel_system").is_ok());
        assert!(validate_diesel_category("  TURBO  ").is_ok()); // Case insensitive, trimmed
    }

    #[test]
    fn test_validate_diesel_category_invalid() {
        assert!(validate_diesel_category("invalid_category").is_err());
    }

    #[test]
    fn test_validate_hex_string_valid() {
        let result = validate_hex_string("0102030405");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![0x01, 0x02, 0x03, 0x04, 0x05]);
    }

    #[test]
    fn test_validate_hex_string_with_spaces() {
        let result = validate_hex_string("01 02 03");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![0x01, 0x02, 0x03]);
    }

    #[test]
    fn test_validate_hex_string_empty() {
        assert!(validate_hex_string("").is_err());
    }

    #[test]
    fn test_validate_device_index_valid() {
        assert!(validate_device_index(0, 3).is_ok());
        assert!(validate_device_index(2, 3).is_ok());
    }

    #[test]
    fn test_validate_device_index_invalid() {
        assert!(validate_device_index(-1, 3).is_err());
        assert!(validate_device_index(5, 3).is_err());
    }
}
