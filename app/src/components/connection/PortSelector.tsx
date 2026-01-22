import { useEffect, useState } from "react"
import { Button } from "@/components/ui/button"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import { useSerial, type PortInfo } from "@/hooks/useSerial"
import {
  RefreshCw,
  Plug,
  Unplug,
  Usb,
  AlertCircle,
  CheckCircle2,
  Loader2,
} from "lucide-react"

export function PortSelector() {
  const {
    ports,
    status,
    isLoading,
    error,
    isConnected,
    listPorts,
    connect,
    disconnect,
  } = useSerial()

  const [selectedPort, setSelectedPort] = useState<string>("")

  // Load ports on mount
  useEffect(() => {
    listPorts()
  }, [listPorts])

  // Handle connect/disconnect
  const handleToggleConnection = async () => {
    if (isConnected) {
      await disconnect()
    } else if (selectedPort) {
      await connect(selectedPort, 10400) // K-Line default baud
    }
  }

  // Format port display name
  const formatPortName = (port: PortInfo) => {
    const parts = [port.name]
    if (port.product) {
      parts.push(`- ${port.product}`)
    }
    if (port.is_ftdi) {
      parts.push("(FTDI)")
    }
    return parts.join(" ")
  }

  // Separate FTDI and other ports
  const ftdiPorts = ports.filter((p) => p.is_ftdi)
  const otherPorts = ports.filter((p) => !p.is_ftdi)

  return (
    <div className="space-y-4">
      {/* Port Selection */}
      <div className="flex gap-2">
        <Select
          value={selectedPort}
          onValueChange={setSelectedPort}
          disabled={isConnected || isLoading}
        >
          <SelectTrigger className="flex-1">
            <SelectValue placeholder="Select port..." />
          </SelectTrigger>
          <SelectContent>
            {ftdiPorts.length > 0 && (
              <>
                <div className="px-2 py-1.5 text-xs font-semibold text-blue-400">
                  K+DCAN Adapters (FTDI)
                </div>
                {ftdiPorts.map((port) => (
                  <SelectItem key={port.name} value={port.name}>
                    <div className="flex items-center gap-2">
                      <Usb className="h-3 w-3 text-blue-400" />
                      {formatPortName(port)}
                    </div>
                  </SelectItem>
                ))}
              </>
            )}
            {otherPorts.length > 0 && (
              <>
                {ftdiPorts.length > 0 && (
                  <div className="my-1 h-px bg-zinc-700" />
                )}
                <div className="px-2 py-1.5 text-xs font-semibold text-zinc-400">
                  Other Ports
                </div>
                {otherPorts.map((port) => (
                  <SelectItem key={port.name} value={port.name}>
                    {formatPortName(port)}
                  </SelectItem>
                ))}
              </>
            )}
            {ports.length === 0 && (
              <div className="px-2 py-4 text-center text-sm text-zinc-500">
                No ports found
              </div>
            )}
          </SelectContent>
        </Select>

        {/* Refresh button */}
        <Button
          variant="outline"
          size="icon"
          onClick={() => listPorts()}
          disabled={isLoading || isConnected}
          className="border-zinc-700"
        >
          <RefreshCw
            className={`h-4 w-4 ${isLoading ? "animate-spin" : ""}`}
          />
        </Button>
      </div>

      {/* Connect/Disconnect button */}
      <Button
        className={`w-full ${
          isConnected
            ? "bg-red-600 hover:bg-red-700"
            : "bg-blue-600 hover:bg-blue-700"
        }`}
        onClick={handleToggleConnection}
        disabled={isLoading || (!isConnected && !selectedPort)}
      >
        {isLoading ? (
          <>
            <Loader2 className="h-4 w-4 animate-spin" />
            {status.state === "connecting" ? "Connecting..." : "Loading..."}
          </>
        ) : isConnected ? (
          <>
            <Unplug className="h-4 w-4" />
            Disconnect
          </>
        ) : (
          <>
            <Plug className="h-4 w-4" />
            Connect
          </>
        )}
      </Button>

      {/* Status display */}
      <div className="rounded-md border border-zinc-800 bg-zinc-950 p-3">
        <div className="flex items-center gap-2">
          {status.state === "connected" ? (
            <>
              <CheckCircle2 className="h-4 w-4 text-green-500" />
              <span className="text-sm text-green-500">
                Connected to {status.port}
              </span>
            </>
          ) : status.state === "error" ? (
            <>
              <AlertCircle className="h-4 w-4 text-red-500" />
              <span className="text-sm text-red-500">{status.error}</span>
            </>
          ) : (
            <>
              <AlertCircle className="h-4 w-4 text-amber-500" />
              <span className="text-sm text-zinc-400">Not connected</span>
            </>
          )}
        </div>

        {/* Port details when connected */}
        {isConnected && selectedPort && (
          <div className="mt-2 text-xs text-zinc-500">
            {(() => {
              const port = ports.find((p) => p.name === selectedPort)
              if (!port) return null
              return (
                <div className="space-y-0.5">
                  {port.manufacturer && (
                    <div>Manufacturer: {port.manufacturer}</div>
                  )}
                  {port.product && <div>Product: {port.product}</div>}
                  {port.vid && port.pid && (
                    <div>
                      VID:PID: {port.vid.toString(16).toUpperCase()}:
                      {port.pid.toString(16).toUpperCase()}
                    </div>
                  )}
                </div>
              )
            })()}
          </div>
        )}
      </div>

      {/* Error display */}
      {error && !status.error && (
        <div className="flex items-center gap-2 rounded-md border border-red-900 bg-red-950 p-3 text-sm text-red-400">
          <AlertCircle className="h-4 w-4" />
          {error}
        </div>
      )}
    </div>
  )
}
