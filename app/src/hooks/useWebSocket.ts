import { io, Socket } from "socket.io-client"
import { useState, useCallback, useEffect, useRef } from "react"
import type { Dtc } from "./useBMW"
import type { LiveDataValue } from "./useLiveData"

// Server URL - can be configured via environment
const SERVER_URL = import.meta.env.VITE_SERVER_URL || "https://taller.giormo.com"

interface VehicleInfo {
  model?: string
  vin?: string
  mileage?: string
}

export function useWebSocket() {
  const [isConnected, setIsConnected] = useState(false)
  const [sessionId, setSessionId] = useState<string | null>(null)
  const [dashboardCount, setDashboardCount] = useState(0)
  const [error, setError] = useState<string | null>(null)

  const socketRef = useRef<Socket | null>(null)
  const vehicleInfoRef = useRef<VehicleInfo>({})

  // Connect to server
  const connect = useCallback((vehicleInfo?: VehicleInfo) => {
    if (socketRef.current?.connected) {
      return
    }

    // Store vehicle info for session creation
    if (vehicleInfo) {
      vehicleInfoRef.current = vehicleInfo
    }

    const socket = io(`${SERVER_URL}/app`, {
      transports: ["websocket", "polling"],
      reconnection: true,
      reconnectionAttempts: 5,
      reconnectionDelay: 1000,
    })

    socket.on("connect", () => {
      console.log("[WebSocket] Connected to server")
      setIsConnected(true)
      setError(null)

      // Create/join session
      const newSessionId = `session-${Date.now()}`
      socket.emit("session:create", {
        sessionId: newSessionId,
        vehicleInfo: vehicleInfoRef.current,
      })
    })

    socket.on("session:joined", (data: { sessionId: string }) => {
      console.log("[WebSocket] Session joined:", data.sessionId)
      setSessionId(data.sessionId)
    })

    socket.on("dashboard:connected", (data: { dashboardCount: number }) => {
      setDashboardCount(data.dashboardCount)
      console.log("[WebSocket] Dashboard connected. Total:", data.dashboardCount)
    })

    socket.on("livedata:request", (data: { action: "start" | "stop"; pids?: number[] }) => {
      // Dashboard is requesting live data - we'll handle this via callback
      console.log("[WebSocket] Live data request:", data)
    })

    socket.on("disconnect", () => {
      console.log("[WebSocket] Disconnected from server")
      setIsConnected(false)
    })

    socket.on("connect_error", (err) => {
      console.error("[WebSocket] Connection error:", err)
      setError(`Connection error: ${err.message}`)
    })

    socketRef.current = socket
  }, [])

  // Disconnect from server
  const disconnect = useCallback(() => {
    if (socketRef.current) {
      socketRef.current.disconnect()
      socketRef.current = null
      setIsConnected(false)
      setSessionId(null)
    }
  }, [])

  // Update vehicle info
  const updateVehicleInfo = useCallback((info: VehicleInfo) => {
    vehicleInfoRef.current = { ...vehicleInfoRef.current, ...info }

    if (socketRef.current?.connected && sessionId) {
      // Re-emit session:create to update vehicle info
      socketRef.current.emit("session:create", {
        sessionId,
        vehicleInfo: vehicleInfoRef.current,
      })
    }
  }, [sessionId])

  // Send DTCs to server
  const sendDtcs = useCallback((dtcs: Dtc[]) => {
    if (!socketRef.current?.connected) {
      return
    }

    const dtcData = dtcs.map((dtc) => ({
      code: dtc.code,
      status: {
        confirmed: dtc.status.confirmed,
        pending: dtc.status.pending,
        test_failed: dtc.status.test_failed,
      },
    }))

    socketRef.current.emit("dtcs:update", { dtcs: dtcData })
    console.log("[WebSocket] Sent DTCs:", dtcData.length)
  }, [])

  // Send live data to server
  const sendLiveData = useCallback((data: Map<number, LiveDataValue>) => {
    if (!socketRef.current?.connected) {
      return
    }

    // Convert Map to object with named keys
    const liveDataObj: Record<string, { value: number; unit: string }> = {}

    data.forEach((value, pid) => {
      // Use short name or create a key from PID
      const key = value.name.toLowerCase().replace(/\s+/g, "_") || `pid_${pid}`
      liveDataObj[key] = {
        value: value.value,
        unit: value.unit,
      }
    })

    socketRef.current.emit("livedata:update", liveDataObj)
  }, [])

  // Send ECU status
  const sendEcuStatus = useCallback((connected: boolean, ecu?: string, protocol?: string) => {
    if (!socketRef.current?.connected) {
      return
    }

    socketRef.current.emit("ecu:status", { connected, ecu, protocol })
  }, [])

  // Auto-connect on mount (optional - can be disabled)
  useEffect(() => {
    // Don't auto-connect, let the app control this
    return () => {
      if (socketRef.current) {
        socketRef.current.disconnect()
      }
    }
  }, [])

  return {
    // State
    isConnected,
    sessionId,
    dashboardCount,
    error,

    // Actions
    connect,
    disconnect,
    updateVehicleInfo,
    sendDtcs,
    sendLiveData,
    sendEcuStatus,
  }
}
