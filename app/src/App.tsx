import { PortSelector } from "@/components/connection/PortSelector"
import { DiagnosticsPanel } from "@/components/dtc/DiagnosticsPanel"
import { LiveDataPanel } from "@/components/live-data/LiveDataPanel"
import { DPFPanel } from "@/components/dpf/DPFPanel"
import { DSCPanel, KOMBIPanel, FRMPanel, EGSPanel } from "@/components/ecu"
import { HistoryPanel } from "@/components/history/HistoryPanel"
import { useSerial } from "@/hooks/useSerial"
import { useBMW } from "@/hooks/useBMW"
import { useWebSocket } from "@/hooks/useWebSocket"
import {
  Card,
  CardContent,
  Badge,
  LedIndicator,
  Tabs,
  TabsList,
  TabsTrigger,
  TabsContent,
  ToastProvider,
} from "@/components/ui"
import {
  Car,
  Activity,
  AlertTriangle,
  Wifi,
  WifiOff,
  Filter,
  CircleDot,
  Gauge,
  Lightbulb,
  Settings,
  History,
  Cpu,
  Cable,
} from "lucide-react"
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
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  // Send DTCs when they change
  useEffect(() => {
    if (bmw.dtcs.length > 0) {
      ws.sendDtcs(bmw.dtcs)
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [bmw.dtcs])

  // Send ECU status when connection changes
  useEffect(() => {
    ws.sendEcuStatus(
      bmw.isInitialized,
      bmw.selectedEcu?.name,
      bmw.protocol || undefined
    )
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [bmw.isInitialized, bmw.selectedEcu, bmw.protocol])

  // Determine connection status for LED
  const connectionStatus = isConnected
    ? bmw.isInitialized
      ? "success"
      : "warning"
    : "off"

  return (
    <ToastProvider>
      <div className="min-h-screen bg-gradient-to-b from-zinc-950 via-zinc-900 to-zinc-950 text-white">
        {/* Header */}
        <header className="sticky top-0 z-[var(--z-sticky)] glass border-b border-zinc-800/50">
          <div className="container mx-auto px-4 sm:px-6">
            <div className="flex items-center justify-between h-16">
              {/* Logo & Title */}
              <div className="flex items-center gap-3">
                <div className="relative">
                  <div className="absolute inset-0 bg-blue-500/20 blur-xl rounded-full" />
                  <div className={`relative p-2.5 rounded-xl bg-gradient-to-br from-blue-600 to-blue-700 shadow-lg shadow-blue-600/25 ${
                    isConnected ? "animate-pulse-glow" : ""
                  }`}>
                    <Car className="h-6 w-6 text-white" />
                  </div>
                </div>
                <div>
                  <h1 className="text-lg sm:text-xl font-bold bg-gradient-to-r from-white to-zinc-400 bg-clip-text text-transparent">
                    BMW Diagnostic Tool
                  </h1>
                  <p className="text-xs text-zinc-500 hidden sm:block">K+DCAN Interface for E-Series</p>
                </div>
              </div>

              {/* Status Indicators */}
              <div className="flex items-center gap-3">
                {/* Connection Status */}
                <div className="hidden sm:flex items-center gap-2 px-3 py-1.5 rounded-lg bg-zinc-800/50 border border-zinc-700/50">
                  <LedIndicator
                    status={connectionStatus}
                    size="sm"
                  />
                  <span className="text-xs text-zinc-400">
                    {isConnected ? (bmw.isInitialized ? "Ready" : "Connected") : "Disconnected"}
                  </span>
                </div>

                {/* WebSocket Status */}
                {ws.isConnected ? (
                  <Badge variant="success" size="sm" dot dotColor="success">
                    <Wifi className="h-3 w-3" />
                    <span className="hidden sm:inline">Dashboard:</span> {ws.dashboardCount}
                  </Badge>
                ) : (
                  <Badge variant="outline" size="sm">
                    <WifiOff className="h-3 w-3" />
                    <span className="hidden sm:inline">Offline</span>
                  </Badge>
                )}
              </div>
            </div>
          </div>
        </header>

        {/* Main Content */}
        <main className="container mx-auto px-4 sm:px-6 py-6 pb-24">
          <div className="max-w-3xl mx-auto space-y-6">
            {/* Connection Section */}
            <section className="animate-fade-in-up">
              <div className="flex items-center gap-3 mb-4">
                <div className="flex items-center justify-center w-8 h-8 rounded-lg bg-blue-600/20 border border-blue-600/30">
                  <Cable className="h-4 w-4 text-blue-400" />
                </div>
                <div>
                  <h2 className="font-semibold text-white">Connection</h2>
                  <p className="text-xs text-zinc-500">Connect your K+DCAN interface</p>
                </div>
              </div>
              <Card variant="elevated" padding="lg">
                <CardContent>
                  <PortSelector />
                </CardContent>
              </Card>
            </section>

            {/* Tab Navigation */}
            <section className="animate-fade-in-up" style={{ animationDelay: "100ms" }}>
              <div className="flex items-center gap-3 mb-4">
                <div className="flex items-center justify-center w-8 h-8 rounded-lg bg-purple-600/20 border border-purple-600/30">
                  <Cpu className="h-4 w-4 text-purple-400" />
                </div>
                <div>
                  <h2 className="font-semibold text-white">Diagnostics</h2>
                  <p className="text-xs text-zinc-500">
                    {bmw.selectedEcu ? `ECU: ${bmw.selectedEcu.name}` : "Select an ECU to begin"}
                  </p>
                </div>
              </div>

              <Tabs defaultValue="diagnostics" value={activeTab} onChange={(v) => setActiveTab(v as Tab)}>
                <TabsList className="mb-4">
                  <TabsTrigger
                    value="diagnostics"
                    icon={<AlertTriangle className="h-4 w-4" />}
                  >
                    <span className="hidden sm:inline">Diagnostics</span>
                    <span className="sm:hidden">DTCs</span>
                  </TabsTrigger>
                  <TabsTrigger
                    value="live-data"
                    icon={<Activity className="h-4 w-4" />}
                  >
                    <span className="hidden sm:inline">Live Data</span>
                    <span className="sm:hidden">Live</span>
                  </TabsTrigger>
                  <TabsTrigger
                    value="dpf"
                    icon={<Filter className="h-4 w-4" />}
                  >
                    DPF
                  </TabsTrigger>
                  {bmw.selectedEcu && ["DSC", "KOMBI", "FRM", "EGS"].includes(bmw.selectedEcu.id) && (
                    <TabsTrigger
                      value="ecu-specific"
                      icon={
                        bmw.selectedEcu.id === "DSC" ? <CircleDot className="h-4 w-4" /> :
                        bmw.selectedEcu.id === "KOMBI" ? <Gauge className="h-4 w-4" /> :
                        bmw.selectedEcu.id === "FRM" ? <Lightbulb className="h-4 w-4" /> :
                        <Settings className="h-4 w-4" />
                      }
                    >
                      {bmw.selectedEcu.id}
                    </TabsTrigger>
                  )}
                  <TabsTrigger
                    value="history"
                    icon={<History className="h-4 w-4" />}
                  >
                    <span className="hidden sm:inline">History</span>
                    <span className="sm:hidden">Hist</span>
                  </TabsTrigger>
                </TabsList>

                <TabsContent value="diagnostics">
                  <DiagnosticsPanel isConnected={isConnected} bmw={bmw} />
                </TabsContent>

                <TabsContent value="live-data">
                  <LiveDataPanel
                    isConnected={isConnected}
                    isInitialized={bmw.isInitialized}
                    selectedEcu={bmw.selectedEcu}
                    onLiveDataUpdate={ws.sendLiveData}
                  />
                </TabsContent>

                <TabsContent value="dpf">
                  <DPFPanel
                    isConnected={isConnected}
                    isInitialized={bmw.isInitialized}
                    selectedEcu={bmw.selectedEcu}
                  />
                </TabsContent>

                <TabsContent value="ecu-specific">
                  {bmw.selectedEcu?.id === "DSC" && (
                    <DSCPanel
                      isConnected={isConnected}
                      isInitialized={bmw.isInitialized}
                      selectedEcu={bmw.selectedEcu}
                    />
                  )}
                  {bmw.selectedEcu?.id === "KOMBI" && (
                    <KOMBIPanel
                      isConnected={isConnected}
                      isInitialized={bmw.isInitialized}
                      selectedEcu={bmw.selectedEcu}
                    />
                  )}
                  {bmw.selectedEcu?.id === "FRM" && (
                    <FRMPanel
                      isConnected={isConnected}
                      isInitialized={bmw.isInitialized}
                      selectedEcu={bmw.selectedEcu}
                    />
                  )}
                  {bmw.selectedEcu?.id === "EGS" && (
                    <EGSPanel
                      isConnected={isConnected}
                      isInitialized={bmw.isInitialized}
                      selectedEcu={bmw.selectedEcu}
                    />
                  )}
                </TabsContent>

                <TabsContent value="history">
                  <HistoryPanel />
                </TabsContent>
              </Tabs>
            </section>
          </div>
        </main>

        {/* Footer */}
        <footer className="fixed bottom-0 left-0 right-0 glass border-t border-zinc-800/50">
          <div className="container mx-auto px-4 sm:px-6">
            <div className="flex flex-col sm:flex-row justify-between items-center gap-2 py-3">
              <div className="flex items-center gap-4 text-xs text-zinc-500">
                <span className="flex items-center gap-2">
                  <Car className="h-3.5 w-3.5" />
                  BMW E60/E61/E63/E64 (2003-2010)
                </span>
                {bmw.protocol && (
                  <Badge variant="outline" size="sm">
                    {bmw.protocol}
                  </Badge>
                )}
              </div>
              <div className="flex items-center gap-3 text-xs text-zinc-500">
                {bmw.selectedEcu && (
                  <span className="flex items-center gap-1.5">
                    <LedIndicator status={bmw.isInitialized ? "success" : "off"} size="sm" />
                    {bmw.selectedEcu.name}
                  </span>
                )}
                <span className="text-zinc-600">|</span>
                <span>v0.1.0</span>
              </div>
            </div>
          </div>
        </footer>
      </div>
    </ToastProvider>
  )
}

export default App
