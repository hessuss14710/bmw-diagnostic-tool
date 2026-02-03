//! Tauri commands for database operations

use crate::database::{
    Database, DatabaseStats, DiagnosticSession, NewDtc, NewSession, NewVehicle, Setting,
    StoredDtc, Vehicle,
};
use std::sync::Mutex;
use tauri::State;

/// Database state for Tauri
pub struct DbState(pub Mutex<Option<Database>>);

// ============================================================================
// VEHICLE COMMANDS
// ============================================================================

/// Get all vehicles
#[tauri::command]
pub fn db_get_vehicles(state: State<DbState>) -> Result<Vec<Vehicle>, String> {
    let guard = state.0.lock().map_err(|e| format!("Lock error: {}", e))?;
    let db = guard.as_ref().ok_or("Database not initialized")?;
    db.get_all_vehicles().map_err(|e| format!("Database error: {}", e))
}

/// Get a vehicle by ID
#[tauri::command]
pub fn db_get_vehicle(state: State<DbState>, id: i64) -> Result<Option<Vehicle>, String> {
    let guard = state.0.lock().map_err(|e| format!("Lock error: {}", e))?;
    let db = guard.as_ref().ok_or("Database not initialized")?;
    db.get_vehicle(id).map_err(|e| format!("Database error: {}", e))
}

/// Get a vehicle by VIN
#[tauri::command]
pub fn db_get_vehicle_by_vin(state: State<DbState>, vin: String) -> Result<Option<Vehicle>, String> {
    let guard = state.0.lock().map_err(|e| format!("Lock error: {}", e))?;
    let db = guard.as_ref().ok_or("Database not initialized")?;
    db.get_vehicle_by_vin(&vin).map_err(|e| format!("Database error: {}", e))
}

/// Create a new vehicle
#[tauri::command]
pub fn db_create_vehicle(state: State<DbState>, vehicle: NewVehicle) -> Result<i64, String> {
    let guard = state.0.lock().map_err(|e| format!("Lock error: {}", e))?;
    let db = guard.as_ref().ok_or("Database not initialized")?;
    db.create_vehicle(&vehicle).map_err(|e| format!("Database error: {}", e))
}

/// Update a vehicle
#[tauri::command]
pub fn db_update_vehicle(
    state: State<DbState>,
    id: i64,
    vehicle: NewVehicle,
) -> Result<bool, String> {
    let guard = state.0.lock().map_err(|e| format!("Lock error: {}", e))?;
    let db = guard.as_ref().ok_or("Database not initialized")?;
    db.update_vehicle(id, &vehicle).map_err(|e| format!("Database error: {}", e))
}

/// Delete a vehicle
#[tauri::command]
pub fn db_delete_vehicle(state: State<DbState>, id: i64) -> Result<bool, String> {
    let guard = state.0.lock().map_err(|e| format!("Lock error: {}", e))?;
    let db = guard.as_ref().ok_or("Database not initialized")?;
    db.delete_vehicle(id).map_err(|e| format!("Database error: {}", e))
}

// ============================================================================
// SESSION COMMANDS
// ============================================================================

/// Create a new diagnostic session
#[tauri::command]
pub fn db_create_session(state: State<DbState>, session: NewSession) -> Result<i64, String> {
    let guard = state.0.lock().map_err(|e| format!("Lock error: {}", e))?;
    let db = guard.as_ref().ok_or("Database not initialized")?;
    db.create_session(&session).map_err(|e| format!("Database error: {}", e))
}

/// Get sessions for a vehicle
#[tauri::command]
pub fn db_get_sessions_for_vehicle(
    state: State<DbState>,
    vehicle_id: i64,
) -> Result<Vec<DiagnosticSession>, String> {
    let guard = state.0.lock().map_err(|e| format!("Lock error: {}", e))?;
    let db = guard.as_ref().ok_or("Database not initialized")?;
    db.get_sessions_for_vehicle(vehicle_id)
        .map_err(|e| format!("Database error: {}", e))
}

/// Get recent sessions
#[tauri::command]
pub fn db_get_recent_sessions(
    state: State<DbState>,
    limit: i32,
) -> Result<Vec<DiagnosticSession>, String> {
    let guard = state.0.lock().map_err(|e| format!("Lock error: {}", e))?;
    let db = guard.as_ref().ok_or("Database not initialized")?;
    db.get_recent_sessions(limit).map_err(|e| format!("Database error: {}", e))
}

/// Delete a session
#[tauri::command]
pub fn db_delete_session(state: State<DbState>, id: i64) -> Result<bool, String> {
    let guard = state.0.lock().map_err(|e| format!("Lock error: {}", e))?;
    let db = guard.as_ref().ok_or("Database not initialized")?;
    db.delete_session(id).map_err(|e| format!("Database error: {}", e))
}

// ============================================================================
// DTC COMMANDS
// ============================================================================

/// Add DTCs to a session
#[tauri::command]
pub fn db_add_dtcs(state: State<DbState>, dtcs: Vec<NewDtc>) -> Result<(), String> {
    let guard = state.0.lock().map_err(|e| format!("Lock error: {}", e))?;
    let db = guard.as_ref().ok_or("Database not initialized")?;
    db.add_dtcs(&dtcs).map_err(|e| format!("Database error: {}", e))
}

/// Get DTCs for a session
#[tauri::command]
pub fn db_get_dtcs_for_session(
    state: State<DbState>,
    session_id: i64,
) -> Result<Vec<StoredDtc>, String> {
    let guard = state.0.lock().map_err(|e| format!("Lock error: {}", e))?;
    let db = guard.as_ref().ok_or("Database not initialized")?;
    db.get_dtcs_for_session(session_id)
        .map_err(|e| format!("Database error: {}", e))
}

/// Get DTC history for a vehicle
#[tauri::command]
pub fn db_get_dtc_history(
    state: State<DbState>,
    vehicle_id: i64,
) -> Result<Vec<StoredDtc>, String> {
    let guard = state.0.lock().map_err(|e| format!("Lock error: {}", e))?;
    let db = guard.as_ref().ok_or("Database not initialized")?;
    db.get_dtc_history_for_vehicle(vehicle_id)
        .map_err(|e| format!("Database error: {}", e))
}

// ============================================================================
// SETTINGS COMMANDS
// ============================================================================

/// Get a setting
#[tauri::command]
pub fn db_get_setting(state: State<DbState>, key: String) -> Result<Option<String>, String> {
    let guard = state.0.lock().map_err(|e| format!("Lock error: {}", e))?;
    let db = guard.as_ref().ok_or("Database not initialized")?;
    db.get_setting(&key).map_err(|e| format!("Database error: {}", e))
}

/// Set a setting
#[tauri::command]
pub fn db_set_setting(state: State<DbState>, key: String, value: String) -> Result<(), String> {
    let guard = state.0.lock().map_err(|e| format!("Lock error: {}", e))?;
    let db = guard.as_ref().ok_or("Database not initialized")?;
    db.set_setting(&key, &value).map_err(|e| format!("Database error: {}", e))
}

/// Get all settings
#[tauri::command]
pub fn db_get_all_settings(state: State<DbState>) -> Result<Vec<Setting>, String> {
    let guard = state.0.lock().map_err(|e| format!("Lock error: {}", e))?;
    let db = guard.as_ref().ok_or("Database not initialized")?;
    db.get_all_settings().map_err(|e| format!("Database error: {}", e))
}

// ============================================================================
// EXPORT/STATS COMMANDS
// ============================================================================

/// Export all data as JSON
#[tauri::command]
pub fn db_export_all(state: State<DbState>) -> Result<String, String> {
    let guard = state.0.lock().map_err(|e| format!("Lock error: {}", e))?;
    let db = guard.as_ref().ok_or("Database not initialized")?;
    db.export_all().map_err(|e| format!("Database error: {}", e))
}

/// Get database statistics
#[tauri::command]
pub fn db_get_stats(state: State<DbState>) -> Result<DatabaseStats, String> {
    let guard = state.0.lock().map_err(|e| format!("Lock error: {}", e))?;
    let db = guard.as_ref().ok_or("Database not initialized")?;
    db.get_stats().map_err(|e| format!("Database error: {}", e))
}
