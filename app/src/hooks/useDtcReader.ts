import { invoke } from "@tauri-apps/api/core"
import { useState, useCallback } from "react"
import type { Dtc, DtcReadResult } from "./useBMW"

/**
 * Options for useDtcReader hook
 */
interface UseDtcReaderOptions {
  /**
   * The Tauri command name for reading DTCs
   * Defaults to "bmw_read_dtcs_kline"
   */
  command?: string
  /**
   * Called when DTCs are successfully read
   */
  onSuccess?: (dtcs: Dtc[]) => void
  /**
   * Called when reading fails
   */
  onError?: (error: Error) => void
}

/**
 * Shared hook for reading DTCs across multiple ECU modules
 *
 * This consolidates the repetitive DTC reading pattern used in:
 * - useDPF
 * - useDSC
 * - useEGS
 * - useFRM
 * - useKOMBI
 *
 * @example
 * ```tsx
 * // In useDSC.ts
 * const dtcReader = useDtcReader({ command: "bmw_dsc_read_dtcs" })
 *
 * return {
 *   dtcs: dtcReader.dtcs,
 *   readDtcs: dtcReader.readDtcs,
 *   isLoading: dtcReader.isLoading,
 *   error: dtcReader.error,
 * }
 * ```
 */
export function useDtcReader(options: UseDtcReaderOptions = {}) {
  const { command = "bmw_read_dtcs_kline", onSuccess, onError } = options

  const [dtcs, setDtcs] = useState<Dtc[]>([])
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  /**
   * Read DTCs from the ECU
   */
  const readDtcs = useCallback(
    async (targetAddress?: number): Promise<DtcReadResult | null> => {
      setIsLoading(true)
      setError(null)

      try {
        const result = await invoke<DtcReadResult>(command, {
          targetAddress,
        })

        if (result.success) {
          setDtcs(result.dtcs)
          onSuccess?.(result.dtcs)
        } else {
          setError(result.message)
          onError?.(new Error(result.message))
        }

        return result
      } catch (e) {
        const errorMsg = e instanceof Error ? e.message : String(e)
        setError(errorMsg)
        onError?.(e instanceof Error ? e : new Error(errorMsg))
        throw e
      } finally {
        setIsLoading(false)
      }
    },
    [command, onSuccess, onError]
  )

  /**
   * Clear DTCs from state
   */
  const clearDtcs = useCallback(() => {
    setDtcs([])
  }, [])

  /**
   * Clear error message
   */
  const clearError = useCallback(() => {
    setError(null)
  }, [])

  /**
   * Reset all state
   */
  const reset = useCallback(() => {
    setDtcs([])
    setIsLoading(false)
    setError(null)
  }, [])

  /**
   * Check if there are any confirmed DTCs
   */
  const hasConfirmedDtcs = dtcs.some((dtc) => dtc.status?.confirmed)

  /**
   * Check if there are any pending DTCs
   */
  const hasPendingDtcs = dtcs.some((dtc) => dtc.status?.pending)

  /**
   * Get count of DTCs by severity
   */
  const dtcCounts = {
    total: dtcs.length,
    confirmed: dtcs.filter((d) => d.status?.confirmed).length,
    pending: dtcs.filter((d) => d.status?.pending).length,
  }

  return {
    dtcs,
    isLoading,
    error,
    readDtcs,
    clearDtcs,
    clearError,
    reset,
    hasConfirmedDtcs,
    hasPendingDtcs,
    dtcCounts,
  }
}

/**
 * Convenience hook for DSC DTCs
 */
export function useDscDtcs() {
  return useDtcReader({ command: "bmw_dsc_read_dtcs" })
}

/**
 * Convenience hook for KOMBI DTCs
 */
export function useKombiDtcs() {
  return useDtcReader({ command: "bmw_kombi_read_dtcs" })
}

/**
 * Convenience hook for FRM DTCs
 */
export function useFrmDtcs() {
  return useDtcReader({ command: "bmw_frm_read_dtcs" })
}

/**
 * Convenience hook for EGS DTCs
 */
export function useEgsDtcs() {
  return useDtcReader({ command: "bmw_egs_read_dtcs" })
}
