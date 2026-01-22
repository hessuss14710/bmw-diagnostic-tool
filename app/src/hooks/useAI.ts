import { useState, useCallback } from "react"
import type { Dtc } from "./useBMW"

// API base URL - can be configured via environment
const API_URL = import.meta.env.VITE_API_URL || "https://taller.giormo.com/api"

export interface DtcAnalysis {
  code: string
  description: string
  severity: "low" | "moderate" | "high" | "critical"
  category: string
  symptoms: string[]
  causes: string[]
  repairs: string[]
  bmw_specific: boolean
  estimated_cost?: string
}

export interface FullAnalysis {
  dtcs: DtcAnalysis[]
  summary: string
  priority_order: string[]
  combined_diagnosis: string
  recommended_actions: string[]
  safety_warning?: string
}

export function useAI() {
  const [analysis, setAnalysis] = useState<FullAnalysis | null>(null)
  const [isAnalyzing, setIsAnalyzing] = useState(false)
  const [error, setError] = useState<string | null>(null)

  // Analyze DTCs with AI
  const analyzeDtcs = useCallback(
    async (
      dtcs: Dtc[],
      vehicle?: { model?: string; year?: string; mileage?: string }
    ) => {
      if (dtcs.length === 0) {
        setError("No hay códigos de error para analizar")
        return null
      }

      setIsAnalyzing(true)
      setError(null)

      try {
        const response = await fetch(`${API_URL}/dtc/analyze`, {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
          },
          body: JSON.stringify({
            dtcs: dtcs.map((dtc) => ({
              code: dtc.code,
              status: {
                confirmed: dtc.status.confirmed,
                pending: dtc.status.pending,
                test_failed: dtc.status.test_failed,
              },
            })),
            vehicle,
          }),
        })

        if (!response.ok) {
          const errorData = await response.json().catch(() => ({}))
          throw new Error(
            errorData.message || `Error del servidor: ${response.status}`
          )
        }

        const result = (await response.json()) as FullAnalysis
        setAnalysis(result)
        return result
      } catch (e) {
        const errorMsg =
          e instanceof Error ? e.message : "Error desconocido al analizar"
        setError(errorMsg)
        return null
      } finally {
        setIsAnalyzing(false)
      }
    },
    []
  )

  // Lookup a single code
  const lookupCode = useCallback(async (code: string) => {
    try {
      const response = await fetch(`${API_URL}/dtc/lookup/${code}`)

      if (!response.ok) {
        if (response.status === 404) {
          return null
        }
        throw new Error(`Error del servidor: ${response.status}`)
      }

      return (await response.json()) as DtcAnalysis
    } catch (e) {
      console.error("Error looking up code:", e)
      return null
    }
  }, [])

  // Clear analysis
  const clearAnalysis = useCallback(() => {
    setAnalysis(null)
    setError(null)
  }, [])

  return {
    analysis,
    isAnalyzing,
    error,
    analyzeDtcs,
    lookupCode,
    clearAnalysis,
  }
}
