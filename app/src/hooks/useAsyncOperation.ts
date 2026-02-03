import { invoke } from "@tauri-apps/api/core"
import { useState, useCallback } from "react"

/**
 * State for async operations
 */
interface AsyncState<T> {
  data: T | null
  isLoading: boolean
  error: string | null
}

/**
 * Options for useAsyncOperation hook
 */
interface UseAsyncOperationOptions<T> {
  /** Custom validation function to check if result is successful */
  validateSuccess?: (result: T) => boolean
  /** Path to error message in result object (e.g., "message" or "error.message") */
  errorMessagePath?: string
  /** Called when operation succeeds */
  onSuccess?: (result: T) => void
  /** Called when operation fails */
  onError?: (error: Error) => void
}

/**
 * Get nested value from object using dot notation path
 */
function getNestedValue<T>(obj: T, path: string): unknown {
  return path.split(".").reduce((current: unknown, prop: string) => {
    if (current && typeof current === "object" && prop in current) {
      return (current as Record<string, unknown>)[prop]
    }
    return undefined
  }, obj)
}

/**
 * Generic hook for async Tauri invoke operations with loading/error state
 *
 * Consolidates the repetitive pattern of:
 * - setIsLoading(true)
 * - setError(null)
 * - try { invoke() } catch { setError() }
 * - finally { setIsLoading(false) }
 *
 * @example
 * ```tsx
 * const { data, isLoading, error, execute } = useAsyncOperation<DpfStatus>()
 *
 * const readStatus = async () => {
 *   await execute("bmw_dpf_read_status", { targetAddress: 0x12 })
 * }
 * ```
 */
export function useAsyncOperation<T = unknown>(
  options: UseAsyncOperationOptions<T> = {}
) {
  const [state, setState] = useState<AsyncState<T>>({
    data: null,
    isLoading: false,
    error: null,
  })

  /**
   * Execute a Tauri invoke command
   */
  const execute = useCallback(
    async (
      command: string,
      args?: Record<string, unknown>
    ): Promise<T | null> => {
      setState((prev) => ({ ...prev, isLoading: true, error: null }))

      try {
        const result = await invoke<T>(command, args)

        // Custom validation if provided
        if (options.validateSuccess && !options.validateSuccess(result)) {
          // Try to extract error message from result
          let errorMsg = "Operation failed"
          if (options.errorMessagePath) {
            const extracted = getNestedValue(result, options.errorMessagePath)
            if (typeof extracted === "string") {
              errorMsg = extracted
            }
          }
          throw new Error(errorMsg)
        }

        setState((prev) => ({ ...prev, data: result, isLoading: false }))
        options.onSuccess?.(result)
        return result
      } catch (e) {
        const errorMsg = e instanceof Error ? e.message : String(e)
        setState((prev) => ({ ...prev, error: errorMsg, isLoading: false }))
        options.onError?.(e instanceof Error ? e : new Error(errorMsg))
        throw e
      }
    },
    [options]
  )

  /**
   * Execute without throwing (returns null on error)
   */
  const executeSafe = useCallback(
    async (
      command: string,
      args?: Record<string, unknown>
    ): Promise<T | null> => {
      try {
        return await execute(command, args)
      } catch {
        return null
      }
    },
    [execute]
  )

  /**
   * Reset state to initial values
   */
  const reset = useCallback(() => {
    setState({ data: null, isLoading: false, error: null })
  }, [])

  /**
   * Clear error only
   */
  const clearError = useCallback(() => {
    setState((prev) => ({ ...prev, error: null }))
  }, [])

  /**
   * Set data manually (useful for optimistic updates)
   */
  const setData = useCallback((data: T | null) => {
    setState((prev) => ({ ...prev, data }))
  }, [])

  return {
    ...state,
    execute,
    executeSafe,
    reset,
    clearError,
    setData,
  }
}

/**
 * Hook for simple loading state management
 */
export function useLoadingState(initialError: string | null = null) {
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(initialError)

  const startLoading = useCallback(() => {
    setIsLoading(true)
    setError(null)
  }, [])

  const stopLoading = useCallback(() => {
    setIsLoading(false)
  }, [])

  const setErrorMsg = useCallback((msg: string | null) => {
    setError(msg)
    setIsLoading(false)
  }, [])

  const clearError = useCallback(() => {
    setError(null)
  }, [])

  const reset = useCallback(() => {
    setIsLoading(false)
    setError(null)
  }, [])

  return {
    isLoading,
    error,
    startLoading,
    stopLoading,
    setError: setErrorMsg,
    clearError,
    reset,
  }
}
