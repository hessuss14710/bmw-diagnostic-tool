import { PortSelector } from "@/components/connection/PortSelector"
import { DiagnosticsPanel } from "@/components/dtc/DiagnosticsPanel"
import { LiveDataPanel } from "@/components/live-data/LiveDataPanel"
import { DPFPanel } from "@/components/dpf/DPFPanel"
import { DSCPanel, KOMBIPanel, FRMPanel, EGSPanel } from "@/components/ecu"
import { HistoryPanel } from "@/components/history/HistoryPanel"
import { useSerial } from "@/hooks/useSerial"
import { useBMW } from "@/hooks/useBMW"
import { useWebSocket } from "@/hooks/useWebSocket"
import { Car, Activity, AlertTriangle, Wifi, WifiOff, Filter, CircleDot, Gauge, Lightbulb, Settings, History } from "lucide-react"
import { useState, useEffect } from "react"

type Tab = "diagnostics" | "live-data" | "dpf" | "ecu-specific" | "history"

function App() {
  const { isConnected } = useSerial()
  const bmw = useBMW()
  const ws = useWebSocket()
  const [activeTab, setActiveTab] = useState<Tab>("diagnostics")

  // Connect to WebSocket server on mount
  useEffect(() => {
    ws.connect({ model: "BMW E60" })
  }, [])

  // Send DTCs when they change
  useEffect(() => {
    if (bmw.dtcs.length > 0) {
      ws.sendDtcs(bmw.dtcs)
    }
  }, [bmw.dtcs])

  // Send ECU status when connection changes
  useEffect(() => {
    ws.sendEcuStatus(
      bmw.isInitialized,
      bmw.selectedEcu?.name,
      bmw.protocol || undefined
    )
  }, [bmw.isInitialized, bmw.selectedEcu, bmw.protocol])

  return (
    <div className="min-h-screen bg-zinc-900 text-white">
      {/* Header */}
      <header className="border-b border-zinc-800 px-6 py-4">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <Car className="h-8 w-8 text-blue-500" />
            <div>
              <h1 className="text-xl font-bold">BMW Diagnostic Tool</h1>
              <p className="text-xs text-zinc-500">K+DCAN Interface for E60</p>
            </div>
          </div>
          {/* WebSocket Status */}
          <div className="flex items-center gap-2">
            {ws.isConnected ? (
              <div className="flex items-center gap-2 px-3 py-1.5 rounded-full bg-green-900/30 text-green-400 text-xs">
                <Wifi className="h-3.5 w-3.5" />
                <span>Dashboard: {ws.dashboardCount}</span>
              </div>
            ) : (
              <div className="flex items-center gap-2 px-3 py-1.5 rounded-full bg-zinc-800 text-zinc-500 text-xs">
                <WifiOff className="h-3.5 w-3.5" />
                <span>Offline</span>
              </div>
            )}
          </div>
        </div>
      </header>

      {/* Main Content */}
      <main className="container mx-auto px-6 py-6 pb-20">
        <div className="max-w-2xl mx-auto space-y-6">
          {/* Connection Section */}
          <section>
            <h2 className="text-lg font-semibold mb-3 flex items-center gap-2">
              <span className="w-6 h-6 rounded-full bg-blue-600 text-white text-xs flex items-center justify-center">
                1
              </span>
              Connection
            </h2>
            <div className="rounded-lg border border-zinc-800 bg-zinc-950 p-4">
              <PortSelector />
            </div>
          </section>

          {/* Tab Navigation */}
          <section>
            <div className="flex flex-wrap gap-2 mb-3">
              <button
                onClick={() => setActiveTab("diagnostics")}
                className={`flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-medium transition-colors ${
                  activeTab === "diagnostics"
                    ? "bg-blue-600 text-white"
                    : "bg-zinc-800 text-zinc-400 hover:bg-zinc-700"
                }`}
              >
                <AlertTriangle className="h-4 w-4" />
                Diagnostics
              </button>
              <button
                onClick={() => setActiveTab("live-data")}
                className={`flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-medium transition-colors ${
                  activeTab === "live-data"
                    ? "bg-blue-600 text-white"
                    : "bg-zinc-800 text-zinc-400 hover:bg-zinc-700"
                }`}
              >
                <Activity className="h-4 w-4" />
                Live Data
              </button>
              <button
                onClick={() => setActiveTab("dpf")}
                className={`flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-medium transition-colors ${
                  activeTab === "dpf"
                    ? "bg-amber-600 text-white"
                    : "bg-zinc-800 text-zinc-400 hover:bg-zinc-700"
                }`}
              >
                <Filter className="h-4 w-4" />
                DPF
              </button>
              {/* ECU-Specific Tab - shown when non-DDE ECU is selected */}
              {bmw.selectedEcu && ["DSC", "KOMBI", "FRM", "EGS"].includes(bmw.selectedEcu.id) && (
                <button
                  onClick={() => setActiveTab("ecu-specific")}
                  className={`flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-medium transition-colors ${
                    activeTab === "ecu-specific"
                      ? "bg-purple-600 text-white"
                      : "bg-zinc-800 text-zinc-400 hover:bg-zinc-700"
                  }`}
                >
                  {bmw.selectedEcu.id === "DSC" && <CircleDot className="h-4 w-4" />}
                  {bmw.selectedEcu.id === "KOMBI" && <Gauge className="h-4 w-4" />}
                  {bmw.selectedEcu.id === "FRM" && <Lightbulb className="h-4 w-4" />}
                  {bmw.selectedEcu.id === "EGS" && <Settings className="h-4 w-4" />}
                  {bmw.selectedEcu.id}
                </button>
              )}
              {/* History Tab */}
              <button
                onClick={() => setActiveTab("history")}
                className={`flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-medium transition-colors ${
                  activeTab === "history"
                    ? "bg-cyan-600 text-white"
                    : "bg-zinc-800 text-zinc-400 hover:bg-zinc-700"
                }`}
              >
                <History className="h-4 w-4" />
                History
              </button>
            </div>

            {activeTab === "diagnostics" && (
              <DiagnosticsPanel isConnected={isConnected} bmw={bmw} />
            )}

            {activeTab === "live-data" && (
              <LiveDataPanel
                isConnected={isConnected}
                isInitialized={bmw.isInitialized}
                selectedEcu={bmw.selectedEcu}
                onLiveDataUpdate={ws.sendLiveData}
              />
            )}

            {activeTab === "dpf" && (
              <DPFPanel
                isConnected={isConnected}
                isInitialized={bmw.isInitialized}
                selectedEcu={bmw.selectedEcu}
              />
            )}

            {activeTab === "ecu-specific" && bmw.selectedEcu && (
              <>
                {bmw.selectedEcu.id === "DSC" && (
                  <DSCPanel
                    isConnected={isConnected}
                    isInitialized={bmw.isInitialized}
                    selectedEcu={bmw.selectedEcu}
                  />
                )}
                {bmw.selectedEcu.id === "KOMBI" && (
                  <KOMBIPanel
                    isConnected={isConnected}
                    isInitialized={bmw.isInitialized}
                    selectedEcu={bmw.selectedEcu}
                  />
                )}
                {bmw.selectedEcu.id === "FRM" && (
                  <FRMPanel
                    isConnected={isConnected}
                    isInitialized={bmw.isInitialized}
                    selectedEcu={bmw.selectedEcu}
                  />
                )}
                {bmw.selectedEcu.id === "EGS" && (
                  <EGSPanel
                    isConnected={isConnected}
                    isInitialized={bmw.isInitialized}
                    selectedEcu={bmw.selectedEcu}
                  />
                )}
              </>
            )}

            {activeTab === "history" && (
              <HistoryPanel />
            )}
          </section>
        </div>
      </main>

      {/* Footer */}
      <footer className="fixed bottom-0 left-0 right-0 border-t border-zinc-800 px-6 py-3 bg-zinc-900">
        <div className="flex justify-between items-center text-xs text-zinc-500">
          <span>BMW E60 (2003-2010) Compatible</span>
          <span>v0.1.0</span>
        </div>
      </footer>
    </div>
  )
}

export default App
