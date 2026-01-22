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

// POST /api/dtc/analyze
export async function analyzeHandler(
  req: Request<object, object, AnalyzeRequest>,
  res: Response
) {
  try {
    const { dtcs, vehicle } = req.body

    if (!dtcs || !Array.isArray(dtcs) || dtcs.length === 0) {
      res.status(400).json({
        error: "Invalid request",
        message: "dtcs array is required",
      })
      return
    }

    // Validate DTC format
    for (const dtc of dtcs) {
      if (!dtc.code || typeof dtc.code !== "string") {
        res.status(400).json({
          error: "Invalid request",
          message: "Each DTC must have a code string",
        })
        return
      }
    }

    const analysis = await analyzeDtcs(dtcs, vehicle)
    res.json(analysis)
  } catch (error) {
    console.error("Error analyzing DTCs:", error)
    res.status(500).json({
      error: "Analysis failed",
      message: error instanceof Error ? error.message : "Unknown error",
    })
  }
}

// GET /api/dtc/lookup/:code
export function lookupHandler(
  req: Request<{ code: string }>,
  res: Response
) {
  try {
    const { code } = req.params

    if (!code) {
      res.status(400).json({
        error: "Invalid request",
        message: "code parameter is required",
      })
      return
    }

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
      message: error instanceof Error ? error.message : "Unknown error",
    })
  }
}

// GET /api/dtc/codes
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
      message: error instanceof Error ? error.message : "Unknown error",
    })
  }
}

// GET /api/health
export function healthHandler(_req: Request, res: Response) {
  res.json({
    status: "ok",
    timestamp: new Date().toISOString(),
    version: "1.0.0",
  })
}
