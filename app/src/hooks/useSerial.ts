import { invoke } from "@tauri-apps/api/core"
import { useState, useCallback } from "react"

export interface PortInfo {
  name: string
  port_type: string
  vid: number | null
  pid: number | null
  manufacturer: string | null
  product: string | null
  serial_number: string | null
  is_ftdi: boolean
}

export interface ConnectionStatus {
  state: "disconnected" | "connecting" | "connected" | "error"
  port: string | null
  error: string | null
}

export function useSerial() {
  const [ports, setPorts] = useState<PortInfo[]>([])
  const [status, setStatus] = useState<ConnectionStatus>({
    state: "disconnected",
    port: null,
    error: null,
  })
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  // List available serial ports
  const listPorts = useCallback(async () => {
    setIsLoading(true)
    setError(null)
    try {
      const result = await invoke<PortInfo[]>("list_serial_ports")
      setPorts(result)
      return result
    } catch (e) {
      const errorMsg = e instanceof Error ? e.message : String(e)
      setError(errorMsg)
      return []
    } finally {
      setIsLoading(false)
    }
  }, [])

  // Connect to a port
  const connect = useCallback(async (portName: string, baudRate?: number) => {
    setIsLoading(true)
    setError(null)
    try {
      const result = await invoke<ConnectionStatus>("serial_connect", {
        portName,
        baudRate,
      })
      setStatus(result)
      return result
    } catch (e) {
      const errorMsg = e instanceof Error ? e.message : String(e)
      setError(errorMsg)
      setStatus({ state: "error", port: null, error: errorMsg })
      throw e
    } finally {
      setIsLoading(false)
    }
  }, [])

  // Disconnect
  const disconnect = useCallback(async () => {
    setIsLoading(true)
    setError(null)
    try {
      const result = await invoke<ConnectionStatus>("serial_disconnect")
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

  // Get current status
  const getStatus = useCallback(async () => {
    try {
      const result = await invoke<ConnectionStatus>("serial_status")
      setStatus(result)
      return result
    } catch (e) {
      const errorMsg = e instanceof Error ? e.message : String(e)
      setError(errorMsg)
      throw e
    }
  }, [])

  // Send raw bytes
  const write = useCallback(async (data: number[]) => {
    try {
      return await invoke<number>("serial_write", { data })
    } catch (e) {
      const errorMsg = e instanceof Error ? e.message : String(e)
      setError(errorMsg)
      throw e
    }
  }, [])

  // Read available bytes
  const read = useCallback(async () => {
    try {
      return await invoke<number[]>("serial_read")
    } catch (e) {
      const errorMsg = e instanceof Error ? e.message : String(e)
      setError(errorMsg)
      throw e
    }
  }, [])

  // Send hex command and get response
  const sendHex = useCallback(async (hexData: string) => {
    try {
      return await invoke<string>("serial_send_hex", { hexData })
    } catch (e) {
      const errorMsg = e instanceof Error ? e.message : String(e)
      setError(errorMsg)
      throw e
    }
  }, [])

  // Set DTR line
  const setDTR = useCallback(async (level: boolean) => {
    try {
      await invoke("serial_set_dtr", { level })
    } catch (e) {
      const errorMsg = e instanceof Error ? e.message : String(e)
      setError(errorMsg)
      throw e
    }
  }, [])

  // Set RTS line
  const setRTS = useCallback(async (level: boolean) => {
    try {
      await invoke("serial_set_rts", { level })
    } catch (e) {
      const errorMsg = e instanceof Error ? e.message : String(e)
      setError(errorMsg)
      throw e
    }
  }, [])

  // Set baud rate
  const setBaudRate = useCallback(async (baudRate: number) => {
    try {
      await invoke("serial_set_baud", { baudRate })
    } catch (e) {
      const errorMsg = e instanceof Error ? e.message : String(e)
      setError(errorMsg)
      throw e
    }
  }, [])

  // Clear buffers
  const clearBuffers = useCallback(async () => {
    try {
      await invoke("serial_clear")
    } catch (e) {
      const errorMsg = e instanceof Error ? e.message : String(e)
      setError(errorMsg)
      throw e
    }
  }, [])

  return {
    // State
    ports,
    status,
    isLoading,
    error,
    isConnected: status.state === "connected",

    // Actions
    listPorts,
    connect,
    disconnect,
    getStatus,
    write,
    read,
    sendHex,
    setDTR,
    setRTS,
    setBaudRate,
    clearBuffers,
  }
}
