import { invoke } from "@tauri-apps/api/core"
import { useState, useCallback } from "react"
import type { Dtc, DtcReadResult } from "./useBMW"

export interface ServiceInfo {
  oil_service_km: number | null
  oil_service_days: number | null
  inspection_km: number | null
  inspection_days: number | null
  brake_fluid_months: number | null
}

export interface VehicleInfo {
  vin: string | null
  mileage_km: number | null
  fuel_level_percent: number | null
  coolant_temp: number | null
  outside_temp: number | null
}

export interface DpfRoutineResult {
  success: boolean
  routine_id: number
  status: string
  data: number[]
}

export function useKOMBI() {
  const [serviceInfo, setServiceInfo] = useState<ServiceInfo | null>(null)
  const [vehicleInfo, setVehicleInfo] = useState<VehicleInfo | null>(null)
  const [dtcs, setDtcs] = useState<Dtc[]>([])
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  // Read DTCs from KOMBI
  const readDtcs = useCallback(async () => {
    setIsLoading(true)
    setError(null)
    try {
      const result = await invoke<DtcReadResult>("bmw_kombi_read_dtcs")
      if (result.success) {
        setDtcs(result.dtcs)
      } else {
        setError(result.message)
      }
      return result
    } catch (e) {
      const errorMsg = e instanceof Error ? e.message : String(e)
      setError(errorMsg)
      throw e
    } finally {
      setIsLoading(false)
    }
  }, [])

  // Read service intervals
  const readServiceInfo = useCallback(async () => {
    setIsLoading(true)
    setError(null)
    try {
      const result = await invoke<ServiceInfo>("bmw_kombi_read_service")
      setServiceInfo(result)
      return result
    } catch (e) {
      const errorMsg = e instanceof Error ? e.message : String(e)
      setError(errorMsg)
      throw e
    } finally {
      setIsLoading(false)
    }
  }, [])

  // Read vehicle info
  const readVehicleInfo = useCallback(async () => {
    setIsLoading(true)
    setError(null)
    try {
      const result = await invoke<VehicleInfo>("bmw_kombi_read_info")
      setVehicleInfo(result)
      return result
    } catch (e) {
      const errorMsg = e instanceof Error ? e.message : String(e)
      setError(errorMsg)
      throw e
    } finally {
      setIsLoading(false)
    }
  }, [])

  // Reset service interval
  const resetService = useCallback(async (serviceType: "oil" | "inspection" | "brake_fluid" | "all") => {
    setIsLoading(true)
    setError(null)
    try {
      const result = await invoke<DpfRoutineResult>("bmw_kombi_reset_service", { serviceType })
      if (!result.success) {
        setError(result.status)
      }
      return result
    } catch (e) {
      const errorMsg = e instanceof Error ? e.message : String(e)
      setError(errorMsg)
      throw e
    } finally {
      setIsLoading(false)
    }
  }, [])

  // Run gauge test
  const gaugeTest = useCallback(async () => {
    setIsLoading(true)
    setError(null)
    try {
      const result = await invoke<DpfRoutineResult>("bmw_kombi_gauge_test")
      if (!result.success) {
        setError(result.status)
      }
      return result
    } catch (e) {
      const errorMsg = e instanceof Error ? e.message : String(e)
      setError(errorMsg)
      throw e
    } finally {
      setIsLoading(false)
    }
  }, [])

  return {
    serviceInfo,
    vehicleInfo,
    dtcs,
    isLoading,
    error,
    readDtcs,
    readServiceInfo,
    readVehicleInfo,
    resetService,
    gaugeTest,
  }
}
