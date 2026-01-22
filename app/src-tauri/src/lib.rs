mod bmw;
mod bmw_commands;
mod commands;
mod dcan;
mod kline;
mod pid_commands;
mod serial;

use serial::SerialState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(SerialState::new())
        .plugin(tauri_plugin_log::Builder::default().build())
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
            // PID/Live data commands
            pid_commands::get_available_pids,
            pid_commands::read_pid_kline,
            pid_commands::read_pids_kline,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
