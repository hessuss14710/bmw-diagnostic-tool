import { useState, useEffect } from "react"
import { useEGS } from "@/hooks/useEGS"
import { Settings, RefreshCw, AlertTriangle, Thermometer, Cog } from "lucide-react"
import type { EcuInfo } from "@/hooks/useBMW"

interface EGSPanelProps {
  isConnected: boolean
  isInitialized: boolean
  selectedEcu: EcuInfo | null
}

export function EGSPanel({ isConnected, isInitialized, selectedEcu }: EGSPanelProps) {
  const egs = useEGS()
  const [autoRefresh, setAutoRefresh] = useState(false)
  const [confirmReset, setConfirmReset] = useState(false)

  // Auto-refresh status - egs.readStatus is stable from hook
  useEffect(() => {
    if (!autoRefresh || !isInitialized) return

    const interval = setInterval(() => {
      egs.readStatus().catch(() => {})
    }, 500)

    return () => clearInterval(interval)
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [autoRefresh, isInitialized])

  const canOperate = isConnected && isInitialized && selectedEcu?.id === "EGS"

  const handleResetAdaptations = async () => {
    if (!confirmReset) {
      setConfirmReset(true)
      return
    }
    setConfirmReset(false)
    await egs.resetAdaptations()
  }

  const getOilTempColor = (temp: number | null) => {
    if (temp === null) return "text-zinc-400"
    if (temp < 40) return "text-blue-400" // Cold
    if (temp < 80) return "text-green-400" // Normal
    if (temp < 120) return "text-yellow-400" // Warm
    return "text-red-400" // Hot
  }

  const getOilTempStatus = (temp: number | null) => {
    if (temp === null) return ""
    if (temp < 40) return "(Cold)"
    if (temp < 80) return "(Normal)"
    if (temp < 120) return "(Warm)"
    return "(HOT!)"
  }

  return (
    <div className="rounded-lg border border-zinc-800 bg-zinc-950 p-4 space-y-4">
      <div className="flex items-center justify-between">
        <h3 className="text-lg font-semibold flex items-center gap-2">
          <Settings className="h-5 w-5 text-purple-500" />
          EGS - Automatic Transmission
        </h3>
        {egs.isLoading && (
          <RefreshCw className="h-4 w-4 animate-spin text-purple-500" />
        )}
      </div>

      {egs.error && (
        <div className="p-3 rounded-lg bg-red-900/30 border border-red-800 text-red-300 text-sm">
          {egs.error}
        </div>
      )}

      {/* Transmission Status */}
      <div className="space-y-2">
        <div className="flex items-center justify-between">
          <h4 className="text-sm font-medium text-zinc-400">Status</h4>
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
          {/* Gear Position */}
          <div className="p-4 rounded-lg bg-zinc-900 border border-zinc-800 col-span-2">
            <div className="text-xs text-zinc-500 mb-1">Gear Position</div>
            <div className="flex items-center gap-4">
              <div className="text-4xl font-bold font-mono text-purple-400">
                {egs.status?.gear_position ?? "--"}
              </div>
              {egs.status?.target_gear !== null && egs.status?.actual_gear !== null && (
                <div className="text-sm text-zinc-500">
                  <div>Target: {egs.status?.target_gear}</div>
                  <div>Actual: {egs.status?.actual_gear}</div>
                </div>
              )}
              <div className="ml-auto flex flex-col gap-1">
                {egs.status?.sport_mode && (
                  <span className="px-2 py-1 rounded bg-red-900/50 text-red-300 text-xs">
                    Sport
                  </span>
                )}
                {egs.status?.torque_converter_lockup && (
                  <span className="px-2 py-1 rounded bg-green-900/50 text-green-300 text-xs">
                    TC Locked
                  </span>
                )}
              </div>
            </div>
          </div>

          {/* Oil Temperature */}
          <div className="p-3 rounded-lg bg-zinc-900 border border-zinc-800 col-span-2">
            <div className="flex items-center gap-2 mb-1">
              <Thermometer className="h-4 w-4 text-zinc-500" />
              <span className="text-xs text-zinc-500">Oil Temperature</span>
            </div>
            <div className={`text-2xl font-mono ${getOilTempColor(egs.status?.oil_temp ?? null)}`}>
              {egs.status?.oil_temp?.toFixed(0) ?? "--"}
              <span className="text-sm ml-1">C</span>
              <span className="text-sm ml-2 text-zinc-500">
                {getOilTempStatus(egs.status?.oil_temp ?? null)}
              </span>
            </div>
            {/* Temperature bar */}
            <div className="mt-2 h-2 bg-zinc-800 rounded-full overflow-hidden">
              <div
                className={`h-full transition-all ${
                  (egs.status?.oil_temp ?? 0) < 40
                    ? "bg-blue-500"
                    : (egs.status?.oil_temp ?? 0) < 80
                    ? "bg-green-500"
                    : (egs.status?.oil_temp ?? 0) < 120
                    ? "bg-yellow-500"
                    : "bg-red-500"
                }`}
                style={{
                  width: `${Math.min(100, Math.max(0, ((egs.status?.oil_temp ?? 0) / 150) * 100))}%`
                }}
              />
            </div>
            <div className="flex justify-between text-xs text-zinc-600 mt-1">
              <span>0</span>
              <span>40</span>
              <span>80</span>
              <span>120</span>
              <span>150C</span>
            </div>
          </div>
        </div>

        <button
          onClick={() => egs.readStatus()}
          disabled={!canOperate || egs.isLoading}
          className="w-full py-2 rounded-lg bg-zinc-800 hover:bg-zinc-700 disabled:opacity-50 disabled:cursor-not-allowed text-sm"
        >
          Read Status
        </button>
      </div>

      {/* Functions */}
      <div className="space-y-2">
        <h4 className="text-sm font-medium text-zinc-400 flex items-center gap-2">
          <Cog className="h-4 w-4" />
          Functions
        </h4>
        <button
          onClick={handleResetAdaptations}
          disabled={!canOperate || egs.isLoading}
          className={`w-full py-2 rounded-lg text-sm ${
            confirmReset
              ? "bg-red-600 text-white"
              : "bg-amber-900/30 border border-amber-800 text-amber-300 hover:bg-amber-900/50"
          } disabled:opacity-50 disabled:cursor-not-allowed`}
        >
          {confirmReset ? "Confirm Reset Adaptations?" : "Reset Transmission Adaptations"}
        </button>
        <p className="text-xs text-zinc-500">
          Resets shift point adaptations. The transmission will relearn your driving style.
          Requires extended session.
        </p>
      </div>

      {/* DTCs */}
      <div className="space-y-2">
        <h4 className="text-sm font-medium text-zinc-400 flex items-center gap-2">
          <AlertTriangle className="h-4 w-4" />
          Fault Codes ({egs.dtcs.length})
        </h4>

        {egs.dtcs.length > 0 && (
          <div className="space-y-1 max-h-40 overflow-y-auto">
            {egs.dtcs.map((dtc, i) => (
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
          onClick={() => egs.readDtcs()}
          disabled={!canOperate || egs.isLoading}
          className="w-full py-2 rounded-lg bg-zinc-800 hover:bg-zinc-700 disabled:opacity-50 disabled:cursor-not-allowed text-sm"
        >
          Read DTCs
        </button>
      </div>
    </div>
  )
}
