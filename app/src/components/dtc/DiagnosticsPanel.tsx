import { useEffect, useRef } from "react"
import { EcuSelector } from "@/components/connection/EcuSelector"
import { DTCList } from "@/components/dtc/DTCList"
import { AIInterpretation } from "@/components/dtc/AIInterpretation"
import type { useBMW } from "@/hooks/useBMW"
import { useAI } from "@/hooks/useAI"
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  Button,
  Alert,
  Badge,
  LedIndicator,
  ConfirmModal,
} from "@/components/ui"
import {
  Search,
  Trash2,
  XCircle,
  Zap,
  Cpu,
  AlertTriangle,
} from "lucide-react"
import { useState } from "react"

interface DiagnosticsPanelProps {
  isConnected: boolean
  bmw: ReturnType<typeof useBMW>
}

export function DiagnosticsPanel({ isConnected, bmw }: DiagnosticsPanelProps) {
  const {
    ecus,
    selectedEcu,
    dtcs,
    isInitialized,
    isLoading,
    error,
    protocol,
    getEcus,
    initKLine,
    readDtcs,
    clearDtcs,
    selectEcu,
  } = bmw

  const {
    analysis,
    isAnalyzing,
    error: aiError,
    analyzeDtcs,
    clearAnalysis,
  } = useAI()

  const [showClearConfirm, setShowClearConfirm] = useState(false)
  const testerPresentRef = useRef<ReturnType<typeof setInterval> | null>(null)

  // Load ECUs when connected
  useEffect(() => {
    if (isConnected && ecus.length === 0) {
      getEcus()
    }
  }, [isConnected, ecus.length, getEcus])

  // Start TesterPresent keepalive when initialized
  useEffect(() => {
    if (isInitialized && selectedEcu?.kline_address) {
      testerPresentRef.current = setInterval(() => {
        // Silent tester present
      }, 2000)
    }

    return () => {
      if (testerPresentRef.current) {
        clearInterval(testerPresentRef.current)
        testerPresentRef.current = null
      }
    }
  }, [isInitialized, selectedEcu])

  const handleReadDtcs = async () => {
    if (selectedEcu?.kline_address) {
      await readDtcs(selectedEcu.kline_address)
    }
  }

  const handleClearDtcs = async () => {
    if (selectedEcu?.kline_address) {
      await clearDtcs(selectedEcu.kline_address)
      clearAnalysis()
      setShowClearConfirm(false)
    }
  }

  const handleAnalyze = () => {
    analyzeDtcs(dtcs, {
      model: selectedEcu?.name || "E60",
    })
  }

  const handleInitialize = async () => {
    if (selectedEcu?.kline_address) {
      await initKLine(selectedEcu.kline_address)
    }
  }

  if (!isConnected) {
    return (
      <Card variant="default" padding="lg" className="animate-fade-in">
        <div className="flex flex-col items-center justify-center py-8 text-zinc-500">
          <div className="p-4 rounded-full bg-zinc-800/50 mb-4">
            <XCircle className="h-10 w-10 text-zinc-600" />
          </div>
          <p className="text-sm font-medium text-zinc-400">No Connection</p>
          <p className="text-xs text-zinc-600 mt-1">Connect to your K+DCAN cable first</p>
        </div>
      </Card>
    )
  }

  return (
    <div className="space-y-4">
      {/* ECU Selection */}
      <Card variant="elevated" padding="md" className="animate-fade-in">
        <CardHeader>
          <CardTitle>
            <Cpu className="h-5 w-5 text-blue-400" />
            ECU Selection
          </CardTitle>
          {selectedEcu && (
            <Badge
              variant={isInitialized ? "success" : "warning"}
              size="sm"
            >
              {isInitialized ? "Ready" : "Not Initialized"}
            </Badge>
          )}
        </CardHeader>
        <CardContent>
          <EcuSelector
            ecus={ecus}
            selectedEcu={selectedEcu}
            onSelect={selectEcu}
            onLoadEcus={getEcus}
            disabled={isLoading}
          />

          {/* Connection Status */}
          {selectedEcu && (
            <div className="mt-4 p-3 rounded-lg bg-zinc-900/50 border border-zinc-800 flex items-center justify-between">
              <div className="flex items-center gap-3">
                <LedIndicator
                  status={isInitialized ? "success" : "warning"}
                  size="md"
                />
                <div>
                  {isInitialized ? (
                    <div className="flex items-center gap-2">
                      <span className="text-sm font-medium text-emerald-400">
                        Connected
                      </span>
                      <Badge variant="outline" size="sm">
                        {protocol}
                      </Badge>
                    </div>
                  ) : (
                    <span className="text-sm font-medium text-amber-400">
                      Not initialized
                    </span>
                  )}
                  <p className="text-xs text-zinc-500">
                    {selectedEcu.name}
                  </p>
                </div>
              </div>

              {!isInitialized && (
                <Button
                  size="sm"
                  variant="bmw"
                  onClick={handleInitialize}
                  loading={isLoading}
                  leftIcon={<Zap className="h-4 w-4" />}
                >
                  Initialize
                </Button>
              )}
            </div>
          )}
        </CardContent>
      </Card>

      {/* Error Display */}
      {error && (
        <Alert variant="error" title="Error" onClose={() => {}}>
          {error}
        </Alert>
      )}

      {/* Diagnostics Actions */}
      {selectedEcu && isInitialized && (
        <Card variant="elevated" padding="md" className="animate-fade-in-up">
          <CardHeader>
            <CardTitle>
              <AlertTriangle className="h-5 w-5 text-amber-400" />
              Fault Codes
              {dtcs.length > 0 && (
                <Badge variant="error" size="sm" className="ml-2">
                  {dtcs.length} {dtcs.length === 1 ? "code" : "codes"}
                </Badge>
              )}
            </CardTitle>
            <div className="flex gap-2">
              <Button
                size="sm"
                variant="default"
                onClick={handleReadDtcs}
                loading={isLoading}
                leftIcon={<Search className="h-4 w-4" />}
              >
                Read
              </Button>
              <Button
                size="sm"
                variant="outline"
                onClick={() => setShowClearConfirm(true)}
                disabled={isLoading || dtcs.length === 0}
                leftIcon={<Trash2 className="h-4 w-4" />}
                className="border-red-900/50 text-red-400 hover:bg-red-950/50 hover:border-red-800"
              >
                Clear
              </Button>
            </div>
          </CardHeader>
          <CardContent>
            <DTCList dtcs={dtcs} isLoading={isLoading} />
          </CardContent>
        </Card>
      )}

      {/* AI Analysis Section */}
      {selectedEcu && isInitialized && (
        <AIInterpretation
          analysis={analysis}
          isAnalyzing={isAnalyzing}
          error={aiError}
          onAnalyze={handleAnalyze}
          disabled={isLoading}
          dtcCount={dtcs.length}
        />
      )}

      {/* Help text when ECU selected but not initialized */}
      {selectedEcu && !isInitialized && !isLoading && (
        <Alert variant="info" icon>
          <div className="space-y-1">
            <p>Click "Initialize" to start communication with the ECU</p>
            <p className="text-xs opacity-75">
              Make sure ignition is ON (engine can be off)
            </p>
          </div>
        </Alert>
      )}

      {/* Clear Confirmation Modal */}
      <ConfirmModal
        isOpen={showClearConfirm}
        onClose={() => setShowClearConfirm(false)}
        onConfirm={handleClearDtcs}
        title="Clear Fault Codes"
        message="Are you sure you want to clear all fault codes? This action cannot be undone."
        confirmText="Clear All"
        cancelText="Cancel"
        variant="danger"
        loading={isLoading}
      />
    </div>
  )
}
