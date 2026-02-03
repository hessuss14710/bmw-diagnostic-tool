/**
 * Database hook for vehicle profiles, sessions, and settings persistence
 */

import { useState, useCallback } from "react"
import { invoke } from "@tauri-apps/api/core"

// ============================================================================
// TYPES
// ============================================================================

export interface Vehicle {
  id: number
  vin: string | null
  make: string
  model: string
  year: number
  engine_code: string | null
  mileage_km: number | null
  notes: string | null
  created_at: string
  updated_at: string
}

export interface NewVehicle {
  vin?: string | null
  make: string
  model: string
  year: number
  engine_code?: string | null
  mileage_km?: number | null
  notes?: string | null
}

export interface DiagnosticSession {
  id: number
  vehicle_id: number
  ecu_id: string
  ecu_name: string
  protocol: string
  mileage_km: number | null
  notes: string | null
  created_at: string
}

export interface NewSession {
  vehicle_id: number
  ecu_id: string
  ecu_name: string
  protocol: string
  mileage_km?: number | null
  notes?: string | null
}

export interface StoredDtc {
  id: number
  session_id: number
  code: string
  status: string
  description: string | null
  is_pending: boolean
  is_confirmed: boolean
  created_at: string
}

export interface NewDtc {
  session_id: number
  code: string
  status: string
  description?: string | null
  is_pending: boolean
  is_confirmed: boolean
}

export interface Setting {
  key: string
  value: string
}

export interface DatabaseStats {
  vehicle_count: number
  session_count: number
  dtc_count: number
}

// ============================================================================
// HOOK
// ============================================================================

export function useDatabase() {
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  // Helper to wrap async calls
  const withLoading = useCallback(async <T>(fn: () => Promise<T>): Promise<T> => {
    setIsLoading(true)
    setError(null)
    try {
      const result = await fn()
      return result
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e)
      setError(message)
      throw e
    } finally {
      setIsLoading(false)
    }
  }, [])

  // ========================================================================
  // VEHICLE OPERATIONS
  // ========================================================================

  const getVehicles = useCallback(async (): Promise<Vehicle[]> => {
    return withLoading(() => invoke<Vehicle[]>("db_get_vehicles"))
  }, [withLoading])

  const getVehicle = useCallback(async (id: number): Promise<Vehicle | null> => {
    return withLoading(() => invoke<Vehicle | null>("db_get_vehicle", { id }))
  }, [withLoading])

  const getVehicleByVin = useCallback(async (vin: string): Promise<Vehicle | null> => {
    return withLoading(() => invoke<Vehicle | null>("db_get_vehicle_by_vin", { vin }))
  }, [withLoading])

  const createVehicle = useCallback(async (vehicle: NewVehicle): Promise<number> => {
    return withLoading(() => invoke<number>("db_create_vehicle", { vehicle }))
  }, [withLoading])

  const updateVehicle = useCallback(async (id: number, vehicle: NewVehicle): Promise<boolean> => {
    return withLoading(() => invoke<boolean>("db_update_vehicle", { id, vehicle }))
  }, [withLoading])

  const deleteVehicle = useCallback(async (id: number): Promise<boolean> => {
    return withLoading(() => invoke<boolean>("db_delete_vehicle", { id }))
  }, [withLoading])

  // ========================================================================
  // SESSION OPERATIONS
  // ========================================================================

  const createSession = useCallback(async (session: NewSession): Promise<number> => {
    return withLoading(() => invoke<number>("db_create_session", { session }))
  }, [withLoading])

  const getSessionsForVehicle = useCallback(async (vehicleId: number): Promise<DiagnosticSession[]> => {
    return withLoading(() => invoke<DiagnosticSession[]>("db_get_sessions_for_vehicle", { vehicleId }))
  }, [withLoading])

  const getRecentSessions = useCallback(async (limit: number = 20): Promise<DiagnosticSession[]> => {
    return withLoading(() => invoke<DiagnosticSession[]>("db_get_recent_sessions", { limit }))
  }, [withLoading])

  const deleteSession = useCallback(async (id: number): Promise<boolean> => {
    return withLoading(() => invoke<boolean>("db_delete_session", { id }))
  }, [withLoading])

  // ========================================================================
  // DTC OPERATIONS
  // ========================================================================

  const addDtcs = useCallback(async (dtcs: NewDtc[]): Promise<void> => {
    return withLoading(() => invoke<void>("db_add_dtcs", { dtcs }))
  }, [withLoading])

  const getDtcsForSession = useCallback(async (sessionId: number): Promise<StoredDtc[]> => {
    return withLoading(() => invoke<StoredDtc[]>("db_get_dtcs_for_session", { sessionId }))
  }, [withLoading])

  const getDtcHistory = useCallback(async (vehicleId: number): Promise<StoredDtc[]> => {
    return withLoading(() => invoke<StoredDtc[]>("db_get_dtc_history", { vehicleId }))
  }, [withLoading])

  // ========================================================================
  // SETTINGS OPERATIONS
  // ========================================================================

  const getSetting = useCallback(async (key: string): Promise<string | null> => {
    return withLoading(() => invoke<string | null>("db_get_setting", { key }))
  }, [withLoading])

  const setSetting = useCallback(async (key: string, value: string): Promise<void> => {
    return withLoading(() => invoke<void>("db_set_setting", { key, value }))
  }, [withLoading])

  const getAllSettings = useCallback(async (): Promise<Setting[]> => {
    return withLoading(() => invoke<Setting[]>("db_get_all_settings"))
  }, [withLoading])

  // ========================================================================
  // EXPORT/STATS
  // ========================================================================

  const exportAll = useCallback(async (): Promise<string> => {
    return withLoading(() => invoke<string>("db_export_all"))
  }, [withLoading])

  const getStats = useCallback(async (): Promise<DatabaseStats> => {
    return withLoading(() => invoke<DatabaseStats>("db_get_stats"))
  }, [withLoading])

  const clearError = useCallback(() => setError(null), [])

  return {
    // State
    isLoading,
    error,
    clearError,

    // Vehicles
    getVehicles,
    getVehicle,
    getVehicleByVin,
    createVehicle,
    updateVehicle,
    deleteVehicle,

    // Sessions
    createSession,
    getSessionsForVehicle,
    getRecentSessions,
    deleteSession,

    // DTCs
    addDtcs,
    getDtcsForSession,
    getDtcHistory,

    // Settings
    getSetting,
    setSetting,
    getAllSettings,

    // Export/Stats
    exportAll,
    getStats,
  }
}
