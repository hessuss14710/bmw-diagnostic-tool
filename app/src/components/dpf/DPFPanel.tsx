import { useState } from "react"
import { useDPF } from "@/hooks/useDPF"
import type { EcuInfo } from "@/hooks/useBMW"
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  Button,
  Badge,
  Alert,
  Progress,
  ConfirmModal,
  EmptyState,
} from "@/components/ui"
import {
  RefreshCw,
  Trash2,
  Flame,
  Square,
  ShieldCheck,
  ShieldX,
  CheckCircle2,
  AlertTriangle,
  Gauge,
  Thermometer,
  Wind,
  RotateCcw,
  PackagePlus,
  Zap,
  Filter,
} from "lucide-react"

interface DPFPanelProps {
  isConnected: boolean
  isInitialized: boolean
  selectedEcu: EcuInfo | null
}

export function DPFPanel({ isConnected, isInitialized, selectedEcu }: DPFPanelProps) {
  const dpf = useDPF()
  const [confirmAction, setConfirmAction] = useState<string | null>(null)

  const targetAddress = selectedEcu?.kline_address ?? 0x12
  const isDDE = selectedEcu?.id === "DDE" || selectedEcu?.id === "DME"

  const handleStartSession = async () => {
    try {
      await dpf.startExtendedSession(targetAddress)
    } catch {
      // Error handled in hook
    }
  }

  const handleSecurityAccess = async () => {
    try {
      await dpf.securityAccess(targetAddress)
    } catch {
      // Error handled in hook
    }
  }

  const handleReadStatus = async () => {
    try {
      await dpf.readStatus(targetAddress)
    } catch {
      // Error handled in hook
    }
  }

  const handleConfirmAction = async (action: string) => {
    setConfirmAction(null)
    try {
      switch (action) {
        case "ash":
          await dpf.resetAsh(targetAddress)
          break
        case "learned":
          await dpf.resetLearned(targetAddress)
          break
        case "new":
          await dpf.newDpfInstalled(targetAddress)
          break
        case "regen":
          await dpf.startRegen(targetAddress)
          break
      }
    } catch {
      // Error handled in hook
    }
  }

  const handleStopRegen = async () => {
    try {
      await dpf.stopRegen(targetAddress)
    } catch {
      // Error handled in hook
    }
  }

  if (!isConnected) {
    return (
      <Card variant="default" padding="lg" className="animate-fade-in">
        <EmptyState
          variant="offline"
          title="No Connection"
          description="Connect to your K+DCAN cable first"
        />
      </Card>
    )
  }

  if (!isInitialized) {
    return (
      <Card variant="default" padding="lg" className="animate-fade-in">
        <EmptyState
          variant="error"
          icon={AlertTriangle}
          title="ECU Not Initialized"
          description="Initialize communication with the ECU in the Diagnostics tab"
        />
      </Card>
    )
  }

  if (!isDDE) {
    return (
      <Card variant="default" padding="lg" className="animate-fade-in">
        <EmptyState
          variant="error"
          icon={Filter}
          title="Select DDE ECU"
          description="DPF functions are only available for diesel engines (DDE - Digital Diesel Electronics)"
        />
      </Card>
    )
  }

  // Calculate soot loading variant
  const getSootVariant = (value: number | null): "success" | "warning" | "error" => {
    if (value === null) return "success"
    if (value > 80) return "error"
    if (value > 50) return "warning"
    return "success"
  }

  return (
    <div className="space-y-4">
      {/* Session & Security Status */}
      <Card variant="elevated" padding="md" className="animate-fade-in">
        <CardHeader>
          <CardTitle>
            <Zap className="h-5 w-5 text-blue-400" />
            Session Status
          </CardTitle>
          <div className="flex gap-2">
            <Badge
              variant={dpf.sessionActive ? "success" : "outline"}
              size="sm"
            >
              {dpf.sessionActive ? "Extended Session" : "No Session"}
            </Badge>
            <Badge
              variant={dpf.securityUnlocked ? "success" : "outline"}
              size="sm"
            >
              {dpf.securityUnlocked ? (
                <><ShieldCheck className="h-3 w-3" /> Unlocked</>
              ) : (
                <><ShieldX className="h-3 w-3" /> Locked</>
              )}
            </Badge>
          </div>
        </CardHeader>
        <CardContent>
          <div className="flex gap-2">
            <Button
              size="sm"
              variant={dpf.sessionActive ? "secondary" : "default"}
              onClick={handleStartSession}
              loading={dpf.isLoading}
              disabled={dpf.sessionActive}
              leftIcon={<CheckCircle2 className="h-4 w-4" />}
            >
              Start Session
            </Button>
            <Button
              size="sm"
              variant={dpf.securityUnlocked ? "secondary" : "outline"}
              onClick={handleSecurityAccess}
              loading={dpf.isLoading}
              disabled={!dpf.sessionActive || dpf.securityUnlocked}
              leftIcon={<ShieldCheck className="h-4 w-4" />}
            >
              Unlock Security
            </Button>
          </div>
        </CardContent>
      </Card>

      {/* Error Display */}
      {dpf.error && (
        <Alert variant="error" title="Error" onClose={dpf.clearError}>
          {dpf.error}
        </Alert>
      )}

      {/* Last Result */}
      {dpf.lastResult && (
        <Alert
          variant={dpf.lastResult.success ? "success" : "warning"}
          title={`Routine 0x${dpf.lastResult.routine_id.toString(16).toUpperCase().padStart(4, '0')}`}
        >
          {dpf.lastResult.status}
        </Alert>
      )}

      {/* DPF Status */}
      <Card variant="elevated" padding="md" className="animate-fade-in-up">
        <CardHeader>
          <CardTitle>
            <Filter className="h-5 w-5 text-amber-400" />
            DPF Status
            {dpf.status?.regen_active && (
              <Badge variant="warning" size="sm" pulse className="ml-2">
                <Flame className="h-3 w-3" /> Regenerating
              </Badge>
            )}
          </CardTitle>
          <Button
            size="sm"
            variant="outline"
            onClick={handleReadStatus}
            loading={dpf.isLoading}
            leftIcon={<RefreshCw className="h-4 w-4" />}
          >
            Read
          </Button>
        </CardHeader>
        <CardContent>
          {dpf.status ? (
            <div className="grid grid-cols-2 gap-3">
              {/* Soot Loading */}
              <div className="col-span-2 p-4 rounded-lg bg-zinc-900/50 border border-zinc-800">
                <div className="flex items-center justify-between mb-2">
                  <div className="flex items-center gap-2 text-sm text-zinc-400">
                    <Gauge className="h-4 w-4" />
                    <span>Soot Loading</span>
                  </div>
                  <span className="text-xl font-mono font-semibold">
                    {dpf.status.soot_loading_percent !== null
                      ? `${dpf.status.soot_loading_percent.toFixed(1)}%`
                      : "---"
                    }
                  </span>
                </div>
                {dpf.status.soot_loading_percent !== null && (
                  <Progress
                    value={dpf.status.soot_loading_percent}
                    variant={getSootVariant(dpf.status.soot_loading_percent)}
                    size="md"
                  />
                )}
              </div>

              {/* Ash Loading */}
              <StatusCard
                icon={Wind}
                label="Ash"
                value={dpf.status.ash_loading_grams !== null ? `${dpf.status.ash_loading_grams.toFixed(0)} g` : "---"}
              />

              {/* Regen Count */}
              <StatusCard
                icon={Flame}
                label="Regenerations"
                value={dpf.status.regen_count !== null ? dpf.status.regen_count.toString() : "---"}
              />

              {/* Temp Before DPF */}
              <StatusCard
                icon={Thermometer}
                label="Inlet Temp"
                value={dpf.status.temp_before_dpf !== null ? `${dpf.status.temp_before_dpf.toFixed(0)}°C` : "---"}
              />

              {/* Temp After DPF */}
              <StatusCard
                icon={Thermometer}
                label="Outlet Temp"
                value={dpf.status.temp_after_dpf !== null ? `${dpf.status.temp_after_dpf.toFixed(0)}°C` : "---"}
              />

              {/* Differential Pressure */}
              <StatusCard
                icon={Gauge}
                label="Diff. Pressure"
                value={dpf.status.differential_pressure_mbar !== null ? `${dpf.status.differential_pressure_mbar.toFixed(1)} mbar` : "---"}
              />

              {/* Distance Since Regen */}
              <StatusCard
                icon={RefreshCw}
                label="Since Regen"
                value={dpf.status.distance_since_regen_km !== null ? `${dpf.status.distance_since_regen_km.toFixed(0)} km` : "---"}
              />
            </div>
          ) : (
            <EmptyState
              variant="empty"
              title="No Data"
              description="Click 'Read' to get DPF status"
            />
          )}
        </CardContent>
      </Card>

      {/* Reset Functions */}
      <Card variant="elevated" padding="md" className="animate-fade-in-up" style={{ animationDelay: "100ms" }}>
        <CardHeader>
          <CardTitle>
            <RotateCcw className="h-5 w-5 text-cyan-400" />
            Reset Functions
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          <ResetRow
            title="Reset Ash Counter"
            description="Resets the accumulated ash counter"
            icon={Trash2}
            onAction={() => setConfirmAction("ash")}
            loading={dpf.isLoading}
            variant="warning"
          />
          <ResetRow
            title="Reset DPF Model"
            description="Resets learned adaptation values"
            icon={RotateCcw}
            onAction={() => setConfirmAction("learned")}
            loading={dpf.isLoading}
            variant="warning"
          />
          <ResetRow
            title="New DPF Installed"
            description="Register installation of a new DPF"
            icon={PackagePlus}
            onAction={() => setConfirmAction("new")}
            loading={dpf.isLoading}
            variant="success"
          />
        </CardContent>
      </Card>

      {/* Forced Regeneration */}
      <Card variant="default" padding="md" className="border-red-900/50 animate-fade-in-up" style={{ animationDelay: "150ms" }}>
        <CardHeader>
          <CardTitle className="text-red-400">
            <AlertTriangle className="h-5 w-5" />
            Forced Regeneration
          </CardTitle>
        </CardHeader>
        <CardContent>
          <Alert variant="warning" icon={false} className="mb-4">
            <div className="text-xs">
              <strong>WARNING:</strong> Vehicle must be stationary with engine running.
              Exhaust temperatures can exceed 600°C.
            </div>
          </Alert>
          <div className="flex gap-2">
            <Button
              size="sm"
              variant="destructive"
              onClick={() => setConfirmAction("regen")}
              loading={dpf.isLoading}
              disabled={dpf.status?.regen_active ?? false}
              leftIcon={<Flame className="h-4 w-4" />}
            >
              Start Regeneration
            </Button>
            <Button
              size="sm"
              variant="outline"
              onClick={handleStopRegen}
              loading={dpf.isLoading}
              disabled={!(dpf.status?.regen_active ?? false)}
              leftIcon={<Square className="h-4 w-4" />}
            >
              Stop
            </Button>
          </div>
        </CardContent>
      </Card>

      {/* Confirmation Modals */}
      <ConfirmModal
        isOpen={confirmAction === "ash"}
        onClose={() => setConfirmAction(null)}
        onConfirm={() => handleConfirmAction("ash")}
        title="Reset Ash Counter"
        message="Are you sure you want to reset the ash counter? Only do this after DPF cleaning or replacement."
        confirmText="Reset"
        variant="warning"
        loading={dpf.isLoading}
      />
      <ConfirmModal
        isOpen={confirmAction === "learned"}
        onClose={() => setConfirmAction(null)}
        onConfirm={() => handleConfirmAction("learned")}
        title="Reset DPF Model"
        message="Are you sure you want to reset the learned values? The ECU will need to relearn the DPF characteristics."
        confirmText="Reset"
        variant="warning"
        loading={dpf.isLoading}
      />
      <ConfirmModal
        isOpen={confirmAction === "new"}
        onClose={() => setConfirmAction(null)}
        onConfirm={() => handleConfirmAction("new")}
        title="New DPF Installed"
        message="Register a new DPF installation? This will reset all DPF-related counters and adaptations."
        confirmText="Register"
        variant="default"
        loading={dpf.isLoading}
      />
      <ConfirmModal
        isOpen={confirmAction === "regen"}
        onClose={() => setConfirmAction(null)}
        onConfirm={() => handleConfirmAction("regen")}
        title="Start Forced Regeneration"
        message="This will start a forced DPF regeneration. Ensure the vehicle is stationary, engine is running, and area is well ventilated. Exhaust temperatures will be extremely high."
        confirmText="Start Regeneration"
        variant="danger"
        loading={dpf.isLoading}
      />
    </div>
  )
}

// Helper components
function StatusCard({ icon: Icon, label, value }: { icon: typeof Gauge; label: string; value: string }) {
  return (
    <div className="p-3 rounded-lg bg-zinc-900/50 border border-zinc-800 hover:border-zinc-700 transition-colors">
      <div className="flex items-center gap-2 text-xs text-zinc-500 mb-1">
        <Icon className="h-3.5 w-3.5" />
        <span>{label}</span>
      </div>
      <div className="text-lg font-mono font-semibold">{value}</div>
    </div>
  )
}

function ResetRow({
  title,
  description,
  icon: Icon,
  onAction,
  loading,
  variant,
}: {
  title: string
  description: string
  icon: typeof Trash2
  onAction: () => void
  loading: boolean
  variant: "warning" | "success"
}) {
  const buttonClass = variant === "warning"
    ? "border-amber-800/50 text-amber-400 hover:bg-amber-950/50"
    : "border-emerald-800/50 text-emerald-400 hover:bg-emerald-950/50"

  return (
    <div className="flex items-center justify-between p-3 rounded-lg bg-zinc-900/50 border border-zinc-800 hover:border-zinc-700 transition-colors">
      <div>
        <p className="text-sm font-medium">{title}</p>
        <p className="text-xs text-zinc-500">{description}</p>
      </div>
      <Button
        size="sm"
        variant="outline"
        onClick={onAction}
        loading={loading}
        leftIcon={<Icon className="h-4 w-4" />}
        className={buttonClass}
      >
        {variant === "success" ? "Register" : "Reset"}
      </Button>
    </div>
  )
}
