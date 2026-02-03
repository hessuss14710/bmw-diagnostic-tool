//! Integration tests with realistic BMW diagnostic data
//!
//! These tests simulate complete diagnostic workflows for BMW E60 520d diesel vehicles.

#[cfg(test)]
mod tests {
    use crate::database::{Database, NewDtc, NewSession, NewVehicle};

    // ========================================================================
    // REALISTIC BMW TEST DATA
    // ========================================================================

    fn bmw_e60_520d() -> NewVehicle {
        NewVehicle {
            vin: Some("WBANE71000B123456".to_string()),
            make: "BMW".to_string(),
            model: "520d E60".to_string(),
            year: 2008,
            engine_code: Some("M47TU2D20".to_string()),
            mileage_km: Some(245000),
            notes: Some("2.0L Diesel, 163HP, Manual 6-speed, DPF equipped".to_string()),
        }
    }

    fn bmw_e90_320d() -> NewVehicle {
        NewVehicle {
            vin: Some("WBAPH5C55BA654321".to_string()),
            make: "BMW".to_string(),
            model: "320d E90".to_string(),
            year: 2010,
            engine_code: Some("N47D20".to_string()),
            mileage_km: Some(180000),
            notes: Some("2.0L Diesel, 177HP, Automatic, Twin-turbo".to_string()),
        }
    }

    fn dde_session(vehicle_id: i64) -> NewSession {
        NewSession {
            vehicle_id,
            ecu_id: "0x12".to_string(),
            ecu_name: "DDE (Digital Diesel Electronics)".to_string(),
            protocol: "K-Line KWP2000".to_string(),
            mileage_km: Some(245000),
            notes: Some("DPF warning light on, rough idle at cold start".to_string()),
        }
    }

    fn dsc_session(vehicle_id: i64) -> NewSession {
        NewSession {
            vehicle_id,
            ecu_id: "0x00".to_string(),
            ecu_name: "DSC (Dynamic Stability Control)".to_string(),
            protocol: "D-CAN ISO 15765".to_string(),
            mileage_km: Some(245000),
            notes: Some("ABS light intermittent".to_string()),
        }
    }

    fn kombi_session(vehicle_id: i64) -> NewSession {
        NewSession {
            vehicle_id,
            ecu_id: "0x40".to_string(),
            ecu_name: "KOMBI (Instrument Cluster)".to_string(),
            protocol: "K-Line KWP2000".to_string(),
            mileage_km: Some(245000),
            notes: Some("Service reset after oil change".to_string()),
        }
    }

    // Real BMW DTC codes
    fn dpf_dtcs(session_id: i64) -> Vec<NewDtc> {
        vec![
            NewDtc {
                session_id,
                code: "2AAF".to_string(),
                status: "0x24".to_string(),
                description: Some("Differential pressure sensor - circuit open".to_string()),
                is_pending: false,
                is_confirmed: true,
            },
            NewDtc {
                session_id,
                code: "2AB0".to_string(),
                status: "0x24".to_string(),
                description: Some("DPF soot mass - limit exceeded".to_string()),
                is_pending: true,
                is_confirmed: false,
            },
            NewDtc {
                session_id,
                code: "4B93".to_string(),
                status: "0x27".to_string(),
                description: Some("DPF regeneration - unsuccessful".to_string()),
                is_pending: false,
                is_confirmed: true,
            },
            NewDtc {
                session_id,
                code: "480A".to_string(),
                status: "0x24".to_string(),
                description: Some("Exhaust gas temperature sensor 1 - signal implausible".to_string()),
                is_pending: false,
                is_confirmed: true,
            },
        ]
    }

    fn egr_dtcs(session_id: i64) -> Vec<NewDtc> {
        vec![
            NewDtc {
                session_id,
                code: "2A00".to_string(),
                status: "0x24".to_string(),
                description: Some("EGR valve - stuck open".to_string()),
                is_pending: false,
                is_confirmed: true,
            },
            NewDtc {
                session_id,
                code: "2A82".to_string(),
                status: "0x24".to_string(),
                description: Some("EGR cooler bypass valve - malfunction".to_string()),
                is_pending: true,
                is_confirmed: false,
            },
        ]
    }

    fn injector_dtcs(session_id: i64) -> Vec<NewDtc> {
        vec![
            NewDtc {
                session_id,
                code: "2BAE".to_string(),
                status: "0x24".to_string(),
                description: Some("Injector cylinder 1 - circuit malfunction".to_string()),
                is_pending: false,
                is_confirmed: true,
            },
            NewDtc {
                session_id,
                code: "2BBE".to_string(),
                status: "0x24".to_string(),
                description: Some("Injector cylinder 2 - circuit malfunction".to_string()),
                is_pending: false,
                is_confirmed: true,
            },
        ]
    }

    fn dsc_dtcs(session_id: i64) -> Vec<NewDtc> {
        vec![
            NewDtc {
                session_id,
                code: "5E17".to_string(),
                status: "0x24".to_string(),
                description: Some("ABS wheel speed sensor front left - signal missing".to_string()),
                is_pending: false,
                is_confirmed: true,
            },
            NewDtc {
                session_id,
                code: "5E20".to_string(),
                status: "0x27".to_string(),
                description: Some("Steering angle sensor - not calibrated".to_string()),
                is_pending: true,
                is_confirmed: false,
            },
        ]
    }

    // ========================================================================
    // INTEGRATION TESTS
    // ========================================================================

    #[test]
    fn test_complete_diagnostic_workflow() {
        let db = Database::in_memory().unwrap();

        // 1. Register vehicle
        let vehicle_id = db.create_vehicle(&bmw_e60_520d()).unwrap();
        assert!(vehicle_id > 0);

        // Verify vehicle data
        let vehicle = db.get_vehicle(vehicle_id).unwrap().unwrap();
        assert_eq!(vehicle.make, "BMW");
        assert_eq!(vehicle.model, "520d E60");
        assert_eq!(vehicle.year, 2008);
        assert_eq!(vehicle.engine_code, Some("M47TU2D20".to_string()));

        // 2. Create DDE diagnostic session
        let dde_session_id = db.create_session(&dde_session(vehicle_id)).unwrap();
        assert!(dde_session_id > 0);

        // 3. Store DPF and EGR DTCs
        db.add_dtcs(&dpf_dtcs(dde_session_id)).unwrap();
        db.add_dtcs(&egr_dtcs(dde_session_id)).unwrap();

        // Verify DTCs stored
        let dtcs = db.get_dtcs_for_session(dde_session_id).unwrap();
        assert_eq!(dtcs.len(), 6); // 4 DPF + 2 EGR

        // Check specific DTC codes
        let codes: Vec<&str> = dtcs.iter().map(|d| d.code.as_str()).collect();
        assert!(codes.contains(&"2AAF")); // DPF pressure sensor
        assert!(codes.contains(&"2AB0")); // DPF soot mass
        assert!(codes.contains(&"2A00")); // EGR stuck open

        // 4. Create DSC session
        let dsc_session_id = db.create_session(&dsc_session(vehicle_id)).unwrap();
        db.add_dtcs(&dsc_dtcs(dsc_session_id)).unwrap();

        // 5. Create KOMBI session (no DTCs, just service reset)
        let _kombi_session_id = db.create_session(&kombi_session(vehicle_id)).unwrap();

        // 6. Get all sessions for vehicle
        let sessions = db.get_sessions_for_vehicle(vehicle_id).unwrap();
        assert_eq!(sessions.len(), 3);

        // 7. Get DTC history for vehicle
        let history = db.get_dtc_history_for_vehicle(vehicle_id).unwrap();
        assert_eq!(history.len(), 8); // 6 from DDE + 2 from DSC

        // 8. Check stats
        let stats = db.get_stats().unwrap();
        assert_eq!(stats.vehicle_count, 1);
        assert_eq!(stats.session_count, 3);
        assert_eq!(stats.dtc_count, 8);
    }

    #[test]
    fn test_multiple_vehicles_diagnostic() {
        let db = Database::in_memory().unwrap();

        // Register two vehicles
        let e60_id = db.create_vehicle(&bmw_e60_520d()).unwrap();
        let e90_id = db.create_vehicle(&bmw_e90_320d()).unwrap();

        // Create sessions for each
        let e60_session = db.create_session(&dde_session(e60_id)).unwrap();
        let e90_session = db.create_session(&dde_session(e90_id)).unwrap();

        // Add different DTCs to each
        db.add_dtcs(&dpf_dtcs(e60_session)).unwrap(); // DPF issues on E60
        db.add_dtcs(&injector_dtcs(e90_session)).unwrap(); // Injector issues on E90

        // Verify each vehicle has its own DTCs
        let e60_dtcs = db.get_dtc_history_for_vehicle(e60_id).unwrap();
        let e90_dtcs = db.get_dtc_history_for_vehicle(e90_id).unwrap();

        assert_eq!(e60_dtcs.len(), 4); // DPF codes
        assert_eq!(e90_dtcs.len(), 2); // Injector codes

        // E60 should have DPF codes
        assert!(e60_dtcs.iter().any(|d| d.code == "2AAF"));

        // E90 should have injector codes
        assert!(e90_dtcs.iter().any(|d| d.code == "2BAE"));
    }

    #[test]
    fn test_find_vehicle_by_vin() {
        let db = Database::in_memory().unwrap();

        db.create_vehicle(&bmw_e60_520d()).unwrap();
        db.create_vehicle(&bmw_e90_320d()).unwrap();

        // Find E60 by VIN
        let e60 = db.get_vehicle_by_vin("WBANE71000B123456").unwrap().unwrap();
        assert_eq!(e60.model, "520d E60");
        assert_eq!(e60.engine_code, Some("M47TU2D20".to_string()));

        // Find E90 by VIN
        let e90 = db.get_vehicle_by_vin("WBAPH5C55BA654321").unwrap().unwrap();
        assert_eq!(e90.model, "320d E90");
        assert_eq!(e90.engine_code, Some("N47D20".to_string()));

        // Non-existent VIN
        let not_found = db.get_vehicle_by_vin("NONEXISTENT").unwrap();
        assert!(not_found.is_none());
    }

    #[test]
    fn test_update_vehicle_mileage() {
        let db = Database::in_memory().unwrap();

        let id = db.create_vehicle(&bmw_e60_520d()).unwrap();

        // Update mileage after service
        let mut updated = bmw_e60_520d();
        updated.mileage_km = Some(250000);
        updated.notes = Some("Oil change, DPF regeneration performed".to_string());

        db.update_vehicle(id, &updated).unwrap();

        let vehicle = db.get_vehicle(id).unwrap().unwrap();
        assert_eq!(vehicle.mileage_km, Some(250000));
        assert!(vehicle.notes.unwrap().contains("DPF regeneration"));
    }

    #[test]
    fn test_cascade_delete_vehicle() {
        let db = Database::in_memory().unwrap();

        // Create vehicle with sessions and DTCs
        let vehicle_id = db.create_vehicle(&bmw_e60_520d()).unwrap();
        let session_id = db.create_session(&dde_session(vehicle_id)).unwrap();
        db.add_dtcs(&dpf_dtcs(session_id)).unwrap();

        // Verify data exists
        assert_eq!(db.get_stats().unwrap().dtc_count, 4);

        // Delete vehicle
        db.delete_vehicle(vehicle_id).unwrap();

        // All related data should be gone
        let stats = db.get_stats().unwrap();
        assert_eq!(stats.vehicle_count, 0);
        assert_eq!(stats.session_count, 0);
        assert_eq!(stats.dtc_count, 0);
    }

    #[test]
    fn test_export_full_diagnostic_data() {
        let db = Database::in_memory().unwrap();

        // Create comprehensive data set
        let vehicle_id = db.create_vehicle(&bmw_e60_520d()).unwrap();
        let session_id = db.create_session(&dde_session(vehicle_id)).unwrap();
        db.add_dtcs(&dpf_dtcs(session_id)).unwrap();
        db.set_setting("last_connected_port", "/dev/ttyUSB0").unwrap();
        db.set_setting("theme", "dark").unwrap();
        db.set_setting("language", "es").unwrap();

        // Export
        let export = db.export_all().unwrap();

        // Verify JSON structure
        let json: serde_json::Value = serde_json::from_str(&export).unwrap();

        assert_eq!(json["version"], "1.0");
        assert!(json["exported_at"].is_string());
        assert_eq!(json["vehicles"].as_array().unwrap().len(), 1);
        assert_eq!(json["sessions"].as_array().unwrap().len(), 1);
        assert_eq!(json["settings"].as_array().unwrap().len(), 3);

        // Verify vehicle data in export
        let exported_vehicle = &json["vehicles"][0];
        assert_eq!(exported_vehicle["make"], "BMW");
        assert_eq!(exported_vehicle["model"], "520d E60");
        assert_eq!(exported_vehicle["engine_code"], "M47TU2D20");

        // Verify session with DTCs
        let exported_session = &json["sessions"][0];
        assert_eq!(exported_session["session"]["ecu_name"], "DDE (Digital Diesel Electronics)");
        assert_eq!(exported_session["dtcs"].as_array().unwrap().len(), 4);
    }

    #[test]
    fn test_recent_sessions_limit() {
        let db = Database::in_memory().unwrap();

        let vehicle_id = db.create_vehicle(&bmw_e60_520d()).unwrap();

        // Create 10 sessions
        for i in 0..10 {
            let session = NewSession {
                vehicle_id,
                ecu_id: format!("0x{:02X}", i),
                ecu_name: format!("ECU {}", i),
                protocol: "K-Line".to_string(),
                mileage_km: Some(245000 + i as i32 * 100),
                notes: None,
            };
            db.create_session(&session).unwrap();
        }

        // Get only 5 recent
        let recent = db.get_recent_sessions(5).unwrap();
        assert_eq!(recent.len(), 5);
    }

    #[test]
    fn test_dtc_status_flags() {
        let db = Database::in_memory().unwrap();

        let vehicle_id = db.create_vehicle(&bmw_e60_520d()).unwrap();
        let session_id = db.create_session(&dde_session(vehicle_id)).unwrap();
        db.add_dtcs(&dpf_dtcs(session_id)).unwrap();

        let dtcs = db.get_dtcs_for_session(session_id).unwrap();

        // Check confirmed vs pending
        let confirmed: Vec<_> = dtcs.iter().filter(|d| d.is_confirmed).collect();
        let pending: Vec<_> = dtcs.iter().filter(|d| d.is_pending).collect();

        assert!(!confirmed.is_empty());
        assert!(!pending.is_empty());

        // 2AAF should be confirmed
        let dpf_pressure = dtcs.iter().find(|d| d.code == "2AAF").unwrap();
        assert!(dpf_pressure.is_confirmed);
        assert!(!dpf_pressure.is_pending);

        // 2AB0 should be pending
        let dpf_soot = dtcs.iter().find(|d| d.code == "2AB0").unwrap();
        assert!(!dpf_soot.is_confirmed);
        assert!(dpf_soot.is_pending);
    }

    #[test]
    fn test_settings_persistence() {
        let db = Database::in_memory().unwrap();

        // Store diagnostic settings
        db.set_setting("default_protocol", "K-Line").unwrap();
        db.set_setting("baud_rate", "10400").unwrap();
        db.set_setting("timeout_ms", "1000").unwrap();
        db.set_setting("auto_clear_dtcs", "false").unwrap();

        // Retrieve and verify
        assert_eq!(
            db.get_setting("default_protocol").unwrap(),
            Some("K-Line".to_string())
        );
        assert_eq!(
            db.get_setting("baud_rate").unwrap(),
            Some("10400".to_string())
        );
        assert_eq!(
            db.get_setting("timeout_ms").unwrap(),
            Some("1000".to_string())
        );
        assert_eq!(
            db.get_setting("auto_clear_dtcs").unwrap(),
            Some("false".to_string())
        );

        // Update setting
        db.set_setting("baud_rate", "115200").unwrap();
        assert_eq!(
            db.get_setting("baud_rate").unwrap(),
            Some("115200".to_string())
        );

        // Get all settings
        let all = db.get_all_settings().unwrap();
        assert_eq!(all.len(), 4);
    }
}
