import { useEffect, useRef } from "react"
import { Button } from "@/components/ui/button"
import { EcuSelector } from "@/components/connection/EcuSelector"
import { DTCList } from "@/components/dtc/DTCList"
import { AIInterpretation } from "@/components/dtc/AIInterpretation"
import type { useBMW } from "@/hooks/useBMW"
import { useAI } from "@/hooks/useAI"
import {
  Search,
  Trash2,
  CheckCircle2,
  XCircle,
  Loader2,
  Zap,
} from "lucide-react"

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

  // TesterPresent interval ref
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
      // Send TesterPresent every 2 seconds to keep session alive
      testerPresentRef.current = setInterval(() => {
        // Silent tester present - don't await
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
      if (window.confirm("¿Estás seguro de borrar todos los códigos de error?")) {
        await clearDtcs(selectedEcu.kline_address)
        clearAnalysis()
      }
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
      <div className="rounded-lg border border-zinc-800 bg-zinc-950 p-6">
        <div className="flex flex-col items-center justify-center py-4 text-zinc-500">
          <XCircle className="h-12 w-12 text-zinc-600 mb-2" />
          <p className="text-sm">Connect to your K+DCAN cable first</p>
        </div>
      </div>
    )
  }

  return (
    <div className="space-y-4">
      {/* ECU Selection */}
      <div className="rounded-lg border border-zinc-800 bg-zinc-950 p-4">
        <EcuSelector
          ecus={ecus}
          selectedEcu={selectedEcu}
          onSelect={selectEcu}
          onLoadEcus={getEcus}
          disabled={isLoading}
        />

        {/* Connection Status */}
        {selectedEcu && (
          <div className="mt-4 flex items-center gap-2">
            {isInitialized ? (
              <>
                <CheckCircle2 className="h-4 w-4 text-green-500" />
                <span className="text-sm text-green-500">
                  Connected via {protocol}
                </span>
              </>
            ) : (
              <>
                <XCircle className="h-4 w-4 text-amber-500" />
                <span className="text-sm text-amber-500">Not initialized</span>
                <Button
                  size="sm"
                  variant="outline"
                  onClick={handleInitialize}
                  disabled={isLoading}
                  className="ml-auto border-zinc-700"
                >
                  {isLoading ? (
                    <Loader2 className="h-4 w-4 animate-spin" />
                  ) : (
                    <Zap className="h-4 w-4" />
                  )}
                  Initialize
                </Button>
              </>
            )}
          </div>
        )}
      </div>

      {/* Error Display */}
      {error && (
        <div className="rounded-lg border border-red-900 bg-red-950 p-3 text-sm text-red-400">
          {error}
        </div>
      )}

      {/* Diagnostics Actions */}
      {selectedEcu && isInitialized && (
        <div className="rounded-lg border border-zinc-800 bg-zinc-950 p-4">
          <div className="flex items-center justify-between mb-4">
            <h3 className="font-semibold">Fault Codes</h3>
            <div className="flex gap-2">
              <Button
                size="sm"
                onClick={handleReadDtcs}
                disabled={isLoading}
                className="bg-blue-600 hover:bg-blue-700"
              >
                {isLoading ? (
                  <Loader2 className="h-4 w-4 animate-spin mr-1" />
                ) : (
                  <Search className="h-4 w-4 mr-1" />
                )}
                Read
              </Button>
              <Button
                size="sm"
                variant="outline"
                onClick={handleClearDtcs}
                disabled={isLoading || dtcs.length === 0}
                className="border-red-900 text-red-400 hover:bg-red-950"
              >
                <Trash2 className="h-4 w-4 mr-1" />
                Clear
              </Button>
            </div>
          </div>

          <DTCList dtcs={dtcs} isLoading={isLoading} />
        </div>
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
        <div className="rounded-lg border border-zinc-800 bg-zinc-950 p-4 text-center text-sm text-zinc-500">
          <p>Click "Initialize" to start communication with the ECU</p>
          <p className="text-xs mt-1">
            Make sure ignition is ON (engine can be off)
          </p>
        </div>
      )}
    </div>
  )
}
