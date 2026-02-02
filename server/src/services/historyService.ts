/**
 * History Service - Stores and retrieves diagnostic sessions
 *
 * Saves sessions to JSON files for persistence without requiring a database.
 */

import fs from "fs/promises"
import path from "path"
import { fileURLToPath } from "url"

const __dirname = path.dirname(fileURLToPath(import.meta.url))
const HISTORY_DIR = path.join(__dirname, "..", "..", "data", "history")

export interface DtcRecord {
  code: string
  description: string | null
  status: {
    confirmed: boolean
    pending: boolean
    raw: number
  }
}

export interface LiveDataSnapshot {
  timestamp: number
  data: Record<string, {
    value: number
    unit: string
  }>
}

export interface AlertRecord {
  timestamp: number
  type: "warning" | "critical"
  parameter: string
  value: number
  threshold: number
  message: string
}

export interface DiagnosticSessionRecord {
  id: string
  createdAt: number
  updatedAt: number
  vehicleInfo: {
    model?: string
    vin?: string
    mileage?: number
  }
  ecuId: string
  protocol: string
  dtcs: DtcRecord[]
  liveDataSnapshots: LiveDataSnapshot[]
  alerts: AlertRecord[]
  notes?: string
}

// Ensure history directory exists
async function ensureHistoryDir(): Promise<void> {
  try {
    await fs.mkdir(HISTORY_DIR, { recursive: true })
  } catch (error) {
    // Directory may already exist
  }
}

/**
 * Save a diagnostic session
 */
export async function saveSession(session: DiagnosticSessionRecord): Promise<string> {
  await ensureHistoryDir()

  const filename = `${session.id}.json`
  const filepath = path.join(HISTORY_DIR, filename)

  session.updatedAt = Date.now()

  await fs.writeFile(filepath, JSON.stringify(session, null, 2), "utf-8")

  return session.id
}

/**
 * Get a session by ID
 */
export async function getSession(sessionId: string): Promise<DiagnosticSessionRecord | null> {
  await ensureHistoryDir()

  const filepath = path.join(HISTORY_DIR, `${sessionId}.json`)

  try {
    const content = await fs.readFile(filepath, "utf-8")
    return JSON.parse(content) as DiagnosticSessionRecord
  } catch (error) {
    return null
  }
}

/**
 * Get all sessions, optionally filtered by VIN
 */
export async function getSessions(vin?: string): Promise<DiagnosticSessionRecord[]> {
  await ensureHistoryDir()

  try {
    const files = await fs.readdir(HISTORY_DIR)
    const sessions: DiagnosticSessionRecord[] = []

    for (const file of files) {
      if (!file.endsWith(".json")) continue

      const filepath = path.join(HISTORY_DIR, file)
      const content = await fs.readFile(filepath, "utf-8")
      const session = JSON.parse(content) as DiagnosticSessionRecord

      if (vin && session.vehicleInfo.vin !== vin) continue

      sessions.push(session)
    }

    // Sort by date, newest first
    sessions.sort((a, b) => b.createdAt - a.createdAt)

    return sessions
  } catch (error) {
    return []
  }
}

/**
 * Delete a session
 */
export async function deleteSession(sessionId: string): Promise<boolean> {
  await ensureHistoryDir()

  const filepath = path.join(HISTORY_DIR, `${sessionId}.json`)

  try {
    await fs.unlink(filepath)
    return true
  } catch (error) {
    return false
  }
}

/**
 * Compare two sessions to find differences
 */
export function compareSessions(
  session1: DiagnosticSessionRecord,
  session2: DiagnosticSessionRecord
): {
  newDtcs: DtcRecord[]
  resolvedDtcs: DtcRecord[]
  persistentDtcs: DtcRecord[]
  mileageDiff: number | null
} {
  const dtcCodes1 = new Set(session1.dtcs.map(d => d.code))
  const dtcCodes2 = new Set(session2.dtcs.map(d => d.code))

  // Find new DTCs (in session2 but not in session1)
  const newDtcs = session2.dtcs.filter(d => !dtcCodes1.has(d.code))

  // Find resolved DTCs (in session1 but not in session2)
  const resolvedDtcs = session1.dtcs.filter(d => !dtcCodes2.has(d.code))

  // Find persistent DTCs (in both sessions)
  const persistentDtcs = session2.dtcs.filter(d => dtcCodes1.has(d.code))

  // Calculate mileage difference
  let mileageDiff: number | null = null
  if (session1.vehicleInfo.mileage && session2.vehicleInfo.mileage) {
    mileageDiff = session2.vehicleInfo.mileage - session1.vehicleInfo.mileage
  }

  return {
    newDtcs,
    resolvedDtcs,
    persistentDtcs,
    mileageDiff
  }
}

/**
 * Create a new session record from current diagnostic data
 */
export function createSessionRecord(
  vehicleInfo: { model?: string; vin?: string; mileage?: number },
  ecuId: string,
  protocol: string,
  dtcs: DtcRecord[]
): DiagnosticSessionRecord {
  const now = Date.now()
  const id = `diag-${now}-${Math.random().toString(36).substring(2, 8)}`

  return {
    id,
    createdAt: now,
    updatedAt: now,
    vehicleInfo,
    ecuId,
    protocol,
    dtcs,
    liveDataSnapshots: [],
    alerts: []
  }
}

/**
 * Add a live data snapshot to a session
 */
export function addLiveDataSnapshot(
  session: DiagnosticSessionRecord,
  data: Record<string, { value: number; unit: string }>
): void {
  session.liveDataSnapshots.push({
    timestamp: Date.now(),
    data
  })

  // Keep only last 100 snapshots to avoid huge files
  if (session.liveDataSnapshots.length > 100) {
    session.liveDataSnapshots = session.liveDataSnapshots.slice(-100)
  }
}

/**
 * Get session summary for list display
 */
export function getSessionSummary(session: DiagnosticSessionRecord) {
  return {
    id: session.id,
    date: new Date(session.createdAt).toISOString(),
    vehicleInfo: session.vehicleInfo,
    ecuId: session.ecuId,
    protocol: session.protocol,
    dtcCount: session.dtcs.length,
    alertCount: session.alerts.length,
    hasLiveData: session.liveDataSnapshots.length > 0
  }
}
