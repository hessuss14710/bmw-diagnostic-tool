import { useEffect, useRef } from "react"
import { Button } from "@/components/ui/button"
import { Gauge } from "./Gauge"
import { LiveChart } from "./LiveChart"
import { useLiveData, type LiveDataValue } from "@/hooks/useLiveData"
import type { EcuInfo } from "@/hooks/useBMW"
import {
  Play,
  Square,
  Trash2,
  XCircle,
  Loader2,
  Activity,
} from "lucide-react"

interface LiveDataPanelProps {
  isConnected: boolean
  isInitialized: boolean
  selectedEcu: EcuInfo | null
  onLiveDataUpdate?: (data: Map<number, LiveDataValue>) => void
}

export function LiveDataPanel({
  isConnected,
  isInitialized,
  selectedEcu,
  onLiveDataUpdate,
}: LiveDataPanelProps) {
  const {
    availablePids,
    selectedPids,
    liveData,
    history,
    isPolling,
    error,
    getAvailablePids,
    startPolling,
    stopPolling,
    togglePid,
    selectAllPids,
    clearSelectedPids,
    clearHistory,
    setTargetAddress,
    getValue,
  } = useLiveData()

  // Load available PIDs on mount
  useEffect(() => {
    getAvailablePids()
  }, [getAvailablePids])

  // Update target address when ECU changes
  useEffect(() => {
    if (selectedEcu?.kline_address) {
      setTargetAddress(selectedEcu.kline_address)
    }
  }, [selectedEcu, setTargetAddress])

  // Stop polling when disconnected or uninitialized
  useEffect(() => {
    if (!isConnected || !isInitialized) {
      stopPolling()
    }
  }, [isConnected, isInitialized, stopPolling])

  // Send live data to server when it changes
  const lastUpdateRef = useRef<number>(0)
  useEffect(() => {
    if (liveData.size > 0 && onLiveDataUpdate) {
      // Throttle updates to max 2 per second
      const now = Date.now()
      if (now - lastUpdateRef.current >= 500) {
        onLiveDataUpdate(liveData)
        lastUpdateRef.current = now
      }
    }
  }, [liveData, onLiveDataUpdate])

  const handleStartPolling = () => {
    if (selectedEcu?.kline_address) {
      startPolling(selectedEcu.kline_address)
    }
  }

  // Get PID definition by ID
  const getPidDef = (pid: number) => availablePids.find((p) => p.id === pid)

  // Primary gauges to show (RPM, Coolant, Speed, Throttle)
  const primaryPids = [0x0c, 0x05, 0x0d, 0x11]

  if (!isConnected) {
    return (
      <div className="rounded-lg border border-zinc-800 bg-zinc-950 p-6">
        <div className="flex flex-col items-center justify-center py-4 text-zinc-500">
          <XCircle className="h-12 w-12 text-zinc-600 mb-2" />
          <p className="text-sm">Connect to your K+DCAN cable first</p>
        </div>
      </div>
    )
  }

  if (!isInitialized || !selectedEcu) {
    return (
      <div className="rounded-lg border border-zinc-800 bg-zinc-950 p-6">
        <div className="flex flex-col items-center justify-center py-4 text-zinc-500">
          <Activity className="h-12 w-12 text-zinc-600 mb-2" />
          <p className="text-sm">Select and initialize an ECU first</p>
        </div>
      </div>
    )
  }

  return (
    <div className="space-y-4">
      {/* Controls */}
      <div className="rounded-lg border border-zinc-800 bg-zinc-950 p-4">
        <div className="flex items-center justify-between mb-4">
          <h3 className="font-semibold flex items-center gap-2">
            <Activity className="h-4 w-4" />
            Live Data
          </h3>
          <div className="flex gap-2">
            {!isPolling ? (
              <Button
                size="sm"
                onClick={handleStartPolling}
                disabled={selectedPids.length === 0}
                className="bg-green-600 hover:bg-green-700"
              >
                <Play className="h-4 w-4 mr-1" />
                Start
              </Button>
            ) : (
              <Button
                size="sm"
                onClick={stopPolling}
                className="bg-red-600 hover:bg-red-700"
              >
                <Square className="h-4 w-4 mr-1" />
                Stop
              </Button>
            )}
            <Button
              size="sm"
              variant="outline"
              onClick={clearHistory}
              className="border-zinc-700"
            >
              <Trash2 className="h-4 w-4" />
            </Button>
          </div>
        </div>

        {/* PID Selection */}
        <div className="mb-4">
          <div className="flex items-center justify-between mb-2">
            <span className="text-xs text-zinc-400">Select PIDs to monitor:</span>
            <div className="flex gap-1">
              <button
                onClick={selectAllPids}
                className="text-xs text-blue-400 hover:text-blue-300"
              >
                All
              </button>
              <span className="text-zinc-600">|</span>
              <button
                onClick={clearSelectedPids}
                className="text-xs text-zinc-400 hover:text-zinc-300"
              >
                None
              </button>
            </div>
          </div>
          <div className="flex flex-wrap gap-2">
            {availablePids.map((pid) => (
              <button
                key={pid.id}
                onClick={() => togglePid(pid.id)}
                disabled={isPolling}
                className={`px-2 py-1 rounded text-xs transition-colors ${
                  selectedPids.includes(pid.id)
                    ? "bg-blue-600 text-white"
                    : "bg-zinc-800 text-zinc-400 hover:bg-zinc-700"
                } ${isPolling ? "opacity-50 cursor-not-allowed" : ""}`}
              >
                {pid.short_name}
              </button>
            ))}
          </div>
        </div>

        {/* Polling status */}
        {isPolling && (
          <div className="flex items-center gap-2 text-xs text-green-400">
            <Loader2 className="h-3 w-3 animate-spin" />
            Polling {selectedPids.length} PIDs...
          </div>
        )}
      </div>

      {/* Error Display */}
      {error && (
        <div className="rounded-lg border border-red-900 bg-red-950 p-3 text-sm text-red-400">
          {error}
        </div>
      )}

      {/* Primary Gauges */}
      {liveData.size > 0 && (
        <div className="rounded-lg border border-zinc-800 bg-zinc-950 p-4">
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
            {primaryPids
              .filter((pid) => selectedPids.includes(pid))
              .map((pid) => {
                const def = getPidDef(pid)
                const value = getValue(pid)
                if (!def) return null
                return (
                  <Gauge
                    key={pid}
                    value={value?.value ?? 0}
                    min={def.min}
                    max={def.max}
                    unit={def.unit}
                    label={def.short_name}
                    format={def.format as "temperature" | "rpm" | "percent" | "speed" | "voltage"}
                  />
                )
              })}
          </div>
        </div>
      )}

      {/* Charts for selected PIDs */}
      {history.size > 0 && (
        <div className="space-y-3">
          {Array.from(history.entries())
            .filter(([pid]) => selectedPids.includes(pid))
            .map(([pid, pidHistory]) => {
              const def = getPidDef(pid)
              if (!def || pidHistory.values.length < 2) return null
              return <LiveChart key={pid} history={pidHistory} definition={def} />
            })}
        </div>
      )}

      {/* Data Table */}
      {liveData.size > 0 && (
        <div className="rounded-lg border border-zinc-800 bg-zinc-950 overflow-hidden">
          <table className="w-full text-sm">
            <thead className="bg-zinc-900">
              <tr className="text-left text-zinc-400">
                <th className="px-4 py-2">Parameter</th>
                <th className="px-4 py-2 text-right">Value</th>
                <th className="px-4 py-2 text-right">Unit</th>
              </tr>
            </thead>
            <tbody>
              {Array.from(liveData.values())
                .filter((v) => selectedPids.includes(v.pid))
                .map((data) => (
                  <tr
                    key={data.pid}
                    className="border-t border-zinc-800 hover:bg-zinc-900"
                  >
                    <td className="px-4 py-2 text-white">{data.name}</td>
                    <td className="px-4 py-2 text-right font-mono text-green-400">
                      {data.value.toFixed(1)}
                    </td>
                    <td className="px-4 py-2 text-right text-zinc-400">
                      {data.unit}
                    </td>
                  </tr>
                ))}
            </tbody>
          </table>
        </div>
      )}

      {/* Help text when no data */}
      {liveData.size === 0 && !isPolling && (
        <div className="rounded-lg border border-zinc-800 bg-zinc-950 p-4 text-center text-sm text-zinc-500">
          <p>Select PIDs above and click "Start" to begin monitoring</p>
          <p className="text-xs mt-1">
            Engine should be running for most PIDs
          </p>
        </div>
      )}
    </div>
  )
}
