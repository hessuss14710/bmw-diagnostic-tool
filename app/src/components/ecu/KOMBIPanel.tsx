import { useState } from "react"
import { useKOMBI } from "@/hooks/useKOMBI"
import { Gauge, RefreshCw, AlertTriangle, Wrench, Car, Clock } from "lucide-react"
import type { EcuInfo } from "@/hooks/useBMW"

interface KOMBIPanelProps {
  isConnected: boolean
  isInitialized: boolean
  selectedEcu: EcuInfo | null
}

export function KOMBIPanel({ isConnected, isInitialized, selectedEcu }: KOMBIPanelProps) {
  const kombi = useKOMBI()
  const [confirmReset, setConfirmReset] = useState<string | null>(null)

  const canOperate = isConnected && isInitialized && selectedEcu?.id === "KOMBI"

  const handleResetService = async (serviceType: "oil" | "inspection" | "brake_fluid" | "all") => {
    if (confirmReset !== serviceType) {
      setConfirmReset(serviceType)
      return
    }
    setConfirmReset(null)
    await kombi.resetService(serviceType)
    await kombi.readServiceInfo()
  }

  const formatDistance = (km: number | null) => {
    if (km === null) return "--"
    return km >= 0 ? `${km} km` : `${Math.abs(km)} km overdue`
  }

  const formatDays = (days: number | null) => {
    if (days === null) return "--"
    return days >= 0 ? `${days} days` : `${Math.abs(days)} days overdue`
  }

  return (
    <div className="rounded-lg border border-zinc-800 bg-zinc-950 p-4 space-y-4">
      <div className="flex items-center justify-between">
        <h3 className="text-lg font-semibold flex items-center gap-2">
          <Gauge className="h-5 w-5 text-green-500" />
          KOMBI - Instrument Cluster
        </h3>
        {kombi.isLoading && (
          <RefreshCw className="h-4 w-4 animate-spin text-green-500" />
        )}
      </div>

      {kombi.error && (
        <div className="p-3 rounded-lg bg-red-900/30 border border-red-800 text-red-300 text-sm">
          {kombi.error}
        </div>
      )}

      {/* Vehicle Info */}
      <div className="space-y-2">
        <h4 className="text-sm font-medium text-zinc-400 flex items-center gap-2">
          <Car className="h-4 w-4" />
          Vehicle Information
        </h4>
        <div className="grid grid-cols-2 gap-3">
          <div className="p-3 rounded-lg bg-zinc-900 border border-zinc-800">
            <div className="text-xs text-zinc-500 mb-1">VIN</div>
            <div className="text-sm font-mono truncate">
              {kombi.vehicleInfo?.vin ?? "--"}
            </div>
          </div>
          <div className="p-3 rounded-lg bg-zinc-900 border border-zinc-800">
            <div className="text-xs text-zinc-500 mb-1">Mileage</div>
            <div className="text-lg font-mono">
              {kombi.vehicleInfo?.mileage_km?.toLocaleString() ?? "--"}
              <span className="text-xs text-zinc-500 ml-1">km</span>
            </div>
          </div>
          <div className="p-3 rounded-lg bg-zinc-900 border border-zinc-800">
            <div className="text-xs text-zinc-500 mb-1">Fuel Level</div>
            <div className="text-lg font-mono">
              {kombi.vehicleInfo?.fuel_level_percent?.toFixed(0) ?? "--"}
              <span className="text-xs text-zinc-500 ml-1">%</span>
            </div>
          </div>
          <div className="p-3 rounded-lg bg-zinc-900 border border-zinc-800">
            <div className="text-xs text-zinc-500 mb-1">Outside Temp</div>
            <div className="text-lg font-mono">
              {kombi.vehicleInfo?.outside_temp?.toFixed(0) ?? "--"}
              <span className="text-xs text-zinc-500 ml-1">C</span>
            </div>
          </div>
        </div>

        <button
          onClick={() => kombi.readVehicleInfo()}
          disabled={!canOperate || kombi.isLoading}
          className="w-full py-2 rounded-lg bg-zinc-800 hover:bg-zinc-700 disabled:opacity-50 disabled:cursor-not-allowed text-sm"
        >
          Read Vehicle Info
        </button>
      </div>

      {/* Service Intervals */}
      <div className="space-y-2">
        <h4 className="text-sm font-medium text-zinc-400 flex items-center gap-2">
          <Clock className="h-4 w-4" />
          Service Intervals
        </h4>
        <div className="space-y-2">
          {/* Oil Service */}
          <div className="p-3 rounded-lg bg-zinc-900 border border-zinc-800">
            <div className="flex items-center justify-between mb-2">
              <span className="text-sm font-medium">Oil Service</span>
              <button
                onClick={() => handleResetService("oil")}
                disabled={!canOperate || kombi.isLoading}
                className={`px-3 py-1 rounded text-xs ${
                  confirmReset === "oil"
                    ? "bg-red-600 text-white"
                    : "bg-zinc-700 hover:bg-zinc-600"
                } disabled:opacity-50`}
              >
                {confirmReset === "oil" ? "Confirm Reset?" : "Reset"}
              </button>
            </div>
            <div className="grid grid-cols-2 gap-2 text-sm">
              <div>
                <span className="text-zinc-500">Distance: </span>
                <span className={kombi.serviceInfo?.oil_service_km && kombi.serviceInfo.oil_service_km < 0 ? "text-red-400" : ""}>
                  {formatDistance(kombi.serviceInfo?.oil_service_km ?? null)}
                </span>
              </div>
              <div>
                <span className="text-zinc-500">Time: </span>
                <span className={kombi.serviceInfo?.oil_service_days && kombi.serviceInfo.oil_service_days < 0 ? "text-red-400" : ""}>
                  {formatDays(kombi.serviceInfo?.oil_service_days ?? null)}
                </span>
              </div>
            </div>
          </div>

          {/* Inspection */}
          <div className="p-3 rounded-lg bg-zinc-900 border border-zinc-800">
            <div className="flex items-center justify-between mb-2">
              <span className="text-sm font-medium">Inspection</span>
              <button
                onClick={() => handleResetService("inspection")}
                disabled={!canOperate || kombi.isLoading}
                className={`px-3 py-1 rounded text-xs ${
                  confirmReset === "inspection"
                    ? "bg-red-600 text-white"
                    : "bg-zinc-700 hover:bg-zinc-600"
                } disabled:opacity-50`}
              >
                {confirmReset === "inspection" ? "Confirm Reset?" : "Reset"}
              </button>
            </div>
            <div className="grid grid-cols-2 gap-2 text-sm">
              <div>
                <span className="text-zinc-500">Distance: </span>
                <span className={kombi.serviceInfo?.inspection_km && kombi.serviceInfo.inspection_km < 0 ? "text-red-400" : ""}>
                  {formatDistance(kombi.serviceInfo?.inspection_km ?? null)}
                </span>
              </div>
              <div>
                <span className="text-zinc-500">Time: </span>
                <span className={kombi.serviceInfo?.inspection_days && kombi.serviceInfo.inspection_days < 0 ? "text-red-400" : ""}>
                  {formatDays(kombi.serviceInfo?.inspection_days ?? null)}
                </span>
              </div>
            </div>
          </div>
        </div>

        <div className="flex gap-2">
          <button
            onClick={() => kombi.readServiceInfo()}
            disabled={!canOperate || kombi.isLoading}
            className="flex-1 py-2 rounded-lg bg-zinc-800 hover:bg-zinc-700 disabled:opacity-50 disabled:cursor-not-allowed text-sm"
          >
            Read Service Info
          </button>
          <button
            onClick={() => handleResetService("all")}
            disabled={!canOperate || kombi.isLoading}
            className={`px-4 py-2 rounded-lg text-sm ${
              confirmReset === "all"
                ? "bg-red-600 text-white"
                : "bg-amber-900/30 border border-amber-800 text-amber-300 hover:bg-amber-900/50"
            } disabled:opacity-50`}
          >
            {confirmReset === "all" ? "Confirm?" : "Reset All"}
          </button>
        </div>
      </div>

      {/* Gauge Test */}
      <div className="space-y-2">
        <h4 className="text-sm font-medium text-zinc-400 flex items-center gap-2">
          <Wrench className="h-4 w-4" />
          Functions
        </h4>
        <button
          onClick={() => kombi.gaugeTest()}
          disabled={!canOperate || kombi.isLoading}
          className="w-full py-2 rounded-lg bg-green-900/30 border border-green-800 hover:bg-green-900/50 disabled:opacity-50 disabled:cursor-not-allowed text-sm text-green-300"
        >
          Gauge Sweep Test
        </button>
      </div>

      {/* DTCs */}
      <div className="space-y-2">
        <h4 className="text-sm font-medium text-zinc-400 flex items-center gap-2">
          <AlertTriangle className="h-4 w-4" />
          Fault Codes ({kombi.dtcs.length})
        </h4>

        {kombi.dtcs.length > 0 && (
          <div className="space-y-1 max-h-40 overflow-y-auto">
            {kombi.dtcs.map((dtc, i) => (
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
          onClick={() => kombi.readDtcs()}
          disabled={!canOperate || kombi.isLoading}
          className="w-full py-2 rounded-lg bg-zinc-800 hover:bg-zinc-700 disabled:opacity-50 disabled:cursor-not-allowed text-sm"
        >
          Read DTCs
        </button>
      </div>
    </div>
  )
}
