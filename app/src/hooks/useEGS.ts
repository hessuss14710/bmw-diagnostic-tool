import { invoke } from "@tauri-apps/api/core"
import { useState, useCallback } from "react"
import type { Dtc, DtcReadResult } from "./useBMW"

export interface EgsStatus {
  oil_temp: number | null
  gear_position: string | null
  target_gear: number | null
  actual_gear: number | null
  torque_converter_lockup: boolean
  sport_mode: boolean
}

export interface DpfRoutineResult {
  success: boolean
  routine_id: number
  status: string
  data: number[]
}

export function useEGS() {
  const [status, setStatus] = useState<EgsStatus | null>(null)
  const [dtcs, setDtcs] = useState<Dtc[]>([])
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  // Read DTCs from EGS
  const readDtcs = useCallback(async () => {
    setIsLoading(true)
    setError(null)
    try {
      const result = await invoke<DtcReadResult>("bmw_egs_read_dtcs")
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

  // Read EGS status
  const readStatus = useCallback(async () => {
    setIsLoading(true)
    setError(null)
    try {
      const result = await invoke<EgsStatus>("bmw_egs_read_status")
      setStatus(result)
      return result
    } catch (e) {
      const errorMsg = e instanceof Error ? e.message : String(e)
      setError(errorMsg)
      throw e
    } finally {
      setIsLoading(false)
    }
  }, [])

  // Reset adaptations
  const resetAdaptations = useCallback(async () => {
    setIsLoading(true)
    setError(null)
    try {
      const result = await invoke<DpfRoutineResult>("bmw_egs_reset_adaptations")
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
    status,
    dtcs,
    isLoading,
    error,
    readDtcs,
    readStatus,
    resetAdaptations,
  }
}
