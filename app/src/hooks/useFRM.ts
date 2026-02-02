import { invoke } from "@tauri-apps/api/core"
import { useState, useCallback } from "react"
import type { Dtc, DtcReadResult } from "./useBMW"

export interface LampStatus {
  front_left_low: boolean
  front_right_low: boolean
  front_left_high: boolean
  front_right_high: boolean
  rear_left: boolean
  rear_right: boolean
  brake_left: boolean
  brake_right: boolean
  brake_center: boolean
  turn_front_left: boolean
  turn_front_right: boolean
  turn_rear_left: boolean
  turn_rear_right: boolean
  fog_front_left: boolean
  fog_front_right: boolean
  fog_rear: boolean
  reverse_left: boolean
  reverse_right: boolean
}

export interface DpfRoutineResult {
  success: boolean
  routine_id: number
  status: string
  data: number[]
}

export function useFRM() {
  const [lampStatus, setLampStatus] = useState<LampStatus | null>(null)
  const [dtcs, setDtcs] = useState<Dtc[]>([])
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  // Read DTCs from FRM
  const readDtcs = useCallback(async () => {
    setIsLoading(true)
    setError(null)
    try {
      const result = await invoke<DtcReadResult>("bmw_frm_read_dtcs")
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

  // Read lamp status
  const readLampStatus = useCallback(async () => {
    setIsLoading(true)
    setError(null)
    try {
      const result = await invoke<LampStatus>("bmw_frm_read_lamp_status")
      setLampStatus(result)
      return result
    } catch (e) {
      const errorMsg = e instanceof Error ? e.message : String(e)
      setError(errorMsg)
      throw e
    } finally {
      setIsLoading(false)
    }
  }, [])

  // Run lamp test
  const lampTest = useCallback(async () => {
    setIsLoading(true)
    setError(null)
    try {
      const result = await invoke<DpfRoutineResult>("bmw_frm_lamp_test")
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

  // Control individual lamp
  const controlLamp = useCallback(async (lampId: number, on: boolean) => {
    setIsLoading(true)
    setError(null)
    try {
      const result = await invoke<string>("bmw_frm_control_lamp", { lampId, on })
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
    lampStatus,
    dtcs,
    isLoading,
    error,
    readDtcs,
    readLampStatus,
    lampTest,
    controlLamp,
  }
}
