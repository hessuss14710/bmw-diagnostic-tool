import { invoke } from "@tauri-apps/api/core"
import { useState, useCallback } from "react"

export interface EcuInfo {
  id: string
  name: string
  description: string
  kline_address: number | null
  can_tx_id: number | null
  can_rx_id: number | null
  protocol: "KLine" | "DCan" | "Both"
}

export interface DtcStatus {
  test_failed: boolean
  test_failed_this_cycle: boolean
  pending: boolean
  confirmed: boolean
  test_not_completed_since_clear: boolean
  test_failed_since_clear: boolean
  test_not_completed_this_cycle: boolean
  warning_indicator_requested: boolean
  raw: number
}

export interface Dtc {
  code: string
  status: DtcStatus
  description: string | null
  raw_bytes: number[]
}

export interface BmwInitResult {
  success: boolean
  protocol: string
  message: string
}

export interface DtcReadResult {
  success: boolean
  dtcs: Dtc[]
  count: number
  message: string
}

export function useBMW() {
  const [ecus, setEcus] = useState<EcuInfo[]>([])
  const [selectedEcu, setSelectedEcu] = useState<EcuInfo | null>(null)
  const [dtcs, setDtcs] = useState<Dtc[]>([])
  const [isInitialized, setIsInitialized] = useState(false)
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [protocol, setProtocol] = useState<string | null>(null)

  // Get list of known ECUs
  const getEcus = useCallback(async () => {
    try {
      const result = await invoke<EcuInfo[]>("bmw_get_ecus")
      setEcus(result)
      return result
    } catch (e) {
      const errorMsg = e instanceof Error ? e.message : String(e)
      setError(errorMsg)
      return []
    }
  }, [])

  // Switch to K-Line mode
  const switchToKLine = useCallback(async () => {
    setIsLoading(true)
    setError(null)
    try {
      const result = await invoke<string>("bmw_switch_kline")
      setProtocol("K-Line")
      return result
    } catch (e) {
      const errorMsg = e instanceof Error ? e.message : String(e)
      setError(errorMsg)
      throw e
    } finally {
      setIsLoading(false)
    }
  }, [])

  // Switch to D-CAN mode
  const switchToDCan = useCallback(async () => {
    setIsLoading(true)
    setError(null)
    try {
      const result = await invoke<string>("bmw_switch_dcan")
      setProtocol("D-CAN")
      return result
    } catch (e) {
      const errorMsg = e instanceof Error ? e.message : String(e)
      setError(errorMsg)
      throw e
    } finally {
      setIsLoading(false)
    }
  }, [])

  // Initialize K-Line communication
  const initKLine = useCallback(async (targetAddress?: number) => {
    setIsLoading(true)
    setError(null)
    try {
      const result = await invoke<BmwInitResult>("bmw_kline_init", {
        targetAddress,
      })
      setIsInitialized(result.success)
      setProtocol(result.protocol)
      if (!result.success) {
        setError(result.message)
      }
      return result
    } catch (e) {
      const errorMsg = e instanceof Error ? e.message : String(e)
      setError(errorMsg)
      setIsInitialized(false)
      throw e
    } finally {
      setIsLoading(false)
    }
  }, [])

  // Read DTCs from ECU
  const readDtcs = useCallback(async (targetAddress?: number) => {
    setIsLoading(true)
    setError(null)
    try {
      const result = await invoke<DtcReadResult>("bmw_read_dtcs_kline", {
        targetAddress,
      })
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

  // Clear DTCs from ECU
  const clearDtcs = useCallback(async (targetAddress?: number) => {
    setIsLoading(true)
    setError(null)
    try {
      const result = await invoke<string>("bmw_clear_dtcs_kline", {
        targetAddress,
      })
      setDtcs([])
      return result
    } catch (e) {
      const errorMsg = e instanceof Error ? e.message : String(e)
      setError(errorMsg)
      throw e
    } finally {
      setIsLoading(false)
    }
  }, [])

  // Read ECU identification (VIN)
  const readEcuId = useCallback(async (targetAddress?: number) => {
    setIsLoading(true)
    setError(null)
    try {
      const result = await invoke<string>("bmw_read_ecu_id", {
        targetAddress,
      })
      return result
    } catch (e) {
      const errorMsg = e instanceof Error ? e.message : String(e)
      setError(errorMsg)
      throw e
    } finally {
      setIsLoading(false)
    }
  }, [])

  // Send tester present to keep session alive
  const testerPresent = useCallback(async (targetAddress?: number) => {
    try {
      await invoke("bmw_tester_present", { targetAddress })
    } catch (e) {
      // Silent fail for tester present
      console.warn("TesterPresent failed:", e)
    }
  }, [])

  // Select an ECU and initialize communication
  const selectEcu = useCallback(
    async (ecu: EcuInfo) => {
      setSelectedEcu(ecu)
      setIsInitialized(false)
      setDtcs([])
      setError(null)

      // Initialize based on protocol
      if (ecu.kline_address !== null) {
        try {
          await initKLine(ecu.kline_address)
        } catch {
          // Error already set in initKLine
        }
      }
    },
    [initKLine]
  )

  // Reset state
  const reset = useCallback(() => {
    setSelectedEcu(null)
    setIsInitialized(false)
    setDtcs([])
    setError(null)
    setProtocol(null)
  }, [])

  return {
    // State
    ecus,
    selectedEcu,
    dtcs,
    isInitialized,
    isLoading,
    error,
    protocol,

    // Actions
    getEcus,
    switchToKLine,
    switchToDCan,
    initKLine,
    readDtcs,
    clearDtcs,
    readEcuId,
    testerPresent,
    selectEcu,
    reset,
    setSelectedEcu,
  }
}
