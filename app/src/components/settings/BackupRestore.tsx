/**
 * Backup and restore component for database data
 */

import { useState } from "react"
import { useDatabase, type DatabaseStats } from "@/hooks/useDatabase"
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  Button,
  Alert,
  Badge,
} from "@/components/ui"
import {
  Download,
  Upload,
  Database,
  Car,
  History,
  AlertTriangle,
  RefreshCw,
} from "lucide-react"

export function BackupRestore() {
  const db = useDatabase()
  const [stats, setStats] = useState<DatabaseStats | null>(null)
  const [exportData, setExportData] = useState<string | null>(null)
  const [showExport, setShowExport] = useState(false)

  const loadStats = async () => {
    try {
      const data = await db.getStats()
      setStats(data)
    } catch {
      // Error handled in hook
    }
  }

  const handleExport = async () => {
    try {
      const data = await db.exportAll()
      setExportData(data)
      setShowExport(true)
    } catch {
      // Error handled in hook
    }
  }

  const handleDownload = () => {
    if (!exportData) return

    const blob = new Blob([exportData], { type: "application/json" })
    const url = URL.createObjectURL(blob)
    const a = document.createElement("a")
    a.href = url
    a.download = `bmw-diag-backup-${new Date().toISOString().split("T")[0]}.json`
    document.body.appendChild(a)
    a.click()
    document.body.removeChild(a)
    URL.revokeObjectURL(url)
  }

  const handleCopyToClipboard = async () => {
    if (!exportData) return
    try {
      await navigator.clipboard.writeText(exportData)
    } catch (e) {
      console.error("Failed to copy:", e)
    }
  }

  return (
    <div className="space-y-4">
      {/* Database Stats */}
      <Card variant="elevated" padding="md">
        <CardHeader>
          <CardTitle>
            <Database className="h-5 w-5 text-cyan-400" />
            Database Statistics
          </CardTitle>
          <Button
            size="sm"
            variant="outline"
            onClick={loadStats}
            loading={db.isLoading}
            leftIcon={<RefreshCw className="h-4 w-4" />}
          >
            Refresh
          </Button>
        </CardHeader>
        <CardContent>
          {stats ? (
            <div className="grid grid-cols-3 gap-4">
              <StatCard
                icon={Car}
                label="Vehicles"
                value={stats.vehicle_count}
                color="blue"
              />
              <StatCard
                icon={History}
                label="Sessions"
                value={stats.session_count}
                color="purple"
              />
              <StatCard
                icon={AlertTriangle}
                label="DTCs Stored"
                value={stats.dtc_count}
                color="amber"
              />
            </div>
          ) : (
            <p className="text-sm text-zinc-500 text-center py-4">
              Click "Refresh" to load database statistics
            </p>
          )}
        </CardContent>
      </Card>

      {/* Error */}
      {db.error && (
        <Alert variant="error" title="Error" onClose={db.clearError}>
          {db.error}
        </Alert>
      )}

      {/* Export Section */}
      <Card variant="elevated" padding="md">
        <CardHeader>
          <CardTitle>
            <Download className="h-5 w-5 text-emerald-400" />
            Export Data
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <p className="text-sm text-zinc-400">
            Export all your data (vehicles, diagnostic sessions, DTCs, and settings) as a JSON file for backup purposes.
          </p>

          <Button
            onClick={handleExport}
            loading={db.isLoading}
            leftIcon={<Download className="h-4 w-4" />}
            variant="success"
          >
            Generate Backup
          </Button>

          {showExport && exportData && (
            <div className="space-y-3 pt-4 border-t border-zinc-800">
              <Alert variant="success" title="Backup Generated">
                Your backup file is ready. Download it or copy to clipboard.
              </Alert>

              <div className="flex gap-2">
                <Button onClick={handleDownload} leftIcon={<Download className="h-4 w-4" />}>
                  Download JSON
                </Button>
                <Button variant="outline" onClick={handleCopyToClipboard}>
                  Copy to Clipboard
                </Button>
              </div>

              <details className="mt-4">
                <summary className="text-sm text-zinc-400 cursor-pointer hover:text-zinc-300">
                  Preview export data
                </summary>
                <pre className="mt-2 p-3 bg-zinc-900 rounded-lg text-xs text-zinc-400 overflow-auto max-h-64">
                  {exportData}
                </pre>
              </details>
            </div>
          )}
        </CardContent>
      </Card>

      {/* Import Section (placeholder) */}
      <Card variant="default" padding="md" className="opacity-60">
        <CardHeader>
          <CardTitle>
            <Upload className="h-5 w-5 text-zinc-500" />
            Import Data
            <Badge variant="outline" size="sm" className="ml-2">
              Coming Soon
            </Badge>
          </CardTitle>
        </CardHeader>
        <CardContent>
          <p className="text-sm text-zinc-500">
            Import functionality will be available in a future update.
          </p>
        </CardContent>
      </Card>
    </div>
  )
}

function StatCard({
  icon: Icon,
  label,
  value,
  color,
}: {
  icon: typeof Car
  label: string
  value: number
  color: "blue" | "purple" | "amber"
}) {
  const colors = {
    blue: "text-blue-400 bg-blue-600/20 border-blue-600/30",
    purple: "text-purple-400 bg-purple-600/20 border-purple-600/30",
    amber: "text-amber-400 bg-amber-600/20 border-amber-600/30",
  }

  return (
    <div className="p-4 rounded-lg bg-zinc-900/50 border border-zinc-800 text-center">
      <div className={`inline-flex p-2 rounded-lg mb-2 border ${colors[color]}`}>
        <Icon className="h-5 w-5" />
      </div>
      <p className="text-2xl font-bold text-white">{value}</p>
      <p className="text-xs text-zinc-500">{label}</p>
    </div>
  )
}
