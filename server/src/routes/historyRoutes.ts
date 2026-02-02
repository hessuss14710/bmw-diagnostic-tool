/**
 * History Routes - API endpoints for diagnostic session history
 */

import { Router, Request, Response } from "express"
import {
  saveSession,
  getSession,
  getSessions,
  deleteSession,
  compareSessions,
  getSessionSummary,
  type DiagnosticSessionRecord
} from "../services/historyService.js"
import { checkAllParameters, getAllThresholds, type Alert } from "../services/alertService.js"

const router = Router()

/**
 * GET /api/history
 * Get all diagnostic sessions, optionally filtered by VIN
 */
router.get("/", async (req: Request, res: Response) => {
  try {
    const vin = req.query.vin as string | undefined
    const sessions = await getSessions(vin)
    const summaries = sessions.map(getSessionSummary)

    res.json({
      count: summaries.length,
      sessions: summaries
    })
  } catch (error) {
    res.status(500).json({
      error: "Failed to get sessions",
      message: error instanceof Error ? error.message : "Unknown error"
    })
  }
})

/**
 * GET /api/history/:id
 * Get a specific session by ID
 */
router.get("/:id", async (req: Request, res: Response) => {
  try {
    const session = await getSession(req.params.id as string)

    if (!session) {
      return res.status(404).json({
        error: "Session not found"
      })
    }

    res.json(session)
  } catch (error) {
    res.status(500).json({
      error: "Failed to get session",
      message: error instanceof Error ? error.message : "Unknown error"
    })
  }
})

/**
 * POST /api/history
 * Save a new diagnostic session
 */
router.post("/", async (req: Request, res: Response) => {
  try {
    const sessionData = req.body as DiagnosticSessionRecord

    if (!sessionData.id || !sessionData.createdAt) {
      return res.status(400).json({
        error: "Invalid session data",
        message: "Session must have id and createdAt"
      })
    }

    const sessionId = await saveSession(sessionData)

    res.status(201).json({
      success: true,
      sessionId
    })
  } catch (error) {
    res.status(500).json({
      error: "Failed to save session",
      message: error instanceof Error ? error.message : "Unknown error"
    })
  }
})

/**
 * PUT /api/history/:id
 * Update an existing session
 */
router.put("/:id", async (req: Request, res: Response) => {
  try {
    const id = req.params.id as string
    const existingSession = await getSession(id)

    if (!existingSession) {
      return res.status(404).json({
        error: "Session not found"
      })
    }

    const updatedSession: DiagnosticSessionRecord = {
      ...existingSession,
      ...req.body,
      id, // Ensure ID doesn't change
      createdAt: existingSession.createdAt // Preserve original creation date
    }

    await saveSession(updatedSession)

    res.json({
      success: true,
      sessionId: id
    })
  } catch (error) {
    res.status(500).json({
      error: "Failed to update session",
      message: error instanceof Error ? error.message : "Unknown error"
    })
  }
})

/**
 * DELETE /api/history/:id
 * Delete a session
 */
router.delete("/:id", async (req: Request, res: Response) => {
  try {
    const deleted = await deleteSession(req.params.id as string)

    if (!deleted) {
      return res.status(404).json({
        error: "Session not found"
      })
    }

    res.json({
      success: true
    })
  } catch (error) {
    res.status(500).json({
      error: "Failed to delete session",
      message: error instanceof Error ? error.message : "Unknown error"
    })
  }
})

/**
 * POST /api/history/compare
 * Compare two sessions
 */
router.post("/compare", async (req: Request, res: Response) => {
  try {
    const { sessionId1, sessionId2 } = req.body

    if (!sessionId1 || !sessionId2) {
      return res.status(400).json({
        error: "Both sessionId1 and sessionId2 are required"
      })
    }

    const session1 = await getSession(sessionId1)
    const session2 = await getSession(sessionId2)

    if (!session1) {
      return res.status(404).json({
        error: `Session ${sessionId1} not found`
      })
    }

    if (!session2) {
      return res.status(404).json({
        error: `Session ${sessionId2} not found`
      })
    }

    const comparison = compareSessions(session1, session2)

    res.json({
      session1: {
        id: session1.id,
        date: new Date(session1.createdAt).toISOString(),
        mileage: session1.vehicleInfo.mileage
      },
      session2: {
        id: session2.id,
        date: new Date(session2.createdAt).toISOString(),
        mileage: session2.vehicleInfo.mileage
      },
      comparison
    })
  } catch (error) {
    res.status(500).json({
      error: "Failed to compare sessions",
      message: error instanceof Error ? error.message : "Unknown error"
    })
  }
})

/**
 * POST /api/history/:id/alerts
 * Add alerts to a session based on live data
 */
router.post("/:id/alerts", async (req: Request, res: Response) => {
  try {
    const session = await getSession(req.params.id as string)

    if (!session) {
      return res.status(404).json({
        error: "Session not found"
      })
    }

    const liveData = req.body.liveData as Record<string, { value: number; unit: string }>
    const alerts = checkAllParameters(liveData)

    // Add alerts to session
    session.alerts.push(...alerts.map(a => ({
      timestamp: a.timestamp,
      type: a.type,
      parameter: a.parameter,
      value: a.value,
      threshold: a.threshold,
      message: a.message
    })))

    await saveSession(session)

    res.json({
      alertsGenerated: alerts.length,
      alerts
    })
  } catch (error) {
    res.status(500).json({
      error: "Failed to check alerts",
      message: error instanceof Error ? error.message : "Unknown error"
    })
  }
})

/**
 * GET /api/alerts/thresholds
 * Get all defined parameter thresholds
 */
router.get("/alerts/thresholds", (_req: Request, res: Response) => {
  res.json({
    thresholds: getAllThresholds()
  })
})

/**
 * POST /api/alerts/check
 * Check live data against thresholds without saving
 */
router.post("/alerts/check", (req: Request, res: Response) => {
  try {
    const liveData = req.body as Record<string, { value: number; unit: string }>
    const alerts = checkAllParameters(liveData)

    res.json({
      hasAlerts: alerts.length > 0,
      criticalCount: alerts.filter(a => a.type === "critical").length,
      warningCount: alerts.filter(a => a.type === "warning").length,
      alerts
    })
  } catch (error) {
    res.status(500).json({
      error: "Failed to check alerts",
      message: error instanceof Error ? error.message : "Unknown error"
    })
  }
})

export default router
