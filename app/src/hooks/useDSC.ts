import { invoke } from "@tauri-apps/api/core"
import { useState, useCallback } from "react"
import type { Dtc, DtcReadResult } from "./useBMW"

export interface WheelSpeedData {
  front_left: number
  front_right: number
  rear_left: number
  rear_right: number
  timestamp: number
}

export interface DscSensorStatus {
  steering_angle: number | null
  yaw_rate: number | null
  lateral_acceleration: number | null
  longitudinal_acceleration: number | null
  brake_pressure: number | null
}

export interface DpfRoutineResult {
  success: boolean
  routine_id: number
  status: string
  data: number[]
}

export function useDSC() {
  const [wheelSpeeds, setWheelSpeeds] = useState<WheelSpeedData | null>(null)
  const [sensors, setSensors] = useState<DscSensorStatus | null>(null)
  const [dtcs, setDtcs] = useState<Dtc[]>([])
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  // Read DTCs from DSC
  const readDtcs = useCallback(async () => {
    setIsLoading(true)
    setError(null)
    try {
      const result = await invoke<DtcReadResult>("bmw_dsc_read_dtcs")
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

  // Read wheel speeds
  const readWheelSpeeds = useCallback(async () => {
    setIsLoading(true)
    setError(null)
    try {
      const result = await invoke<WheelSpeedData>("bmw_dsc_read_wheel_speeds")
      setWheelSpeeds(result)
      return result
    } catch (e) {
      const errorMsg = e instanceof Error ? e.message : String(e)
      setError(errorMsg)
      throw e
    } finally {
      setIsLoading(false)
    }
  }, [])

  // Read sensors
  const readSensors = useCallback(async () => {
    setIsLoading(true)
    setError(null)
    try {
      const result = await invoke<DscSensorStatus>("bmw_dsc_read_sensors")
      setSensors(result)
      return result
    } catch (e) {
      const errorMsg = e instanceof Error ? e.message : String(e)
      setError(errorMsg)
      throw e
    } finally {
      setIsLoading(false)
    }
  }, [])

  // Bleed brakes
  const bleedBrakes = useCallback(async (corner: "FL" | "FR" | "RL" | "RR" | "ALL") => {
    setIsLoading(true)
    setError(null)
    try {
      const result = await invoke<DpfRoutineResult>("bmw_dsc_bleed_brakes", { corner })
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
    wheelSpeeds,
    sensors,
    dtcs,
    isLoading,
    error,
    readDtcs,
    readWheelSpeeds,
    readSensors,
    bleedBrakes,
  }
}
