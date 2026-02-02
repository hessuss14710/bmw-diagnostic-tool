import { useState, useEffect } from "react"
import { useDSC } from "@/hooks/useDSC"
import { RefreshCw, AlertTriangle, CircleDot, Droplets } from "lucide-react"
import type { EcuInfo } from "@/hooks/useBMW"

interface DSCPanelProps {
  isConnected: boolean
  isInitialized: boolean
  selectedEcu: EcuInfo | null
}

export function DSCPanel({ isConnected, isInitialized, selectedEcu }: DSCPanelProps) {
  const dsc = useDSC()
  const [autoRefresh, setAutoRefresh] = useState(false)

  // Auto-refresh wheel speeds
  useEffect(() => {
    if (!autoRefresh || !isInitialized) return

    const interval = setInterval(() => {
      dsc.readWheelSpeeds().catch(() => {})
      dsc.readSensors().catch(() => {})
    }, 500)

    return () => clearInterval(interval)
  }, [autoRefresh, isInitialized])

  const canOperate = isConnected && isInitialized && selectedEcu?.id === "DSC"

  return (
    <div className="rounded-lg border border-zinc-800 bg-zinc-950 p-4 space-y-4">
      <div className="flex items-center justify-between">
        <h3 className="text-lg font-semibold flex items-center gap-2">
          <CircleDot className="h-5 w-5 text-blue-500" />
          DSC - Dynamic Stability Control
        </h3>
        {dsc.isLoading && (
          <RefreshCw className="h-4 w-4 animate-spin text-blue-500" />
        )}
      </div>

      {dsc.error && (
        <div className="p-3 rounded-lg bg-red-900/30 border border-red-800 text-red-300 text-sm">
          {dsc.error}
        </div>
      )}

      {/* Wheel Speeds */}
      <div className="space-y-2">
        <div className="flex items-center justify-between">
          <h4 className="text-sm font-medium text-zinc-400">Wheel Speeds</h4>
          <label className="flex items-center gap-2 text-xs text-zinc-500">
            <input
              type="checkbox"
              checked={autoRefresh}
              onChange={(e) => setAutoRefresh(e.target.checked)}
              disabled={!canOperate}
              className="rounded border-zinc-700"
            />
            Auto-refresh
          </label>
        </div>

        <div className="grid grid-cols-2 gap-3">
          <div className="p-3 rounded-lg bg-zinc-900 border border-zinc-800">
            <div className="text-xs text-zinc-500 mb-1">Front Left</div>
            <div className="text-xl font-mono">
              {dsc.wheelSpeeds?.front_left?.toFixed(1) ?? "--"}
              <span className="text-xs text-zinc-500 ml-1">km/h</span>
            </div>
          </div>
          <div className="p-3 rounded-lg bg-zinc-900 border border-zinc-800">
            <div className="text-xs text-zinc-500 mb-1">Front Right</div>
            <div className="text-xl font-mono">
              {dsc.wheelSpeeds?.front_right?.toFixed(1) ?? "--"}
              <span className="text-xs text-zinc-500 ml-1">km/h</span>
            </div>
          </div>
          <div className="p-3 rounded-lg bg-zinc-900 border border-zinc-800">
            <div className="text-xs text-zinc-500 mb-1">Rear Left</div>
            <div className="text-xl font-mono">
              {dsc.wheelSpeeds?.rear_left?.toFixed(1) ?? "--"}
              <span className="text-xs text-zinc-500 ml-1">km/h</span>
            </div>
          </div>
          <div className="p-3 rounded-lg bg-zinc-900 border border-zinc-800">
            <div className="text-xs text-zinc-500 mb-1">Rear Right</div>
            <div className="text-xl font-mono">
              {dsc.wheelSpeeds?.rear_right?.toFixed(1) ?? "--"}
              <span className="text-xs text-zinc-500 ml-1">km/h</span>
            </div>
          </div>
        </div>

        <button
          onClick={() => dsc.readWheelSpeeds()}
          disabled={!canOperate || dsc.isLoading}
          className="w-full py-2 rounded-lg bg-zinc-800 hover:bg-zinc-700 disabled:opacity-50 disabled:cursor-not-allowed text-sm"
        >
          Read Wheel Speeds
        </button>
      </div>

      {/* Sensors */}
      <div className="space-y-2">
        <h4 className="text-sm font-medium text-zinc-400">Sensors</h4>
        <div className="grid grid-cols-2 gap-3">
          <div className="p-3 rounded-lg bg-zinc-900 border border-zinc-800">
            <div className="text-xs text-zinc-500 mb-1">Steering Angle</div>
            <div className="text-lg font-mono">
              {dsc.sensors?.steering_angle?.toFixed(1) ?? "--"}
              <span className="text-xs text-zinc-500 ml-1">deg</span>
            </div>
          </div>
          <div className="p-3 rounded-lg bg-zinc-900 border border-zinc-800">
            <div className="text-xs text-zinc-500 mb-1">Yaw Rate</div>
            <div className="text-lg font-mono">
              {dsc.sensors?.yaw_rate?.toFixed(2) ?? "--"}
              <span className="text-xs text-zinc-500 ml-1">deg/s</span>
            </div>
          </div>
          <div className="p-3 rounded-lg bg-zinc-900 border border-zinc-800">
            <div className="text-xs text-zinc-500 mb-1">Lateral Accel</div>
            <div className="text-lg font-mono">
              {dsc.sensors?.lateral_acceleration?.toFixed(3) ?? "--"}
              <span className="text-xs text-zinc-500 ml-1">g</span>
            </div>
          </div>
          <div className="p-3 rounded-lg bg-zinc-900 border border-zinc-800">
            <div className="text-xs text-zinc-500 mb-1">Brake Pressure</div>
            <div className="text-lg font-mono">
              {dsc.sensors?.brake_pressure?.toFixed(1) ?? "--"}
              <span className="text-xs text-zinc-500 ml-1">bar</span>
            </div>
          </div>
        </div>

        <button
          onClick={() => dsc.readSensors()}
          disabled={!canOperate || dsc.isLoading}
          className="w-full py-2 rounded-lg bg-zinc-800 hover:bg-zinc-700 disabled:opacity-50 disabled:cursor-not-allowed text-sm"
        >
          Read Sensors
        </button>
      </div>

      {/* DTCs */}
      <div className="space-y-2">
        <h4 className="text-sm font-medium text-zinc-400 flex items-center gap-2">
          <AlertTriangle className="h-4 w-4" />
          Fault Codes ({dsc.dtcs.length})
        </h4>

        {dsc.dtcs.length > 0 && (
          <div className="space-y-1 max-h-40 overflow-y-auto">
            {dsc.dtcs.map((dtc, i) => (
              <div key={i} className="p-2 rounded bg-red-900/30 border border-red-800 text-sm">
                <span className="font-mono text-red-300">{dtc.code}</span>
                {dtc.description && (
                  <span className="text-zinc-400 ml-2">{dtc.description}</span>
                )}
              </div>
            ))}
          </div>
        )}

        <button
          onClick={() => dsc.readDtcs()}
          disabled={!canOperate || dsc.isLoading}
          className="w-full py-2 rounded-lg bg-zinc-800 hover:bg-zinc-700 disabled:opacity-50 disabled:cursor-not-allowed text-sm"
        >
          Read DTCs
        </button>
      </div>

      {/* Brake Bleed */}
      <div className="space-y-2">
        <h4 className="text-sm font-medium text-zinc-400 flex items-center gap-2">
          <Droplets className="h-4 w-4" />
          ABS Brake Bleed
        </h4>
        <p className="text-xs text-zinc-500">
          Requires extended session and security access. Engine must be off.
        </p>
        <div className="grid grid-cols-5 gap-2">
          {["FL", "FR", "RL", "RR", "ALL"].map((corner) => (
            <button
              key={corner}
              onClick={() => dsc.bleedBrakes(corner as "FL" | "FR" | "RL" | "RR" | "ALL")}
              disabled={!canOperate || dsc.isLoading}
              className="py-2 rounded-lg bg-amber-900/30 border border-amber-800 hover:bg-amber-900/50 disabled:opacity-50 disabled:cursor-not-allowed text-sm text-amber-300"
            >
              {corner}
            </button>
          ))}
        </div>
      </div>
    </div>
  )
}
