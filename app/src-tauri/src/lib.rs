mod bmw;
mod bmw_commands;
mod commands;
pub mod constants;
pub mod database;
mod db_commands;
mod dcan;
mod kline;
mod pid_commands;
mod serial;
pub mod validators;

use database::Database;
use db_commands::DbState;
use serial::SerialState;
use std::sync::Mutex;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(SerialState::new())
        .manage(DbState(Mutex::new(None)))
        .plugin(tauri_plugin_log::Builder::default().build())
        .setup(|app| {
            // Initialize database
            let app_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to get app data directory");

            // Create directory if it doesn't exist
            std::fs::create_dir_all(&app_dir).expect("Failed to create app data directory");

            let db_path = app_dir.join("bmw_diag.db");
            log::info!("Initializing database at: {:?}", db_path);

            match Database::new(db_path) {
                Ok(db) => {
                    let state: tauri::State<DbState> = app.state();
                    *state.0.lock().unwrap() = Some(db);
                    log::info!("Database initialized successfully");
                }
                Err(e) => {
                    log::error!("Failed to initialize database: {}", e);
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Serial port commands
            commands::list_serial_ports,
            commands::serial_connect,
            commands::serial_disconnect,
            commands::serial_status,
            commands::serial_write,
            commands::serial_read,
            commands::serial_send_hex,
            commands::serial_set_dtr,
            commands::serial_set_rts,
            commands::serial_set_baud,
            commands::serial_clear,
            // BMW diagnostic commands
            bmw_commands::bmw_get_ecus,
            bmw_commands::bmw_switch_kline,
            bmw_commands::bmw_switch_dcan,
            bmw_commands::bmw_kline_init,
            bmw_commands::bmw_kline_request,
            bmw_commands::bmw_read_dtcs_kline,
            bmw_commands::bmw_clear_dtcs_kline,
            bmw_commands::bmw_read_ecu_id,
            bmw_commands::bmw_tester_present,
            // DPF (Diesel Particulate Filter) commands
            bmw_commands::bmw_start_session,
            bmw_commands::bmw_security_access,
            bmw_commands::bmw_dpf_read_status,
            bmw_commands::bmw_dpf_reset_ash,
            bmw_commands::bmw_dpf_reset_learned,
            bmw_commands::bmw_dpf_new_installed,
            bmw_commands::bmw_dpf_start_regen,
            bmw_commands::bmw_dpf_stop_regen,
            bmw_commands::bmw_routine_control,
            // DSC (Dynamic Stability Control) commands
            bmw_commands::bmw_dsc_read_dtcs,
            bmw_commands::bmw_dsc_read_wheel_speeds,
            bmw_commands::bmw_dsc_read_sensors,
            bmw_commands::bmw_dsc_bleed_brakes,
            // KOMBI (Instrument Cluster) commands
            bmw_commands::bmw_kombi_read_dtcs,
            bmw_commands::bmw_kombi_read_service,
            bmw_commands::bmw_kombi_reset_service,
            bmw_commands::bmw_kombi_gauge_test,
            bmw_commands::bmw_kombi_read_info,
            // FRM (Footwell Module - Lights) commands
            bmw_commands::bmw_frm_read_dtcs,
            bmw_commands::bmw_frm_read_lamp_status,
            bmw_commands::bmw_frm_lamp_test,
            bmw_commands::bmw_frm_control_lamp,
            // EGS (Electronic Gearbox Control) commands
            bmw_commands::bmw_egs_read_dtcs,
            bmw_commands::bmw_egs_read_status,
            bmw_commands::bmw_egs_reset_adaptations,
            // Multi-ECU commands
            bmw_commands::bmw_read_all_dtcs,
            // D-CAN specific commands
            bmw_commands::bmw_read_dtcs_dcan,
            bmw_commands::bmw_read_dtcs_auto,
            bmw_commands::bmw_detect_protocol,
            bmw_commands::bmw_read_did_dcan,
            bmw_commands::bmw_start_session_dcan,
            bmw_commands::bmw_routine_control_dcan,
            // PID/Live data commands
            pid_commands::get_available_pids,
            pid_commands::read_pid_kline,
            pid_commands::read_pids_kline,
            // Diesel-specific DID commands (E60 520d M47N2/N47)
            pid_commands::get_diesel_pids,
            pid_commands::read_did_kline,
            pid_commands::read_dids_kline,
            pid_commands::read_diesel_category_kline,
            pid_commands::get_diesel_categories,
            // Database commands - Vehicles
            db_commands::db_get_vehicles,
            db_commands::db_get_vehicle,
            db_commands::db_get_vehicle_by_vin,
            db_commands::db_create_vehicle,
            db_commands::db_update_vehicle,
            db_commands::db_delete_vehicle,
            // Database commands - Sessions
            db_commands::db_create_session,
            db_commands::db_get_sessions_for_vehicle,
            db_commands::db_get_recent_sessions,
            db_commands::db_delete_session,
            // Database commands - DTCs
            db_commands::db_add_dtcs,
            db_commands::db_get_dtcs_for_session,
            db_commands::db_get_dtc_history,
            // Database commands - Settings
            db_commands::db_get_setting,
            db_commands::db_set_setting,
            db_commands::db_get_all_settings,
            // Database commands - Export/Stats
            db_commands::db_export_all,
            db_commands::db_get_stats,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
