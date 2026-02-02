import { useState, useEffect, useCallback } from "react"
import { History, RefreshCw, Trash2, ArrowLeftRight, AlertTriangle, CheckCircle, Clock, Car } from "lucide-react"

interface SessionSummary {
  id: string
  date: string
  vehicleInfo: {
    model?: string
    vin?: string
    mileage?: number
  }
  ecuId: string
  protocol: string
  dtcCount: number
  alertCount: number
  hasLiveData: boolean
}

interface DtcRecord {
  code: string
  description: string | null
  status: {
    confirmed: boolean
    pending: boolean
  }
}

interface SessionDetail {
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
  alerts: Array<{
    timestamp: number
    type: "warning" | "critical"
    parameter: string
    value: number
    threshold: number
    message: string
  }>
  notes?: string
}

interface ComparisonResult {
  session1: { id: string; date: string; mileage?: number }
  session2: { id: string; date: string; mileage?: number }
  comparison: {
    newDtcs: DtcRecord[]
    resolvedDtcs: DtcRecord[]
    persistentDtcs: DtcRecord[]
    mileageDiff: number | null
  }
}

interface HistoryPanelProps {
  serverUrl?: string
}

export function HistoryPanel({ serverUrl = "http://localhost:3002" }: HistoryPanelProps) {
  const [sessions, setSessions] = useState<SessionSummary[]>([])
  const [selectedSession, setSelectedSession] = useState<SessionDetail | null>(null)
  const [compareMode, setCompareMode] = useState(false)
  const [selectedForCompare, setSelectedForCompare] = useState<string[]>([])
  const [comparisonResult, setComparisonResult] = useState<ComparisonResult | null>(null)
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  // Fetch sessions
  const fetchSessions = useCallback(async () => {
    setIsLoading(true)
    setError(null)
    try {
      const response = await fetch(`${serverUrl}/api/history`)
      if (!response.ok) throw new Error("Failed to fetch sessions")
      const data = await response.json()
      setSessions(data.sessions || [])
    } catch (e) {
      setError(e instanceof Error ? e.message : "Error fetching sessions")
    } finally {
      setIsLoading(false)
    }
  }, [serverUrl])

  // Fetch session detail
  const fetchSessionDetail = useCallback(async (id: string) => {
    setIsLoading(true)
    setError(null)
    try {
      const response = await fetch(`${serverUrl}/api/history/${id}`)
      if (!response.ok) throw new Error("Session not found")
      const data = await response.json()
      setSelectedSession(data)
    } catch (e) {
      setError(e instanceof Error ? e.message : "Error fetching session")
    } finally {
      setIsLoading(false)
    }
  }, [serverUrl])

  // Delete session
  const deleteSession = useCallback(async (id: string) => {
    if (!confirm("Are you sure you want to delete this session?")) return

    try {
      const response = await fetch(`${serverUrl}/api/history/${id}`, {
        method: "DELETE"
      })
      if (!response.ok) throw new Error("Failed to delete session")
      await fetchSessions()
      if (selectedSession?.id === id) {
        setSelectedSession(null)
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : "Error deleting session")
    }
  }, [serverUrl, selectedSession, fetchSessions])

  // Compare sessions
  const compareSessions = useCallback(async () => {
    if (selectedForCompare.length !== 2) return

    setIsLoading(true)
    setError(null)
    try {
      const response = await fetch(`${serverUrl}/api/history/compare`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          sessionId1: selectedForCompare[0],
          sessionId2: selectedForCompare[1]
        })
      })
      if (!response.ok) throw new Error("Failed to compare sessions")
      const data = await response.json()
      setComparisonResult(data)
    } catch (e) {
      setError(e instanceof Error ? e.message : "Error comparing sessions")
    } finally {
      setIsLoading(false)
    }
  }, [serverUrl, selectedForCompare])

  // Load sessions on mount
  useEffect(() => {
    fetchSessions()
  }, [fetchSessions])

  // Handle session selection for compare
  const toggleCompareSelection = (id: string) => {
    if (selectedForCompare.includes(id)) {
      setSelectedForCompare(prev => prev.filter(s => s !== id))
    } else if (selectedForCompare.length < 2) {
      setSelectedForCompare(prev => [...prev, id])
    }
  }

  const formatDate = (dateStr: string | number) => {
    const date = new Date(dateStr)
    return date.toLocaleDateString("es-ES", {
      year: "numeric",
      month: "short",
      day: "numeric",
      hour: "2-digit",
      minute: "2-digit"
    })
  }

  return (
    <div className="rounded-lg border border-zinc-800 bg-zinc-950 p-4 space-y-4">
      <div className="flex items-center justify-between">
        <h3 className="text-lg font-semibold flex items-center gap-2">
          <History className="h-5 w-5 text-blue-500" />
          Diagnostic History
        </h3>
        <div className="flex items-center gap-2">
          <button
            onClick={() => {
              setCompareMode(!compareMode)
              setSelectedForCompare([])
              setComparisonResult(null)
            }}
            className={`px-3 py-1.5 rounded text-sm flex items-center gap-1 ${
              compareMode
                ? "bg-purple-600 text-white"
                : "bg-zinc-800 text-zinc-400 hover:bg-zinc-700"
            }`}
          >
            <ArrowLeftRight className="h-3.5 w-3.5" />
            Compare
          </button>
          <button
            onClick={fetchSessions}
            disabled={isLoading}
            className="p-1.5 rounded bg-zinc-800 hover:bg-zinc-700 disabled:opacity-50"
          >
            <RefreshCw className={`h-4 w-4 ${isLoading ? "animate-spin" : ""}`} />
          </button>
        </div>
      </div>

      {error && (
        <div className="p-3 rounded-lg bg-red-900/30 border border-red-800 text-red-300 text-sm">
          {error}
        </div>
      )}

      {/* Compare Mode Instructions */}
      {compareMode && (
        <div className="p-3 rounded-lg bg-purple-900/30 border border-purple-800 text-purple-300 text-sm">
          Select 2 sessions to compare. Selected: {selectedForCompare.length}/2
          {selectedForCompare.length === 2 && (
            <button
              onClick={compareSessions}
              disabled={isLoading}
              className="ml-3 px-3 py-1 rounded bg-purple-600 hover:bg-purple-500 text-white"
            >
              Compare Now
            </button>
          )}
        </div>
      )}

      {/* Comparison Result */}
      {comparisonResult && (
        <div className="space-y-3 p-4 rounded-lg bg-zinc-900 border border-zinc-800">
          <h4 className="font-medium text-sm text-zinc-400">Comparison Results</h4>

          <div className="grid grid-cols-2 gap-4 text-sm">
            <div>
              <div className="text-zinc-500">Session 1</div>
              <div>{formatDate(comparisonResult.session1.date)}</div>
              {comparisonResult.session1.mileage && (
                <div className="text-zinc-500">{comparisonResult.session1.mileage.toLocaleString()} km</div>
              )}
            </div>
            <div>
              <div className="text-zinc-500">Session 2</div>
              <div>{formatDate(comparisonResult.session2.date)}</div>
              {comparisonResult.session2.mileage && (
                <div className="text-zinc-500">{comparisonResult.session2.mileage.toLocaleString()} km</div>
              )}
            </div>
          </div>

          {comparisonResult.comparison.mileageDiff !== null && (
            <div className="text-sm">
              <span className="text-zinc-500">Distance traveled: </span>
              <span className="font-mono">{comparisonResult.comparison.mileageDiff.toLocaleString()} km</span>
            </div>
          )}

          <div className="grid grid-cols-3 gap-3">
            <div className="p-2 rounded bg-red-900/30 border border-red-800">
              <div className="text-xs text-red-400 mb-1">New Faults</div>
              <div className="text-lg font-mono text-red-300">
                {comparisonResult.comparison.newDtcs.length}
              </div>
            </div>
            <div className="p-2 rounded bg-green-900/30 border border-green-800">
              <div className="text-xs text-green-400 mb-1">Resolved</div>
              <div className="text-lg font-mono text-green-300">
                {comparisonResult.comparison.resolvedDtcs.length}
              </div>
            </div>
            <div className="p-2 rounded bg-amber-900/30 border border-amber-800">
              <div className="text-xs text-amber-400 mb-1">Persistent</div>
              <div className="text-lg font-mono text-amber-300">
                {comparisonResult.comparison.persistentDtcs.length}
              </div>
            </div>
          </div>

          {comparisonResult.comparison.newDtcs.length > 0 && (
            <div>
              <div className="text-xs text-zinc-500 mb-1">New Faults:</div>
              <div className="space-y-1">
                {comparisonResult.comparison.newDtcs.map((dtc, i) => (
                  <div key={i} className="text-sm font-mono text-red-300">
                    {dtc.code} {dtc.description && `- ${dtc.description}`}
                  </div>
                ))}
              </div>
            </div>
          )}

          {comparisonResult.comparison.resolvedDtcs.length > 0 && (
            <div>
              <div className="text-xs text-zinc-500 mb-1">Resolved Faults:</div>
              <div className="space-y-1">
                {comparisonResult.comparison.resolvedDtcs.map((dtc, i) => (
                  <div key={i} className="text-sm font-mono text-green-300">
                    {dtc.code} {dtc.description && `- ${dtc.description}`}
                  </div>
                ))}
              </div>
            </div>
          )}

          <button
            onClick={() => {
              setComparisonResult(null)
              setSelectedForCompare([])
              setCompareMode(false)
            }}
            className="text-sm text-zinc-500 hover:text-zinc-300"
          >
            Close comparison
          </button>
        </div>
      )}

      {/* Session List */}
      {!comparisonResult && (
        <div className="space-y-2 max-h-80 overflow-y-auto">
          {sessions.length === 0 ? (
            <div className="text-center py-8 text-zinc-500">
              No diagnostic sessions saved yet
            </div>
          ) : (
            sessions.map((session) => (
              <div
                key={session.id}
                className={`p-3 rounded-lg border transition-colors cursor-pointer ${
                  compareMode && selectedForCompare.includes(session.id)
                    ? "bg-purple-900/30 border-purple-700"
                    : selectedSession?.id === session.id
                    ? "bg-zinc-800 border-zinc-700"
                    : "bg-zinc-900 border-zinc-800 hover:bg-zinc-800"
                }`}
                onClick={() => {
                  if (compareMode) {
                    toggleCompareSelection(session.id)
                  } else {
                    fetchSessionDetail(session.id)
                  }
                }}
              >
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    <Clock className="h-4 w-4 text-zinc-500" />
                    <span className="text-sm">{formatDate(session.date)}</span>
                  </div>
                  <div className="flex items-center gap-2">
                    <span className="text-xs px-2 py-0.5 rounded bg-zinc-800 text-zinc-400">
                      {session.ecuId}
                    </span>
                    {session.dtcCount > 0 && (
                      <span className="text-xs px-2 py-0.5 rounded bg-red-900/50 text-red-300">
                        {session.dtcCount} DTCs
                      </span>
                    )}
                    {session.alertCount > 0 && (
                      <AlertTriangle className="h-4 w-4 text-amber-500" />
                    )}
                    {!compareMode && (
                      <button
                        onClick={(e) => {
                          e.stopPropagation()
                          deleteSession(session.id)
                        }}
                        className="p-1 rounded hover:bg-zinc-700 text-zinc-500 hover:text-red-400"
                      >
                        <Trash2 className="h-3.5 w-3.5" />
                      </button>
                    )}
                  </div>
                </div>
                {session.vehicleInfo.vin && (
                  <div className="text-xs text-zinc-500 mt-1 flex items-center gap-1">
                    <Car className="h-3 w-3" />
                    VIN: {session.vehicleInfo.vin}
                    {session.vehicleInfo.mileage && (
                      <span className="ml-2">{session.vehicleInfo.mileage.toLocaleString()} km</span>
                    )}
                  </div>
                )}
              </div>
            ))
          )}
        </div>
      )}

      {/* Session Detail */}
      {selectedSession && !comparisonResult && (
        <div className="space-y-3 p-4 rounded-lg bg-zinc-900 border border-zinc-800">
          <div className="flex items-center justify-between">
            <h4 className="font-medium text-sm text-zinc-400">Session Details</h4>
            <button
              onClick={() => setSelectedSession(null)}
              className="text-xs text-zinc-500 hover:text-zinc-300"
            >
              Close
            </button>
          </div>

          <div className="grid grid-cols-2 gap-3 text-sm">
            <div>
              <span className="text-zinc-500">ECU: </span>
              <span>{selectedSession.ecuId}</span>
            </div>
            <div>
              <span className="text-zinc-500">Protocol: </span>
              <span>{selectedSession.protocol}</span>
            </div>
            <div>
              <span className="text-zinc-500">Date: </span>
              <span>{formatDate(selectedSession.createdAt)}</span>
            </div>
            {selectedSession.vehicleInfo.mileage && (
              <div>
                <span className="text-zinc-500">Mileage: </span>
                <span>{selectedSession.vehicleInfo.mileage.toLocaleString()} km</span>
              </div>
            )}
          </div>

          {/* DTCs */}
          {selectedSession.dtcs.length > 0 && (
            <div>
              <div className="text-xs text-zinc-500 mb-2 flex items-center gap-1">
                <AlertTriangle className="h-3 w-3" />
                Fault Codes ({selectedSession.dtcs.length})
              </div>
              <div className="space-y-1 max-h-32 overflow-y-auto">
                {selectedSession.dtcs.map((dtc, i) => (
                  <div
                    key={i}
                    className="flex items-center gap-2 p-2 rounded bg-red-900/30 border border-red-800 text-sm"
                  >
                    <span className="font-mono text-red-300">{dtc.code}</span>
                    {dtc.description && (
                      <span className="text-zinc-400 truncate">{dtc.description}</span>
                    )}
                    {dtc.status.confirmed && (
                      <span className="ml-auto text-xs bg-red-800 px-1.5 py-0.5 rounded">Confirmed</span>
                    )}
                  </div>
                ))}
              </div>
            </div>
          )}

          {selectedSession.dtcs.length === 0 && (
            <div className="flex items-center gap-2 text-green-400 text-sm">
              <CheckCircle className="h-4 w-4" />
              No fault codes
            </div>
          )}

          {/* Alerts */}
          {selectedSession.alerts.length > 0 && (
            <div>
              <div className="text-xs text-zinc-500 mb-2">
                Alerts ({selectedSession.alerts.length})
              </div>
              <div className="space-y-1 max-h-32 overflow-y-auto">
                {selectedSession.alerts.map((alert, i) => (
                  <div
                    key={i}
                    className={`p-2 rounded text-sm ${
                      alert.type === "critical"
                        ? "bg-red-900/30 border border-red-800 text-red-300"
                        : "bg-amber-900/30 border border-amber-800 text-amber-300"
                    }`}
                  >
                    {alert.message}
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Notes */}
          {selectedSession.notes && (
            <div>
              <div className="text-xs text-zinc-500 mb-1">Notes</div>
              <div className="text-sm text-zinc-300 p-2 rounded bg-zinc-800">
                {selectedSession.notes}
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  )
}
