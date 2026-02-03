import { useFRM } from "@/hooks/useFRM"
import { Lightbulb, RefreshCw, AlertTriangle, CheckCircle, XCircle } from "lucide-react"
import type { EcuInfo } from "@/hooks/useBMW"

interface FRMPanelProps {
  isConnected: boolean
  isInitialized: boolean
  selectedEcu: EcuInfo | null
}

interface LampIndicatorProps {
  name: string
  working: boolean | undefined
}

/** Lamp status indicator - moved outside component to prevent recreation on each render */
function LampIndicator({ name, working }: LampIndicatorProps) {
  const bgClass = working === undefined
    ? "bg-zinc-900 border-zinc-800"
    : working
    ? "bg-green-900/30 border-green-800"
    : "bg-red-900/30 border-red-800"

  return (
    <div className={`p-2 rounded-lg border flex items-center gap-2 ${bgClass}`}>
      {working === undefined ? (
        <div className="w-3 h-3 rounded-full bg-zinc-600" />
      ) : working ? (
        <CheckCircle className="w-3 h-3 text-green-500" />
      ) : (
        <XCircle className="w-3 h-3 text-red-500" />
      )}
      <span className="text-xs">{name}</span>
    </div>
  )
}

export function FRMPanel({ isConnected, isInitialized, selectedEcu }: FRMPanelProps) {
  const frm = useFRM()

  const canOperate = isConnected && isInitialized && selectedEcu?.id === "FRM"

  return (
    <div className="rounded-lg border border-zinc-800 bg-zinc-950 p-4 space-y-4">
      <div className="flex items-center justify-between">
        <h3 className="text-lg font-semibold flex items-center gap-2">
          <Lightbulb className="h-5 w-5 text-yellow-500" />
          FRM - Footwell Module (Lights)
        </h3>
        {frm.isLoading && (
          <RefreshCw className="h-4 w-4 animate-spin text-yellow-500" />
        )}
      </div>

      {frm.error && (
        <div className="p-3 rounded-lg bg-red-900/30 border border-red-800 text-red-300 text-sm">
          {frm.error}
        </div>
      )}

      {/* Lamp Status */}
      <div className="space-y-3">
        <h4 className="text-sm font-medium text-zinc-400">Lamp Status</h4>

        {/* Front Lights */}
        <div className="space-y-1">
          <div className="text-xs text-zinc-500 uppercase">Front</div>
          <div className="grid grid-cols-4 gap-2">
            <LampIndicator name="Low L" working={frm.lampStatus?.front_left_low} />
            <LampIndicator name="Low R" working={frm.lampStatus?.front_right_low} />
            <LampIndicator name="High L" working={frm.lampStatus?.front_left_high} />
            <LampIndicator name="High R" working={frm.lampStatus?.front_right_high} />
            <LampIndicator name="Fog L" working={frm.lampStatus?.fog_front_left} />
            <LampIndicator name="Fog R" working={frm.lampStatus?.fog_front_right} />
            <LampIndicator name="Turn L" working={frm.lampStatus?.turn_front_left} />
            <LampIndicator name="Turn R" working={frm.lampStatus?.turn_front_right} />
          </div>
        </div>

        {/* Rear Lights */}
        <div className="space-y-1">
          <div className="text-xs text-zinc-500 uppercase">Rear</div>
          <div className="grid grid-cols-4 gap-2">
            <LampIndicator name="Tail L" working={frm.lampStatus?.rear_left} />
            <LampIndicator name="Tail R" working={frm.lampStatus?.rear_right} />
            <LampIndicator name="Brake L" working={frm.lampStatus?.brake_left} />
            <LampIndicator name="Brake R" working={frm.lampStatus?.brake_right} />
            <LampIndicator name="Brake C" working={frm.lampStatus?.brake_center} />
            <LampIndicator name="Fog" working={frm.lampStatus?.fog_rear} />
            <LampIndicator name="Turn L" working={frm.lampStatus?.turn_rear_left} />
            <LampIndicator name="Turn R" working={frm.lampStatus?.turn_rear_right} />
            <LampIndicator name="Rev L" working={frm.lampStatus?.reverse_left} />
            <LampIndicator name="Rev R" working={frm.lampStatus?.reverse_right} />
          </div>
        </div>

        <button
          onClick={() => frm.readLampStatus()}
          disabled={!canOperate || frm.isLoading}
          className="w-full py-2 rounded-lg bg-zinc-800 hover:bg-zinc-700 disabled:opacity-50 disabled:cursor-not-allowed text-sm"
        >
          Read Lamp Status
        </button>
      </div>

      {/* Lamp Test */}
      <div className="space-y-2">
        <h4 className="text-sm font-medium text-zinc-400">Functions</h4>
        <button
          onClick={() => frm.lampTest()}
          disabled={!canOperate || frm.isLoading}
          className="w-full py-2 rounded-lg bg-yellow-900/30 border border-yellow-800 hover:bg-yellow-900/50 disabled:opacity-50 disabled:cursor-not-allowed text-sm text-yellow-300"
        >
          Lamp Test (Flash All Lights)
        </button>
        <p className="text-xs text-zinc-500">
          This will flash all exterior lights for visual inspection.
        </p>
      </div>

      {/* DTCs */}
      <div className="space-y-2">
        <h4 className="text-sm font-medium text-zinc-400 flex items-center gap-2">
          <AlertTriangle className="h-4 w-4" />
          Fault Codes ({frm.dtcs.length})
        </h4>

        {frm.dtcs.length > 0 && (
          <div className="space-y-1 max-h-40 overflow-y-auto">
            {frm.dtcs.map((dtc, i) => (
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
          onClick={() => frm.readDtcs()}
          disabled={!canOperate || frm.isLoading}
          className="w-full py-2 rounded-lg bg-zinc-800 hover:bg-zinc-700 disabled:opacity-50 disabled:cursor-not-allowed text-sm"
        >
          Read DTCs
        </button>
      </div>
    </div>
  )
}
