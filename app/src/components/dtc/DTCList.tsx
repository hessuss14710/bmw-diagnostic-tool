import { type Dtc } from "@/hooks/useBMW"
import { AlertTriangle, AlertCircle, CheckCircle2, Info } from "lucide-react"

interface DTCListProps {
  dtcs: Dtc[]
  isLoading?: boolean
}

export function DTCList({ dtcs, isLoading = false }: DTCListProps) {
  if (isLoading) {
    return (
      <div className="flex items-center justify-center py-8 text-zinc-500">
        <div className="animate-spin h-5 w-5 border-2 border-zinc-500 border-t-transparent rounded-full mr-2" />
        Reading DTCs...
      </div>
    )
  }

  if (dtcs.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center py-8 text-zinc-500">
        <CheckCircle2 className="h-12 w-12 text-green-500 mb-2" />
        <p className="text-sm">No fault codes found</p>
      </div>
    )
  }

  return (
    <div className="space-y-2">
      <div className="flex items-center justify-between text-sm text-zinc-400 mb-3">
        <span>{dtcs.length} fault code{dtcs.length !== 1 ? "s" : ""} found</span>
      </div>

      <div className="space-y-2 max-h-64 overflow-y-auto">
        {dtcs.map((dtc, index) => (
          <DTCCard key={`${dtc.code}-${index}`} dtc={dtc} />
        ))}
      </div>
    </div>
  )
}

interface DTCCardProps {
  dtc: Dtc
}

function DTCCard({ dtc }: DTCCardProps) {
  const getStatusIcon = () => {
    if (dtc.status.confirmed) {
      return <AlertTriangle className="h-5 w-5 text-red-500" />
    }
    if (dtc.status.pending) {
      return <AlertCircle className="h-5 w-5 text-amber-500" />
    }
    return <Info className="h-5 w-5 text-blue-500" />
  }

  const getStatusText = () => {
    const statuses: string[] = []
    if (dtc.status.confirmed) statuses.push("Confirmed")
    if (dtc.status.pending) statuses.push("Pending")
    if (dtc.status.test_failed) statuses.push("Active")
    if (dtc.status.warning_indicator_requested) statuses.push("MIL On")
    return statuses.length > 0 ? statuses.join(", ") : "Stored"
  }

  const getCodeCategory = (code: string) => {
    const prefix = code.charAt(0)
    switch (prefix) {
      case "P":
        return { name: "Powertrain", color: "text-red-400" }
      case "C":
        return { name: "Chassis", color: "text-yellow-400" }
      case "B":
        return { name: "Body", color: "text-blue-400" }
      case "U":
        return { name: "Network", color: "text-purple-400" }
      default:
        return { name: "Unknown", color: "text-zinc-400" }
    }
  }

  const category = getCodeCategory(dtc.code)

  return (
    <div className="rounded-lg border border-zinc-800 bg-zinc-950 p-3 hover:border-zinc-700 transition-colors">
      <div className="flex items-start gap-3">
        {getStatusIcon()}

        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2">
            <span className="font-mono font-bold text-white">{dtc.code}</span>
            <span className={`text-xs ${category.color}`}>{category.name}</span>
          </div>

          <p className="text-sm text-zinc-400 mt-1">
            {dtc.description || "No description available"}
          </p>

          <div className="flex items-center gap-2 mt-2">
            <span
              className={`text-xs px-2 py-0.5 rounded ${
                dtc.status.confirmed
                  ? "bg-red-900 text-red-300"
                  : dtc.status.pending
                  ? "bg-amber-900 text-amber-300"
                  : "bg-zinc-800 text-zinc-300"
              }`}
            >
              {getStatusText()}
            </span>

            <span className="text-xs text-zinc-600 font-mono">
              [{dtc.raw_bytes.map((b) => b.toString(16).padStart(2, "0")).join(" ")}]
            </span>
          </div>
        </div>
      </div>
    </div>
  )
}
