import { invoke } from "@tauri-apps/api/core"
import { useCallback } from "react"

/**
 * Default timeout in milliseconds (30 seconds)
 */
const DEFAULT_TIMEOUT_MS = 30000

/**
 * Custom error class for timeout errors
 */
export class InvokeTimeoutError extends Error {
  constructor(command: string, timeoutMs: number) {
    super(`Command "${command}" timed out after ${timeoutMs}ms`)
    this.name = "InvokeTimeoutError"
  }
}

/**
 * Invoke a Tauri command with a timeout
 *
 * Prevents commands from hanging indefinitely by racing against a timeout promise.
 *
 * @param command - The Tauri command to invoke
 * @param args - Arguments to pass to the command
 * @param timeoutMs - Timeout in milliseconds (default: 30000)
 * @returns Promise that resolves with the command result or rejects on timeout
 *
 * @example
 * ```tsx
 * const result = await invokeWithTimeout<DpfStatus>(
 *   "bmw_dpf_read_status",
 *   { targetAddress: 0x12 },
 *   5000 // 5 second timeout
 * )
 * ```
 */
export async function invokeWithTimeout<T>(
  command: string,
  args?: Record<string, unknown>,
  timeoutMs: number = DEFAULT_TIMEOUT_MS
): Promise<T> {
  const timeoutPromise = new Promise<never>((_, reject) => {
    setTimeout(() => {
      reject(new InvokeTimeoutError(command, timeoutMs))
    }, timeoutMs)
  })

  return Promise.race([invoke<T>(command, args), timeoutPromise])
}

/**
 * Hook that provides an invoke function with timeout support
 *
 * @param defaultTimeoutMs - Default timeout for all invocations
 * @returns Object with invoke function and timeout utilities
 *
 * @example
 * ```tsx
 * const { invokeCmd } = useInvokeWithTimeout(5000)
 *
 * const readStatus = async () => {
 *   try {
 *     const result = await invokeCmd<DpfStatus>("bmw_dpf_read_status")
 *     setStatus(result)
 *   } catch (e) {
 *     if (e instanceof InvokeTimeoutError) {
 *       setError("Operation timed out")
 *     } else {
 *       setError(e.message)
 *     }
 *   }
 * }
 * ```
 */
export function useInvokeWithTimeout(
  defaultTimeoutMs: number = DEFAULT_TIMEOUT_MS
) {
  const invokeCmd = useCallback(
    async <T>(
      command: string,
      args?: Record<string, unknown>,
      timeoutMs?: number
    ): Promise<T> => {
      return invokeWithTimeout<T>(
        command,
        args,
        timeoutMs ?? defaultTimeoutMs
      )
    },
    [defaultTimeoutMs]
  )

  return {
    invokeCmd,
    InvokeTimeoutError,
  }
}

/**
 * Check if an error is a timeout error
 */
export function isTimeoutError(error: unknown): error is InvokeTimeoutError {
  return error instanceof InvokeTimeoutError
}
