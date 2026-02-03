import type { Request, Response } from "express"
import {
  analyzeDtcs,
  lookupDtcCode,
  getAllKnownCodes,
  type DtcInput,
} from "../services/openaiService.js"

interface AnalyzeRequest {
  dtcs: DtcInput[]
  vehicle?: {
    model?: string
    year?: string
    mileage?: string
  }
}

/**
 * POST /api/dtc/analyze
 * Analyze DTCs with AI - validation handled by middleware
 */
export async function analyzeHandler(
  req: Request<object, object, AnalyzeRequest>,
  res: Response
) {
  try {
    const { dtcs, vehicle } = req.body

    // Input already validated by middleware
    const analysis = await analyzeDtcs(dtcs, vehicle)
    res.json(analysis)
  } catch (error) {
    console.error("Error analyzing DTCs:", error)
    res.status(500).json({
      error: "Analysis failed",
      message: "Unable to process analysis request",
    })
  }
}

/**
 * GET /api/dtc/lookup/:code
 * Look up a DTC code - validation handled by middleware
 */
export function lookupHandler(
  req: Request<{ code: string }>,
  res: Response
) {
  try {
    const { code } = req.params

    // Code already validated by middleware
    const result = lookupDtcCode(code.toUpperCase())

    if (!result) {
      res.status(404).json({
        error: "Not found",
        message: `Code ${code} not found in database`,
      })
      return
    }

    res.json(result)
  } catch (error) {
    console.error("Error looking up DTC:", error)
    res.status(500).json({
      error: "Lookup failed",
      message: "Unable to process lookup request",
    })
  }
}

/**
 * GET /api/dtc/codes
 * List all known DTC codes
 */
export function listCodesHandler(_req: Request, res: Response) {
  try {
    const codes = getAllKnownCodes()
    res.json({
      count: codes.length,
      codes,
    })
  } catch (error) {
    console.error("Error listing codes:", error)
    res.status(500).json({
      error: "List failed",
      message: "Unable to retrieve code list",
    })
  }
}

/**
 * GET /api/health
 * Health check endpoint
 */
export function healthHandler(_req: Request, res: Response) {
  res.json({
    status: "ok",
    timestamp: new Date().toISOString(),
    version: "1.0.0",
  })
}
