import { useEffect } from "react"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import { type EcuInfo } from "@/hooks/useBMW"
import { Cpu, Zap, Radio } from "lucide-react"

interface EcuSelectorProps {
  ecus: EcuInfo[]
  selectedEcu: EcuInfo | null
  onSelect: (ecu: EcuInfo) => void
  onLoadEcus: () => Promise<EcuInfo[]>
  disabled?: boolean
}

export function EcuSelector({
  ecus,
  selectedEcu,
  onSelect,
  onLoadEcus,
  disabled = false,
}: EcuSelectorProps) {
  // Load ECUs on mount
  useEffect(() => {
    if (ecus.length === 0) {
      onLoadEcus()
    }
  }, [ecus.length, onLoadEcus])

  const handleSelect = (ecuId: string) => {
    const ecu = ecus.find((e) => e.id === ecuId)
    if (ecu) {
      onSelect(ecu)
    }
  }

  const getProtocolIcon = (protocol: string) => {
    switch (protocol) {
      case "KLine":
        return <Radio className="h-3 w-3 text-yellow-500" />
      case "DCan":
        return <Zap className="h-3 w-3 text-blue-500" />
      case "Both":
        return <Cpu className="h-3 w-3 text-green-500" />
      default:
        return null
    }
  }

  const getProtocolBadge = (protocol: string) => {
    const colors = {
      KLine: "bg-yellow-900 text-yellow-300",
      DCan: "bg-blue-900 text-blue-300",
      Both: "bg-green-900 text-green-300",
    }
    return colors[protocol as keyof typeof colors] || "bg-zinc-800 text-zinc-300"
  }

  return (
    <div className="space-y-2">
      <label className="text-sm font-medium text-zinc-400">Select ECU</label>
      <Select
        value={selectedEcu?.id || ""}
        onValueChange={handleSelect}
        disabled={disabled}
      >
        <SelectTrigger>
          <SelectValue placeholder="Choose ECU to diagnose..." />
        </SelectTrigger>
        <SelectContent>
          {ecus.map((ecu) => (
            <SelectItem key={ecu.id} value={ecu.id}>
              <div className="flex items-center gap-2">
                {getProtocolIcon(ecu.protocol)}
                <span className="font-medium">{ecu.id}</span>
                <span className="text-zinc-500">-</span>
                <span className="text-zinc-400">{ecu.name}</span>
                <span
                  className={`ml-auto text-xs px-1.5 py-0.5 rounded ${getProtocolBadge(
                    ecu.protocol
                  )}`}
                >
                  {ecu.protocol === "Both" ? "K+D" : ecu.protocol}
                </span>
              </div>
            </SelectItem>
          ))}
        </SelectContent>
      </Select>

      {selectedEcu && (
        <div className="text-xs text-zinc-500 mt-1">
          {selectedEcu.description}
          {selectedEcu.kline_address !== null && (
            <span className="ml-2">
              K-Line: 0x{selectedEcu.kline_address.toString(16).toUpperCase()}
            </span>
          )}
          {selectedEcu.can_tx_id !== null && (
            <span className="ml-2">
              CAN: 0x{selectedEcu.can_tx_id.toString(16).toUpperCase()}
            </span>
          )}
        </div>
      )}
    </div>
  )
}
