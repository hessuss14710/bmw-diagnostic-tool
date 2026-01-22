import { Button } from "@/components/ui/button"
import type { FullAnalysis } from "@/hooks/useAI"
import {
  Brain,
  AlertTriangle,
  CheckCircle2,
  AlertCircle,
  Loader2,
  Wrench,
  ListOrdered,
  FileText,
  ShieldAlert,
} from "lucide-react"

interface AIInterpretationProps {
  analysis: FullAnalysis | null
  isAnalyzing: boolean
  error: string | null
  onAnalyze: () => void
  disabled: boolean
  dtcCount: number
}

export function AIInterpretation({
  analysis,
  isAnalyzing,
  error,
  onAnalyze,
  disabled,
  dtcCount,
}: AIInterpretationProps) {
  const getSeverityColor = (severity: string) => {
    switch (severity) {
      case "critical":
        return "text-red-500 bg-red-950 border-red-800"
      case "high":
        return "text-orange-500 bg-orange-950 border-orange-800"
      case "moderate":
        return "text-yellow-500 bg-yellow-950 border-yellow-800"
      case "low":
        return "text-green-500 bg-green-950 border-green-800"
      default:
        return "text-zinc-500 bg-zinc-950 border-zinc-800"
    }
  }

  const getSeverityLabel = (severity: string) => {
    switch (severity) {
      case "critical":
        return "Crítico"
      case "high":
        return "Alto"
      case "moderate":
        return "Moderado"
      case "low":
        return "Bajo"
      default:
        return severity
    }
  }

  if (!analysis && !isAnalyzing && !error) {
    return (
      <div className="rounded-lg border border-zinc-800 bg-zinc-950 p-4">
        <div className="flex items-center justify-between mb-3">
          <h4 className="font-medium flex items-center gap-2">
            <Brain className="h-4 w-4 text-purple-400" />
            Análisis con IA
          </h4>
          <Button
            size="sm"
            onClick={onAnalyze}
            disabled={disabled || dtcCount === 0}
            className="bg-purple-600 hover:bg-purple-700"
          >
            <Brain className="h-4 w-4 mr-1" />
            Analizar
          </Button>
        </div>
        <p className="text-sm text-zinc-500">
          {dtcCount === 0
            ? "Lee los códigos de error primero"
            : `Analizar ${dtcCount} código${dtcCount !== 1 ? "s" : ""} con inteligencia artificial`}
        </p>
      </div>
    )
  }

  if (isAnalyzing) {
    return (
      <div className="rounded-lg border border-purple-900 bg-purple-950/30 p-4">
        <div className="flex items-center gap-3">
          <Loader2 className="h-5 w-5 animate-spin text-purple-400" />
          <div>
            <p className="font-medium text-purple-300">Analizando códigos...</p>
            <p className="text-sm text-purple-400/70">
              Consultando base de datos y modelo de IA
            </p>
          </div>
        </div>
      </div>
    )
  }

  if (error) {
    return (
      <div className="rounded-lg border border-red-900 bg-red-950/30 p-4">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2 text-red-400">
            <AlertCircle className="h-5 w-5" />
            <span>{error}</span>
          </div>
          <Button
            size="sm"
            variant="outline"
            onClick={onAnalyze}
            className="border-red-800 text-red-400"
          >
            Reintentar
          </Button>
        </div>
      </div>
    )
  }

  if (!analysis) return null

  return (
    <div className="space-y-4">
      {/* Safety Warning */}
      {analysis.safety_warning && (
        <div className="rounded-lg border border-red-800 bg-red-950/50 p-4">
          <div className="flex items-start gap-3">
            <ShieldAlert className="h-5 w-5 text-red-500 flex-shrink-0 mt-0.5" />
            <div>
              <p className="font-medium text-red-400">Advertencia de Seguridad</p>
              <p className="text-sm text-red-300 mt-1">{analysis.safety_warning}</p>
            </div>
          </div>
        </div>
      )}

      {/* Summary */}
      <div className="rounded-lg border border-zinc-800 bg-zinc-950 p-4">
        <div className="flex items-center gap-2 mb-3">
          <FileText className="h-4 w-4 text-blue-400" />
          <h4 className="font-medium">Resumen</h4>
        </div>
        <p className="text-sm text-zinc-300">{analysis.summary}</p>
      </div>

      {/* Priority Order */}
      {analysis.priority_order.length > 0 && (
        <div className="rounded-lg border border-zinc-800 bg-zinc-950 p-4">
          <div className="flex items-center gap-2 mb-3">
            <ListOrdered className="h-4 w-4 text-amber-400" />
            <h4 className="font-medium">Orden de Prioridad</h4>
          </div>
          <div className="flex flex-wrap gap-2">
            {analysis.priority_order.map((code, index) => (
              <span
                key={code}
                className="inline-flex items-center gap-1 px-2 py-1 rounded bg-zinc-800 text-sm font-mono"
              >
                <span className="text-amber-400">{index + 1}.</span>
                <span className="text-white">{code}</span>
              </span>
            ))}
          </div>
        </div>
      )}

      {/* Combined Diagnosis */}
      {analysis.combined_diagnosis && (
        <div className="rounded-lg border border-zinc-800 bg-zinc-950 p-4">
          <div className="flex items-center gap-2 mb-3">
            <Brain className="h-4 w-4 text-purple-400" />
            <h4 className="font-medium">Diagnóstico Combinado</h4>
          </div>
          <p className="text-sm text-zinc-300">{analysis.combined_diagnosis}</p>
        </div>
      )}

      {/* DTC Details */}
      {analysis.dtcs.length > 0 && (
        <div className="rounded-lg border border-zinc-800 bg-zinc-950 p-4">
          <div className="flex items-center gap-2 mb-3">
            <AlertTriangle className="h-4 w-4 text-orange-400" />
            <h4 className="font-medium">Detalle de Códigos</h4>
          </div>
          <div className="space-y-3">
            {analysis.dtcs.map((dtc) => (
              <div
                key={dtc.code}
                className={`rounded-lg border p-3 ${getSeverityColor(dtc.severity)}`}
              >
                <div className="flex items-center justify-between mb-2">
                  <span className="font-mono font-bold">{dtc.code}</span>
                  <span className="text-xs px-2 py-0.5 rounded bg-black/20">
                    {getSeverityLabel(dtc.severity)}
                  </span>
                </div>
                <p className="text-sm mb-2 opacity-90">{dtc.description}</p>

                {dtc.causes.length > 0 && (
                  <div className="text-xs mt-2">
                    <span className="opacity-70">Causas posibles: </span>
                    {dtc.causes.slice(0, 3).join(", ")}
                  </div>
                )}
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Recommended Actions */}
      {analysis.recommended_actions.length > 0 && (
        <div className="rounded-lg border border-zinc-800 bg-zinc-950 p-4">
          <div className="flex items-center gap-2 mb-3">
            <Wrench className="h-4 w-4 text-green-400" />
            <h4 className="font-medium">Acciones Recomendadas</h4>
          </div>
          <ul className="space-y-2">
            {analysis.recommended_actions.map((action, index) => (
              <li key={index} className="flex items-start gap-2 text-sm">
                <CheckCircle2 className="h-4 w-4 text-green-500 flex-shrink-0 mt-0.5" />
                <span className="text-zinc-300">{action}</span>
              </li>
            ))}
          </ul>
        </div>
      )}

      {/* Re-analyze button */}
      <div className="flex justify-end">
        <Button
          size="sm"
          variant="outline"
          onClick={onAnalyze}
          disabled={isAnalyzing}
          className="border-zinc-700"
        >
          <Brain className="h-4 w-4 mr-1" />
          Volver a analizar
        </Button>
      </div>
    </div>
  )
}
