import { invoke } from "@tauri-apps/api/core"
import { useState, useCallback, useRef, useEffect } from "react"

export interface PidDefinition {
  id: number
  name: string
  short_name: string
  unit: string
  min: number
  max: number
  format: string
}

export interface LiveDataValue {
  pid: number
  name: string
  value: number
  unit: string
  raw: number[]
  timestamp: number
}

export interface PidHistory {
  pid: number
  values: { timestamp: number; value: number }[]
}

const MAX_HISTORY_POINTS = 100

export function useLiveData() {
  const [availablePids, setAvailablePids] = useState<PidDefinition[]>([])
  const [selectedPids, setSelectedPids] = useState<number[]>([])
  const [liveData, setLiveData] = useState<Map<number, LiveDataValue>>(new Map())
  const [history, setHistory] = useState<Map<number, PidHistory>>(new Map())
  const [isPolling, setIsPolling] = useState(false)
  const [pollInterval, setPollInterval] = useState(500) // ms
  const [error, setError] = useState<string | null>(null)

  const pollingRef = useRef<ReturnType<typeof setInterval> | null>(null)
  const targetAddressRef = useRef<number>(0x12) // Default DME address

  // Load available PIDs
  const getAvailablePids = useCallback(async () => {
    try {
      const result = await invoke<PidDefinition[]>("get_available_pids")
      setAvailablePids(result)
      return result
    } catch (e) {
      const errorMsg = e instanceof Error ? e.message : String(e)
      setError(errorMsg)
      return []
    }
  }, [])

  // Read a single PID
  const readPid = useCallback(async (pid: number, targetAddress?: number) => {
    try {
      const result = await invoke<LiveDataValue>("read_pid_kline", {
        targetAddress: targetAddress ?? targetAddressRef.current,
        pid,
      })

      // Update live data
      setLiveData((prev) => {
        const next = new Map(prev)
        next.set(pid, result)
        return next
      })

      // Update history
      setHistory((prev) => {
        const next = new Map(prev)
        const pidHistory = next.get(pid) || { pid, values: [] }
        pidHistory.values.push({
          timestamp: result.timestamp,
          value: result.value,
        })
        // Keep only last N points
        if (pidHistory.values.length > MAX_HISTORY_POINTS) {
          pidHistory.values = pidHistory.values.slice(-MAX_HISTORY_POINTS)
        }
        next.set(pid, pidHistory)
        return next
      })

      setError(null)
      return result
    } catch (e) {
      const errorMsg = e instanceof Error ? e.message : String(e)
      setError(errorMsg)
      throw e
    }
  }, [])

  // Read multiple PIDs
  const readPids = useCallback(async (pids: number[], targetAddress?: number) => {
    try {
      const results = await invoke<LiveDataValue[]>("read_pids_kline", {
        targetAddress: targetAddress ?? targetAddressRef.current,
        pids,
      })

      // Update live data and history for all results
      setLiveData((prev) => {
        const next = new Map(prev)
        for (const result of results) {
          next.set(result.pid, result)
        }
        return next
      })

      setHistory((prev) => {
        const next = new Map(prev)
        for (const result of results) {
          const pidHistory = next.get(result.pid) || { pid: result.pid, values: [] }
          pidHistory.values.push({
            timestamp: result.timestamp,
            value: result.value,
          })
          if (pidHistory.values.length > MAX_HISTORY_POINTS) {
            pidHistory.values = pidHistory.values.slice(-MAX_HISTORY_POINTS)
          }
          next.set(result.pid, pidHistory)
        }
        return next
      })

      setError(null)
      return results
    } catch (e) {
      const errorMsg = e instanceof Error ? e.message : String(e)
      setError(errorMsg)
      throw e
    }
  }, [])

  // Start polling selected PIDs
  const startPolling = useCallback((targetAddress?: number) => {
    if (selectedPids.length === 0) {
      setError("No PIDs selected for polling")
      return
    }

    if (targetAddress !== undefined) {
      targetAddressRef.current = targetAddress
    }

    setIsPolling(true)
    setError(null)

    // Initial read
    readPids(selectedPids).catch(() => {})

    // Start interval
    pollingRef.current = setInterval(() => {
      readPids(selectedPids).catch(() => {})
    }, pollInterval)
  }, [selectedPids, pollInterval, readPids])

  // Stop polling
  const stopPolling = useCallback(() => {
    if (pollingRef.current) {
      clearInterval(pollingRef.current)
      pollingRef.current = null
    }
    setIsPolling(false)
  }, [])

  // Toggle PID selection
  const togglePid = useCallback((pid: number) => {
    setSelectedPids((prev) => {
      if (prev.includes(pid)) {
        return prev.filter((p) => p !== pid)
      }
      return [...prev, pid]
    })
  }, [])

  // Select all PIDs
  const selectAllPids = useCallback(() => {
    setSelectedPids(availablePids.map((p) => p.id))
  }, [availablePids])

  // Clear PID selection
  const clearSelectedPids = useCallback(() => {
    setSelectedPids([])
  }, [])

  // Clear history
  const clearHistory = useCallback(() => {
    setHistory(new Map())
  }, [])

  // Set target ECU address
  const setTargetAddress = useCallback((address: number) => {
    targetAddressRef.current = address
  }, [])

  // Get current value for a PID
  const getValue = useCallback((pid: number): LiveDataValue | undefined => {
    return liveData.get(pid)
  }, [liveData])

  // Get history for a PID
  const getHistory = useCallback((pid: number): PidHistory | undefined => {
    return history.get(pid)
  }, [history])

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (pollingRef.current) {
        clearInterval(pollingRef.current)
      }
    }
  }, [])

  return {
    // State
    availablePids,
    selectedPids,
    liveData,
    history,
    isPolling,
    pollInterval,
    error,

    // Actions
    getAvailablePids,
    readPid,
    readPids,
    startPolling,
    stopPolling,
    togglePid,
    selectAllPids,
    clearSelectedPids,
    clearHistory,
    setTargetAddress,
    setPollInterval,

    // Getters
    getValue,
    getHistory,
  }
}
