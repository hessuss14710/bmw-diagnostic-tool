import { invoke } from "@tauri-apps/api/core"
import { useState, useCallback } from "react"

export interface DpfStatus {
  soot_loading_percent: number | null
  ash_loading_grams: number | null
  differential_pressure_mbar: number | null
  temp_before_dpf: number | null
  temp_after_dpf: number | null
  distance_since_regen_km: number | null
  regen_count: number | null
  regen_active: boolean
}

export interface DpfRoutineResult {
  success: boolean
  routine_id: number
  status: string
  data: number[]
}

export interface SessionResult {
  success: boolean
  session_type: number
  message: string
}

export interface SecurityResult {
  success: boolean
  level: number
  message: string
}

export function useDPF() {
  const [status, setStatus] = useState<DpfStatus | null>(null)
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [lastResult, setLastResult] = useState<DpfRoutineResult | null>(null)
  const [sessionActive, setSessionActive] = useState(false)
  const [securityUnlocked, setSecurityUnlocked] = useState(false)

  // Start extended diagnostic session
  const startExtendedSession = useCallback(async (targetAddress?: number) => {
    setIsLoading(true)
    setError(null)
    try {
      const result = await invoke<SessionResult>("bmw_start_session", {
        targetAddress,
        sessionType: 0x03, // Extended session
      })
      setSessionActive(result.success)
      if (!result.success) {
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

  // Perform security access
  const securityAccess = useCallback(async (targetAddress?: number, level?: number) => {
    setIsLoading(true)
    setError(null)
    try {
      const result = await invoke<SecurityResult>("bmw_security_access", {
        targetAddress,
        level: level ?? 0x01,
      })
      setSecurityUnlocked(result.success)
      if (!result.success) {
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

  // Read DPF status
  const readStatus = useCallback(async (targetAddress?: number) => {
    setIsLoading(true)
    setError(null)
    try {
      const result = await invoke<DpfStatus>("bmw_dpf_read_status", {
        targetAddress,
      })
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

  // Reset ash counter
  const resetAsh = useCallback(async (targetAddress?: number) => {
    setIsLoading(true)
    setError(null)
    try {
      const result = await invoke<DpfRoutineResult>("bmw_dpf_reset_ash", {
        targetAddress,
      })
      setLastResult(result)
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

  // Reset learned values
  const resetLearned = useCallback(async (targetAddress?: number) => {
    setIsLoading(true)
    setError(null)
    try {
      const result = await invoke<DpfRoutineResult>("bmw_dpf_reset_learned", {
        targetAddress,
      })
      setLastResult(result)
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

  // Register new DPF
  const newDpfInstalled = useCallback(async (targetAddress?: number) => {
    setIsLoading(true)
    setError(null)
    try {
      const result = await invoke<DpfRoutineResult>("bmw_dpf_new_installed", {
        targetAddress,
      })
      setLastResult(result)
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

  // Start forced regeneration
  const startRegen = useCallback(async (targetAddress?: number) => {
    setIsLoading(true)
    setError(null)
    try {
      const result = await invoke<DpfRoutineResult>("bmw_dpf_start_regen", {
        targetAddress,
      })
      setLastResult(result)
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

  // Stop forced regeneration
  const stopRegen = useCallback(async (targetAddress?: number) => {
    setIsLoading(true)
    setError(null)
    try {
      const result = await invoke<DpfRoutineResult>("bmw_dpf_stop_regen", {
        targetAddress,
      })
      setLastResult(result)
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

  // Execute generic routine
  const executeRoutine = useCallback(async (
    targetAddress: number | undefined,
    routineId: number,
    subFunction: number,
    data?: number[]
  ) => {
    setIsLoading(true)
    setError(null)
    try {
      const result = await invoke<DpfRoutineResult>("bmw_routine_control", {
        targetAddress,
        routineId,
        subFunction,
        data,
      })
      setLastResult(result)
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

  // Clear error
  const clearError = useCallback(() => {
    setError(null)
  }, [])

  // Reset all state
  const reset = useCallback(() => {
    setStatus(null)
    setError(null)
    setLastResult(null)
    setSessionActive(false)
    setSecurityUnlocked(false)
  }, [])

  return {
    // State
    status,
    isLoading,
    error,
    lastResult,
    sessionActive,
    securityUnlocked,

    // Actions
    startExtendedSession,
    securityAccess,
    readStatus,
    resetAsh,
    resetLearned,
    newDpfInstalled,
    startRegen,
    stopRegen,
    executeRoutine,
    clearError,
    reset,
  }
}
