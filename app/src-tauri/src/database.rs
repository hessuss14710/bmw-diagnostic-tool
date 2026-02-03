//! Database module for persistent storage
//!
//! Provides SQLite-based storage for vehicles, diagnostic sessions, DTCs, and settings.

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, Result as SqlResult};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;

/// Database connection wrapper
pub struct Database {
    conn: Mutex<Connection>,
}

// ============================================================================
// DATA MODELS
// ============================================================================

/// Vehicle profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vehicle {
    pub id: i64,
    pub vin: Option<String>,
    pub make: String,
    pub model: String,
    pub year: i32,
    pub engine_code: Option<String>,
    pub mileage_km: Option<i32>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Vehicle for creation (without id and timestamps)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewVehicle {
    pub vin: Option<String>,
    pub make: String,
    pub model: String,
    pub year: i32,
    pub engine_code: Option<String>,
    pub mileage_km: Option<i32>,
    pub notes: Option<String>,
}

/// Diagnostic session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticSession {
    pub id: i64,
    pub vehicle_id: i64,
    pub ecu_id: String,
    pub ecu_name: String,
    pub protocol: String,
    pub mileage_km: Option<i32>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// New diagnostic session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewSession {
    pub vehicle_id: i64,
    pub ecu_id: String,
    pub ecu_name: String,
    pub protocol: String,
    pub mileage_km: Option<i32>,
    pub notes: Option<String>,
}

/// Stored DTC (Diagnostic Trouble Code)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredDtc {
    pub id: i64,
    pub session_id: i64,
    pub code: String,
    pub status: String,
    pub description: Option<String>,
    pub is_pending: bool,
    pub is_confirmed: bool,
    pub created_at: DateTime<Utc>,
}

/// New DTC for storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewDtc {
    pub session_id: i64,
    pub code: String,
    pub status: String,
    pub description: Option<String>,
    pub is_pending: bool,
    pub is_confirmed: bool,
}

/// Live data snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveDataSnapshot {
    pub id: i64,
    pub session_id: i64,
    pub parameter_name: String,
    pub value: f64,
    pub unit: String,
    pub timestamp: DateTime<Utc>,
}

/// Application settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Setting {
    pub key: String,
    pub value: String,
}

// ============================================================================
// DATABASE IMPLEMENTATION
// ============================================================================

impl Database {
    /// Create a new database connection
    pub fn new(path: PathBuf) -> SqlResult<Self> {
        let conn = Connection::open(path)?;
        let db = Self {
            conn: Mutex::new(conn),
        };
        db.initialize()?;
        Ok(db)
    }

    /// Create an in-memory database (for testing)
    #[allow(dead_code)]
    pub fn in_memory() -> SqlResult<Self> {
        let conn = Connection::open_in_memory()?;
        let db = Self {
            conn: Mutex::new(conn),
        };
        db.initialize()?;
        Ok(db)
    }

    /// Initialize database schema
    fn initialize(&self) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute_batch(
            r#"
            -- Vehicles table
            CREATE TABLE IF NOT EXISTS vehicles (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                vin TEXT UNIQUE,
                make TEXT NOT NULL,
                model TEXT NOT NULL,
                year INTEGER NOT NULL,
                engine_code TEXT,
                mileage_km INTEGER,
                notes TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            -- Diagnostic sessions table
            CREATE TABLE IF NOT EXISTS diagnostic_sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                vehicle_id INTEGER NOT NULL,
                ecu_id TEXT NOT NULL,
                ecu_name TEXT NOT NULL,
                protocol TEXT NOT NULL,
                mileage_km INTEGER,
                notes TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (vehicle_id) REFERENCES vehicles(id) ON DELETE CASCADE
            );

            -- DTCs table
            CREATE TABLE IF NOT EXISTS dtcs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id INTEGER NOT NULL,
                code TEXT NOT NULL,
                status TEXT NOT NULL,
                description TEXT,
                is_pending INTEGER NOT NULL DEFAULT 0,
                is_confirmed INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (session_id) REFERENCES diagnostic_sessions(id) ON DELETE CASCADE
            );

            -- Live data snapshots table
            CREATE TABLE IF NOT EXISTS live_data_snapshots (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id INTEGER NOT NULL,
                parameter_name TEXT NOT NULL,
                value REAL NOT NULL,
                unit TEXT NOT NULL,
                timestamp TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (session_id) REFERENCES diagnostic_sessions(id) ON DELETE CASCADE
            );

            -- Settings table
            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            -- Indexes for performance
            CREATE INDEX IF NOT EXISTS idx_sessions_vehicle ON diagnostic_sessions(vehicle_id);
            CREATE INDEX IF NOT EXISTS idx_dtcs_session ON dtcs(session_id);
            CREATE INDEX IF NOT EXISTS idx_live_data_session ON live_data_snapshots(session_id);
            CREATE INDEX IF NOT EXISTS idx_vehicles_vin ON vehicles(vin);
            "#,
        )?;

        Ok(())
    }

    // ========================================================================
    // VEHICLE OPERATIONS
    // ========================================================================

    /// Create a new vehicle
    pub fn create_vehicle(&self, vehicle: &NewVehicle) -> SqlResult<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO vehicles (vin, make, model, year, engine_code, mileage_km, notes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                vehicle.vin,
                vehicle.make,
                vehicle.model,
                vehicle.year,
                vehicle.engine_code,
                vehicle.mileage_km,
                vehicle.notes,
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    /// Get all vehicles
    pub fn get_all_vehicles(&self) -> SqlResult<Vec<Vehicle>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, vin, make, model, year, engine_code, mileage_km, notes, created_at, updated_at
             FROM vehicles ORDER BY updated_at DESC",
        )?;

        let vehicles = stmt
            .query_map([], |row| {
                Ok(Vehicle {
                    id: row.get(0)?,
                    vin: row.get(1)?,
                    make: row.get(2)?,
                    model: row.get(3)?,
                    year: row.get(4)?,
                    engine_code: row.get(5)?,
                    mileage_km: row.get(6)?,
                    notes: row.get(7)?,
                    created_at: parse_datetime(row.get::<_, String>(8)?),
                    updated_at: parse_datetime(row.get::<_, String>(9)?),
                })
            })?
            .collect::<SqlResult<Vec<_>>>()?;

        Ok(vehicles)
    }

    /// Get vehicle by ID
    pub fn get_vehicle(&self, id: i64) -> SqlResult<Option<Vehicle>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, vin, make, model, year, engine_code, mileage_km, notes, created_at, updated_at
             FROM vehicles WHERE id = ?1",
        )?;

        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Vehicle {
                id: row.get(0)?,
                vin: row.get(1)?,
                make: row.get(2)?,
                model: row.get(3)?,
                year: row.get(4)?,
                engine_code: row.get(5)?,
                mileage_km: row.get(6)?,
                notes: row.get(7)?,
                created_at: parse_datetime(row.get::<_, String>(8)?),
                updated_at: parse_datetime(row.get::<_, String>(9)?),
            }))
        } else {
            Ok(None)
        }
    }

    /// Get vehicle by VIN
    pub fn get_vehicle_by_vin(&self, vin: &str) -> SqlResult<Option<Vehicle>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, vin, make, model, year, engine_code, mileage_km, notes, created_at, updated_at
             FROM vehicles WHERE vin = ?1",
        )?;

        let mut rows = stmt.query(params![vin])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Vehicle {
                id: row.get(0)?,
                vin: row.get(1)?,
                make: row.get(2)?,
                model: row.get(3)?,
                year: row.get(4)?,
                engine_code: row.get(5)?,
                mileage_km: row.get(6)?,
                notes: row.get(7)?,
                created_at: parse_datetime(row.get::<_, String>(8)?),
                updated_at: parse_datetime(row.get::<_, String>(9)?),
            }))
        } else {
            Ok(None)
        }
    }

    /// Update a vehicle
    pub fn update_vehicle(&self, id: i64, vehicle: &NewVehicle) -> SqlResult<bool> {
        let conn = self.conn.lock().unwrap();
        let rows = conn.execute(
            "UPDATE vehicles SET vin = ?1, make = ?2, model = ?3, year = ?4,
             engine_code = ?5, mileage_km = ?6, notes = ?7, updated_at = datetime('now')
             WHERE id = ?8",
            params![
                vehicle.vin,
                vehicle.make,
                vehicle.model,
                vehicle.year,
                vehicle.engine_code,
                vehicle.mileage_km,
                vehicle.notes,
                id,
            ],
        )?;
        Ok(rows > 0)
    }

    /// Delete a vehicle
    pub fn delete_vehicle(&self, id: i64) -> SqlResult<bool> {
        let conn = self.conn.lock().unwrap();
        let rows = conn.execute("DELETE FROM vehicles WHERE id = ?1", params![id])?;
        Ok(rows > 0)
    }

    // ========================================================================
    // SESSION OPERATIONS
    // ========================================================================

    /// Create a new diagnostic session
    pub fn create_session(&self, session: &NewSession) -> SqlResult<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO diagnostic_sessions (vehicle_id, ecu_id, ecu_name, protocol, mileage_km, notes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                session.vehicle_id,
                session.ecu_id,
                session.ecu_name,
                session.protocol,
                session.mileage_km,
                session.notes,
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    /// Get sessions for a vehicle
    pub fn get_sessions_for_vehicle(&self, vehicle_id: i64) -> SqlResult<Vec<DiagnosticSession>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, vehicle_id, ecu_id, ecu_name, protocol, mileage_km, notes, created_at
             FROM diagnostic_sessions WHERE vehicle_id = ?1 ORDER BY created_at DESC",
        )?;

        let sessions = stmt
            .query_map(params![vehicle_id], |row| {
                Ok(DiagnosticSession {
                    id: row.get(0)?,
                    vehicle_id: row.get(1)?,
                    ecu_id: row.get(2)?,
                    ecu_name: row.get(3)?,
                    protocol: row.get(4)?,
                    mileage_km: row.get(5)?,
                    notes: row.get(6)?,
                    created_at: parse_datetime(row.get::<_, String>(7)?),
                })
            })?
            .collect::<SqlResult<Vec<_>>>()?;

        Ok(sessions)
    }

    /// Get recent sessions (all vehicles)
    pub fn get_recent_sessions(&self, limit: i32) -> SqlResult<Vec<DiagnosticSession>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, vehicle_id, ecu_id, ecu_name, protocol, mileage_km, notes, created_at
             FROM diagnostic_sessions ORDER BY created_at DESC LIMIT ?1",
        )?;

        let sessions = stmt
            .query_map(params![limit], |row| {
                Ok(DiagnosticSession {
                    id: row.get(0)?,
                    vehicle_id: row.get(1)?,
                    ecu_id: row.get(2)?,
                    ecu_name: row.get(3)?,
                    protocol: row.get(4)?,
                    mileage_km: row.get(5)?,
                    notes: row.get(6)?,
                    created_at: parse_datetime(row.get::<_, String>(7)?),
                })
            })?
            .collect::<SqlResult<Vec<_>>>()?;

        Ok(sessions)
    }

    /// Delete a session
    pub fn delete_session(&self, id: i64) -> SqlResult<bool> {
        let conn = self.conn.lock().unwrap();
        let rows = conn.execute("DELETE FROM diagnostic_sessions WHERE id = ?1", params![id])?;
        Ok(rows > 0)
    }

    // ========================================================================
    // DTC OPERATIONS
    // ========================================================================

    /// Add DTCs to a session
    pub fn add_dtcs(&self, dtcs: &[NewDtc]) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "INSERT INTO dtcs (session_id, code, status, description, is_pending, is_confirmed)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        )?;

        for dtc in dtcs {
            stmt.execute(params![
                dtc.session_id,
                dtc.code,
                dtc.status,
                dtc.description,
                dtc.is_pending,
                dtc.is_confirmed,
            ])?;
        }

        Ok(())
    }

    /// Get DTCs for a session
    pub fn get_dtcs_for_session(&self, session_id: i64) -> SqlResult<Vec<StoredDtc>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, session_id, code, status, description, is_pending, is_confirmed, created_at
             FROM dtcs WHERE session_id = ?1 ORDER BY code",
        )?;

        let dtcs = stmt
            .query_map(params![session_id], |row| {
                Ok(StoredDtc {
                    id: row.get(0)?,
                    session_id: row.get(1)?,
                    code: row.get(2)?,
                    status: row.get(3)?,
                    description: row.get(4)?,
                    is_pending: row.get(5)?,
                    is_confirmed: row.get(6)?,
                    created_at: parse_datetime(row.get::<_, String>(7)?),
                })
            })?
            .collect::<SqlResult<Vec<_>>>()?;

        Ok(dtcs)
    }

    /// Get DTC history for a vehicle (all sessions)
    pub fn get_dtc_history_for_vehicle(&self, vehicle_id: i64) -> SqlResult<Vec<StoredDtc>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT d.id, d.session_id, d.code, d.status, d.description, d.is_pending, d.is_confirmed, d.created_at
             FROM dtcs d
             JOIN diagnostic_sessions s ON d.session_id = s.id
             WHERE s.vehicle_id = ?1
             ORDER BY d.created_at DESC",
        )?;

        let dtcs = stmt
            .query_map(params![vehicle_id], |row| {
                Ok(StoredDtc {
                    id: row.get(0)?,
                    session_id: row.get(1)?,
                    code: row.get(2)?,
                    status: row.get(3)?,
                    description: row.get(4)?,
                    is_pending: row.get(5)?,
                    is_confirmed: row.get(6)?,
                    created_at: parse_datetime(row.get::<_, String>(7)?),
                })
            })?
            .collect::<SqlResult<Vec<_>>>()?;

        Ok(dtcs)
    }

    // ========================================================================
    // SETTINGS OPERATIONS
    // ========================================================================

    /// Get a setting value
    pub fn get_setting(&self, key: &str) -> SqlResult<Option<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?1")?;
        let mut rows = stmt.query(params![key])?;

        if let Some(row) = rows.next()? {
            Ok(Some(row.get(0)?))
        } else {
            Ok(None)
        }
    }

    /// Set a setting value
    pub fn set_setting(&self, key: &str, value: &str) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            params![key, value],
        )?;
        Ok(())
    }

    /// Get all settings
    pub fn get_all_settings(&self) -> SqlResult<Vec<Setting>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT key, value FROM settings")?;

        let settings = stmt
            .query_map([], |row| {
                Ok(Setting {
                    key: row.get(0)?,
                    value: row.get(1)?,
                })
            })?
            .collect::<SqlResult<Vec<_>>>()?;

        Ok(settings)
    }

    // ========================================================================
    // BACKUP/EXPORT
    // ========================================================================

    /// Export all data as JSON
    pub fn export_all(&self) -> SqlResult<String> {
        let vehicles = self.get_all_vehicles()?;
        let settings = self.get_all_settings()?;

        // Get all sessions with their DTCs
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, vehicle_id, ecu_id, ecu_name, protocol, mileage_km, notes, created_at
             FROM diagnostic_sessions ORDER BY created_at DESC",
        )?;

        let sessions: Vec<DiagnosticSession> = stmt
            .query_map([], |row| {
                Ok(DiagnosticSession {
                    id: row.get(0)?,
                    vehicle_id: row.get(1)?,
                    ecu_id: row.get(2)?,
                    ecu_name: row.get(3)?,
                    protocol: row.get(4)?,
                    mileage_km: row.get(5)?,
                    notes: row.get(6)?,
                    created_at: parse_datetime(row.get::<_, String>(7)?),
                })
            })?
            .collect::<SqlResult<Vec<_>>>()?;
        drop(stmt);
        drop(conn);

        // Get DTCs for each session
        let mut sessions_with_dtcs: Vec<serde_json::Value> = Vec::new();
        for session in sessions {
            let dtcs = self.get_dtcs_for_session(session.id)?;
            sessions_with_dtcs.push(serde_json::json!({
                "session": session,
                "dtcs": dtcs,
            }));
        }

        let export = serde_json::json!({
            "version": "1.0",
            "exported_at": Utc::now().to_rfc3339(),
            "vehicles": vehicles,
            "sessions": sessions_with_dtcs,
            "settings": settings,
        });

        Ok(serde_json::to_string_pretty(&export).unwrap_or_default())
    }

    /// Get database statistics
    pub fn get_stats(&self) -> SqlResult<DatabaseStats> {
        let conn = self.conn.lock().unwrap();

        let vehicle_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM vehicles",
            [],
            |row| row.get(0),
        )?;

        let session_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM diagnostic_sessions",
            [],
            |row| row.get(0),
        )?;

        let dtc_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM dtcs",
            [],
            |row| row.get(0),
        )?;

        Ok(DatabaseStats {
            vehicle_count,
            session_count,
            dtc_count,
        })
    }
}

/// Database statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseStats {
    pub vehicle_count: i64,
    pub session_count: i64,
    pub dtc_count: i64,
}

// Helper function to parse datetime strings
fn parse_datetime(s: String) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(&s)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to create a test database
    fn test_db() -> Database {
        Database::in_memory().expect("Failed to create in-memory database")
    }

    // Helper to create a test vehicle
    fn create_test_vehicle(db: &Database) -> i64 {
        let vehicle = NewVehicle {
            vin: Some("WBAPH5C55BA123456".to_string()),
            make: "BMW".to_string(),
            model: "520d E60".to_string(),
            year: 2008,
            engine_code: Some("M47TU2D20".to_string()),
            mileage_km: Some(185000),
            notes: Some("Test vehicle".to_string()),
        };
        db.create_vehicle(&vehicle).expect("Failed to create vehicle")
    }

    // ========================================================================
    // VEHICLE TESTS
    // ========================================================================

    #[test]
    fn test_create_vehicle() {
        let db = test_db();
        let id = create_test_vehicle(&db);
        assert!(id > 0);
    }

    #[test]
    fn test_get_vehicle() {
        let db = test_db();
        let id = create_test_vehicle(&db);

        let loaded = db.get_vehicle(id).unwrap().unwrap();
        assert_eq!(loaded.id, id);
        assert_eq!(loaded.make, "BMW");
        assert_eq!(loaded.model, "520d E60");
        assert_eq!(loaded.year, 2008);
        assert_eq!(loaded.engine_code, Some("M47TU2D20".to_string()));
        assert_eq!(loaded.mileage_km, Some(185000));
    }

    #[test]
    fn test_get_vehicle_not_found() {
        let db = test_db();
        let result = db.get_vehicle(999).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_get_vehicle_by_vin() {
        let db = test_db();
        create_test_vehicle(&db);

        let loaded = db.get_vehicle_by_vin("WBAPH5C55BA123456").unwrap().unwrap();
        assert_eq!(loaded.vin, Some("WBAPH5C55BA123456".to_string()));
    }

    #[test]
    fn test_get_vehicle_by_vin_not_found() {
        let db = test_db();
        let result = db.get_vehicle_by_vin("NONEXISTENT").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_get_all_vehicles() {
        let db = test_db();

        // Create multiple vehicles
        for i in 0..3 {
            let vehicle = NewVehicle {
                vin: None,
                make: "BMW".to_string(),
                model: format!("Model {}", i),
                year: 2000 + i,
                engine_code: None,
                mileage_km: None,
                notes: None,
            };
            db.create_vehicle(&vehicle).unwrap();
        }

        let vehicles = db.get_all_vehicles().unwrap();
        assert_eq!(vehicles.len(), 3);
    }

    #[test]
    fn test_update_vehicle() {
        let db = test_db();
        let id = create_test_vehicle(&db);

        let updated = NewVehicle {
            vin: Some("WBAPH5C55BA123456".to_string()),
            make: "BMW".to_string(),
            model: "520d E60 LCI".to_string(),
            year: 2009,
            engine_code: Some("N47D20".to_string()),
            mileage_km: Some(200000),
            notes: Some("Updated".to_string()),
        };

        let success = db.update_vehicle(id, &updated).unwrap();
        assert!(success);

        let loaded = db.get_vehicle(id).unwrap().unwrap();
        assert_eq!(loaded.model, "520d E60 LCI");
        assert_eq!(loaded.year, 2009);
        assert_eq!(loaded.mileage_km, Some(200000));
    }

    #[test]
    fn test_update_nonexistent_vehicle() {
        let db = test_db();

        let vehicle = NewVehicle {
            vin: None,
            make: "BMW".to_string(),
            model: "Test".to_string(),
            year: 2020,
            engine_code: None,
            mileage_km: None,
            notes: None,
        };

        let success = db.update_vehicle(999, &vehicle).unwrap();
        assert!(!success);
    }

    #[test]
    fn test_delete_vehicle() {
        let db = test_db();
        let id = create_test_vehicle(&db);

        let success = db.delete_vehicle(id).unwrap();
        assert!(success);

        let result = db.get_vehicle(id).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_delete_nonexistent_vehicle() {
        let db = test_db();
        let success = db.delete_vehicle(999).unwrap();
        assert!(!success);
    }

    // ========================================================================
    // SESSION TESTS
    // ========================================================================

    #[test]
    fn test_create_session() {
        let db = test_db();
        let vehicle_id = create_test_vehicle(&db);

        let session = NewSession {
            vehicle_id,
            ecu_id: "0x12".to_string(),
            ecu_name: "DME/DDE".to_string(),
            protocol: "K-Line".to_string(),
            mileage_km: Some(185000),
            notes: Some("Initial diagnostic".to_string()),
        };

        let id = db.create_session(&session).unwrap();
        assert!(id > 0);
    }

    #[test]
    fn test_get_sessions_for_vehicle() {
        let db = test_db();
        let vehicle_id = create_test_vehicle(&db);

        // Create multiple sessions
        for i in 0..3 {
            let session = NewSession {
                vehicle_id,
                ecu_id: format!("0x{:02X}", i),
                ecu_name: format!("ECU {}", i),
                protocol: "K-Line".to_string(),
                mileage_km: None,
                notes: None,
            };
            db.create_session(&session).unwrap();
        }

        let sessions = db.get_sessions_for_vehicle(vehicle_id).unwrap();
        assert_eq!(sessions.len(), 3);
    }

    #[test]
    fn test_get_recent_sessions() {
        let db = test_db();
        let vehicle_id = create_test_vehicle(&db);

        // Create 5 sessions
        for i in 0..5 {
            let session = NewSession {
                vehicle_id,
                ecu_id: format!("0x{:02X}", i),
                ecu_name: format!("ECU {}", i),
                protocol: "K-Line".to_string(),
                mileage_km: None,
                notes: None,
            };
            db.create_session(&session).unwrap();
        }

        // Get only 3 recent
        let sessions = db.get_recent_sessions(3).unwrap();
        assert_eq!(sessions.len(), 3);
    }

    #[test]
    fn test_delete_session() {
        let db = test_db();
        let vehicle_id = create_test_vehicle(&db);

        let session = NewSession {
            vehicle_id,
            ecu_id: "0x12".to_string(),
            ecu_name: "DME".to_string(),
            protocol: "K-Line".to_string(),
            mileage_km: None,
            notes: None,
        };
        let session_id = db.create_session(&session).unwrap();

        let success = db.delete_session(session_id).unwrap();
        assert!(success);

        let sessions = db.get_sessions_for_vehicle(vehicle_id).unwrap();
        assert!(sessions.is_empty());
    }

    // ========================================================================
    // DTC TESTS
    // ========================================================================

    #[test]
    fn test_add_and_get_dtcs() {
        let db = test_db();
        let vehicle_id = create_test_vehicle(&db);

        let session = NewSession {
            vehicle_id,
            ecu_id: "DDE".to_string(),
            ecu_name: "Digital Diesel Electronics".to_string(),
            protocol: "KWP2000".to_string(),
            mileage_km: Some(150000),
            notes: None,
        };
        let session_id = db.create_session(&session).unwrap();

        let dtcs = vec![
            NewDtc {
                session_id,
                code: "2AAF".to_string(),
                status: "0x24".to_string(),
                description: Some("DPF pressure sensor - circuit open".to_string()),
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
        ];
        db.add_dtcs(&dtcs).unwrap();

        let loaded_dtcs = db.get_dtcs_for_session(session_id).unwrap();
        assert_eq!(loaded_dtcs.len(), 2);
        assert_eq!(loaded_dtcs[0].code, "2AAF");
        assert_eq!(loaded_dtcs[1].code, "2AB0");
    }

    #[test]
    fn test_get_dtc_history_for_vehicle() {
        let db = test_db();
        let vehicle_id = create_test_vehicle(&db);

        // Create two sessions with DTCs
        for i in 0..2 {
            let session = NewSession {
                vehicle_id,
                ecu_id: "DDE".to_string(),
                ecu_name: "DME".to_string(),
                protocol: "K-Line".to_string(),
                mileage_km: None,
                notes: None,
            };
            let session_id = db.create_session(&session).unwrap();

            let dtc = NewDtc {
                session_id,
                code: format!("P040{}", i),
                status: "Confirmed".to_string(),
                description: None,
                is_pending: false,
                is_confirmed: true,
            };
            db.add_dtcs(&[dtc]).unwrap();
        }

        let history = db.get_dtc_history_for_vehicle(vehicle_id).unwrap();
        assert_eq!(history.len(), 2);
    }

    // ========================================================================
    // SETTINGS TESTS
    // ========================================================================

    #[test]
    fn test_set_and_get_setting() {
        let db = test_db();

        db.set_setting("theme", "dark").unwrap();
        assert_eq!(db.get_setting("theme").unwrap(), Some("dark".to_string()));
    }

    #[test]
    fn test_get_nonexistent_setting() {
        let db = test_db();
        assert_eq!(db.get_setting("nonexistent").unwrap(), None);
    }

    #[test]
    fn test_update_setting() {
        let db = test_db();

        db.set_setting("theme", "light").unwrap();
        db.set_setting("theme", "dark").unwrap();

        assert_eq!(db.get_setting("theme").unwrap(), Some("dark".to_string()));
    }

    #[test]
    fn test_get_all_settings() {
        let db = test_db();

        db.set_setting("theme", "dark").unwrap();
        db.set_setting("language", "en").unwrap();
        db.set_setting("units", "metric").unwrap();

        let settings = db.get_all_settings().unwrap();
        assert_eq!(settings.len(), 3);
    }

    // ========================================================================
    // STATS & EXPORT TESTS
    // ========================================================================

    #[test]
    fn test_get_stats() {
        let db = test_db();

        // Create some data
        let vehicle_id = create_test_vehicle(&db);
        let session = NewSession {
            vehicle_id,
            ecu_id: "DDE".to_string(),
            ecu_name: "DME".to_string(),
            protocol: "K-Line".to_string(),
            mileage_km: None,
            notes: None,
        };
        let session_id = db.create_session(&session).unwrap();

        let dtc = NewDtc {
            session_id,
            code: "P0401".to_string(),
            status: "Confirmed".to_string(),
            description: None,
            is_pending: false,
            is_confirmed: true,
        };
        db.add_dtcs(&[dtc]).unwrap();

        let stats = db.get_stats().unwrap();
        assert_eq!(stats.vehicle_count, 1);
        assert_eq!(stats.session_count, 1);
        assert_eq!(stats.dtc_count, 1);
    }

    #[test]
    fn test_export_all() {
        let db = test_db();

        // Create some data
        let vehicle_id = create_test_vehicle(&db);
        let session = NewSession {
            vehicle_id,
            ecu_id: "DDE".to_string(),
            ecu_name: "DME".to_string(),
            protocol: "K-Line".to_string(),
            mileage_km: None,
            notes: None,
        };
        db.create_session(&session).unwrap();
        db.set_setting("theme", "dark").unwrap();

        let export = db.export_all().unwrap();

        // Verify it's valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&export).unwrap();
        assert_eq!(parsed["version"], "1.0");
        assert!(parsed["vehicles"].as_array().unwrap().len() > 0);
        assert!(parsed["sessions"].as_array().unwrap().len() > 0);
        assert!(parsed["settings"].as_array().unwrap().len() > 0);
    }

    // ========================================================================
    // CASCADE DELETE TESTS
    // ========================================================================

    #[test]
    fn test_delete_vehicle_cascades_to_sessions() {
        let db = test_db();
        let vehicle_id = create_test_vehicle(&db);

        let session = NewSession {
            vehicle_id,
            ecu_id: "DDE".to_string(),
            ecu_name: "DME".to_string(),
            protocol: "K-Line".to_string(),
            mileage_km: None,
            notes: None,
        };
        db.create_session(&session).unwrap();

        // Delete vehicle
        db.delete_vehicle(vehicle_id).unwrap();

        // Sessions should be deleted too
        let sessions = db.get_sessions_for_vehicle(vehicle_id).unwrap();
        assert!(sessions.is_empty());
    }

    #[test]
    fn test_delete_session_cascades_to_dtcs() {
        let db = test_db();
        let vehicle_id = create_test_vehicle(&db);

        let session = NewSession {
            vehicle_id,
            ecu_id: "DDE".to_string(),
            ecu_name: "DME".to_string(),
            protocol: "K-Line".to_string(),
            mileage_km: None,
            notes: None,
        };
        let session_id = db.create_session(&session).unwrap();

        let dtc = NewDtc {
            session_id,
            code: "P0401".to_string(),
            status: "Confirmed".to_string(),
            description: None,
            is_pending: false,
            is_confirmed: true,
        };
        db.add_dtcs(&[dtc]).unwrap();

        // Delete session
        db.delete_session(session_id).unwrap();

        // DTCs should be deleted too
        let dtcs = db.get_dtcs_for_session(session_id).unwrap();
        assert!(dtcs.is_empty());
    }
}
