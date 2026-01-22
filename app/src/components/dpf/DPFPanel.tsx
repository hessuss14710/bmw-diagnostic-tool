import { useState } from "react"
import { Button } from "@/components/ui/button"
import { useDPF } from "@/hooks/useDPF"
import type { EcuInfo } from "@/hooks/useBMW"
import {
  Loader2,
  RefreshCw,
  Trash2,
  Flame,
  Square,
  ShieldCheck,
  ShieldX,
  CheckCircle2,
  XCircle,
  AlertTriangle,
  Gauge,
  Thermometer,
  Wind,
  RotateCcw,
  PackagePlus,
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

  // Check if DDE is selected
  const isDDE = selectedEcu?.id === "DDE" || selectedEcu?.id === "DME"

  const handleStartSession = async () => {
    try {
      await dpf.startExtendedSession(targetAddress)
    } catch {
      // Error already set in hook
    }
  }

  const handleSecurityAccess = async () => {
    try {
      await dpf.securityAccess(targetAddress)
    } catch {
      // Error already set in hook
    }
  }

  const handleReadStatus = async () => {
    try {
      await dpf.readStatus(targetAddress)
    } catch {
      // Error already set in hook
    }
  }

  const handleResetAsh = async () => {
    if (confirmAction !== "ash") {
      setConfirmAction("ash")
      return
    }
    setConfirmAction(null)
    try {
      await dpf.resetAsh(targetAddress)
    } catch {
      // Error already set in hook
    }
  }

  const handleResetLearned = async () => {
    if (confirmAction !== "learned") {
      setConfirmAction("learned")
      return
    }
    setConfirmAction(null)
    try {
      await dpf.resetLearned(targetAddress)
    } catch {
      // Error already set in hook
    }
  }

  const handleNewDPF = async () => {
    if (confirmAction !== "new") {
      setConfirmAction("new")
      return
    }
    setConfirmAction(null)
    try {
      await dpf.newDpfInstalled(targetAddress)
    } catch {
      // Error already set in hook
    }
  }

  const handleStartRegen = async () => {
    if (confirmAction !== "regen") {
      setConfirmAction("regen")
      return
    }
    setConfirmAction(null)
    try {
      await dpf.startRegen(targetAddress)
    } catch {
      // Error already set in hook
    }
  }

  const handleStopRegen = async () => {
    try {
      await dpf.stopRegen(targetAddress)
    } catch {
      // Error already set in hook
    }
  }

  const cancelConfirm = () => {
    setConfirmAction(null)
  }

  if (!isConnected) {
    return (
      <div className="rounded-lg border border-zinc-800 bg-zinc-950 p-6">
        <div className="flex flex-col items-center justify-center py-4 text-zinc-500">
          <XCircle className="h-12 w-12 text-zinc-600 mb-2" />
          <p className="text-sm">Conecta el cable K+DCAN primero</p>
        </div>
      </div>
    )
  }

  if (!isInitialized) {
    return (
      <div className="rounded-lg border border-zinc-800 bg-zinc-950 p-6">
        <div className="flex flex-col items-center justify-center py-4 text-zinc-500">
          <AlertTriangle className="h-12 w-12 text-amber-600 mb-2" />
          <p className="text-sm">Inicializa la comunicacion con la ECU primero</p>
          <p className="text-xs mt-1">Selecciona DDE en la pestana Diagnostics</p>
        </div>
      </div>
    )
  }

  if (!isDDE) {
    return (
      <div className="rounded-lg border border-zinc-800 bg-zinc-950 p-6">
        <div className="flex flex-col items-center justify-center py-4 text-zinc-500">
          <AlertTriangle className="h-12 w-12 text-amber-600 mb-2" />
          <p className="text-sm">Selecciona la ECU DDE (Digital Diesel Electronics)</p>
          <p className="text-xs mt-1">Las funciones DPF solo estan disponibles para motores diesel</p>
        </div>
      </div>
    )
  }

  return (
    <div className="space-y-4">
      {/* Session & Security Status */}
      <div className="rounded-lg border border-zinc-800 bg-zinc-950 p-4">
        <h3 className="text-sm font-semibold mb-3 text-zinc-400">Estado de Sesion</h3>
        <div className="flex flex-wrap gap-2">
          <div className={`flex items-center gap-2 px-3 py-1.5 rounded-full text-xs ${
            dpf.sessionActive
              ? "bg-green-900/30 text-green-400"
              : "bg-zinc-800 text-zinc-500"
          }`}>
            {dpf.sessionActive ? (
              <CheckCircle2 className="h-3.5 w-3.5" />
            ) : (
              <XCircle className="h-3.5 w-3.5" />
            )}
            <span>Sesion Extendida</span>
          </div>
          <div className={`flex items-center gap-2 px-3 py-1.5 rounded-full text-xs ${
            dpf.securityUnlocked
              ? "bg-green-900/30 text-green-400"
              : "bg-zinc-800 text-zinc-500"
          }`}>
            {dpf.securityUnlocked ? (
              <ShieldCheck className="h-3.5 w-3.5" />
            ) : (
              <ShieldX className="h-3.5 w-3.5" />
            )}
            <span>Seguridad</span>
          </div>
        </div>
        <div className="flex gap-2 mt-3">
          <Button
            size="sm"
            variant="outline"
            onClick={handleStartSession}
            disabled={dpf.isLoading || dpf.sessionActive}
            className="border-zinc-700 text-xs"
          >
            {dpf.isLoading ? (
              <Loader2 className="h-3 w-3 animate-spin" />
            ) : (
              <CheckCircle2 className="h-3 w-3" />
            )}
            Iniciar Sesion
          </Button>
          <Button
            size="sm"
            variant="outline"
            onClick={handleSecurityAccess}
            disabled={dpf.isLoading || !dpf.sessionActive || dpf.securityUnlocked}
            className="border-zinc-700 text-xs"
          >
            {dpf.isLoading ? (
              <Loader2 className="h-3 w-3 animate-spin" />
            ) : (
              <ShieldCheck className="h-3 w-3" />
            )}
            Desbloquear
          </Button>
        </div>
      </div>

      {/* Error Display */}
      {dpf.error && (
        <div className="rounded-lg border border-red-900 bg-red-950 p-3 text-sm text-red-400 flex items-start gap-2">
          <XCircle className="h-4 w-4 mt-0.5 flex-shrink-0" />
          <div>
            <p>{dpf.error}</p>
            <button
              onClick={dpf.clearError}
              className="text-xs text-red-500 hover:text-red-400 mt-1"
            >
              Cerrar
            </button>
          </div>
        </div>
      )}

      {/* Last Result */}
      {dpf.lastResult && (
        <div className={`rounded-lg border p-3 text-sm flex items-start gap-2 ${
          dpf.lastResult.success
            ? "border-green-900 bg-green-950 text-green-400"
            : "border-amber-900 bg-amber-950 text-amber-400"
        }`}>
          {dpf.lastResult.success ? (
            <CheckCircle2 className="h-4 w-4 mt-0.5 flex-shrink-0" />
          ) : (
            <AlertTriangle className="h-4 w-4 mt-0.5 flex-shrink-0" />
          )}
          <div>
            <p className="font-medium">
              Rutina 0x{dpf.lastResult.routine_id.toString(16).toUpperCase().padStart(4, '0')}
            </p>
            <p className="text-xs opacity-80">{dpf.lastResult.status}</p>
          </div>
        </div>
      )}

      {/* DPF Status */}
      <div className="rounded-lg border border-zinc-800 bg-zinc-950 p-4">
        <div className="flex items-center justify-between mb-3">
          <h3 className="text-sm font-semibold text-zinc-400">Estado del DPF</h3>
          <Button
            size="sm"
            variant="outline"
            onClick={handleReadStatus}
            disabled={dpf.isLoading}
            className="border-zinc-700 text-xs"
          >
            {dpf.isLoading ? (
              <Loader2 className="h-3 w-3 animate-spin" />
            ) : (
              <RefreshCw className="h-3 w-3" />
            )}
            Leer
          </Button>
        </div>

        {dpf.status ? (
          <div className="grid grid-cols-2 gap-3">
            {/* Soot Loading */}
            <div className="bg-zinc-900 rounded-lg p-3">
              <div className="flex items-center gap-2 text-xs text-zinc-500 mb-1">
                <Gauge className="h-3.5 w-3.5" />
                <span>Carga Hollin</span>
              </div>
              <div className="text-lg font-mono">
                {dpf.status.soot_loading_percent !== null
                  ? `${dpf.status.soot_loading_percent.toFixed(1)}%`
                  : "---"
                }
              </div>
              {dpf.status.soot_loading_percent !== null && (
                <div className="mt-1 h-1.5 bg-zinc-800 rounded-full overflow-hidden">
                  <div
                    className={`h-full rounded-full transition-all ${
                      dpf.status.soot_loading_percent > 80
                        ? "bg-red-500"
                        : dpf.status.soot_loading_percent > 50
                          ? "bg-amber-500"
                          : "bg-green-500"
                    }`}
                    style={{ width: `${Math.min(dpf.status.soot_loading_percent, 100)}%` }}
                  />
                </div>
              )}
            </div>

            {/* Ash Loading */}
            <div className="bg-zinc-900 rounded-lg p-3">
              <div className="flex items-center gap-2 text-xs text-zinc-500 mb-1">
                <Wind className="h-3.5 w-3.5" />
                <span>Cenizas</span>
              </div>
              <div className="text-lg font-mono">
                {dpf.status.ash_loading_grams !== null
                  ? `${dpf.status.ash_loading_grams.toFixed(0)} g`
                  : "---"
                }
              </div>
            </div>

            {/* Temp Before DPF */}
            <div className="bg-zinc-900 rounded-lg p-3">
              <div className="flex items-center gap-2 text-xs text-zinc-500 mb-1">
                <Thermometer className="h-3.5 w-3.5" />
                <span>Temp. Entrada</span>
              </div>
              <div className="text-lg font-mono">
                {dpf.status.temp_before_dpf !== null
                  ? `${dpf.status.temp_before_dpf.toFixed(0)} C`
                  : "---"
                }
              </div>
            </div>

            {/* Temp After DPF */}
            <div className="bg-zinc-900 rounded-lg p-3">
              <div className="flex items-center gap-2 text-xs text-zinc-500 mb-1">
                <Thermometer className="h-3.5 w-3.5" />
                <span>Temp. Salida</span>
              </div>
              <div className="text-lg font-mono">
                {dpf.status.temp_after_dpf !== null
                  ? `${dpf.status.temp_after_dpf.toFixed(0)} C`
                  : "---"
                }
              </div>
            </div>

            {/* Differential Pressure */}
            <div className="bg-zinc-900 rounded-lg p-3">
              <div className="flex items-center gap-2 text-xs text-zinc-500 mb-1">
                <Gauge className="h-3.5 w-3.5" />
                <span>Presion Dif.</span>
              </div>
              <div className="text-lg font-mono">
                {dpf.status.differential_pressure_mbar !== null
                  ? `${dpf.status.differential_pressure_mbar.toFixed(1)} mbar`
                  : "---"
                }
              </div>
            </div>

            {/* Regen Count */}
            <div className="bg-zinc-900 rounded-lg p-3">
              <div className="flex items-center gap-2 text-xs text-zinc-500 mb-1">
                <Flame className="h-3.5 w-3.5" />
                <span>Regeneraciones</span>
              </div>
              <div className="text-lg font-mono">
                {dpf.status.regen_count !== null
                  ? dpf.status.regen_count
                  : "---"
                }
              </div>
            </div>

            {/* Distance Since Regen */}
            <div className="bg-zinc-900 rounded-lg p-3 col-span-2">
              <div className="flex items-center gap-2 text-xs text-zinc-500 mb-1">
                <RefreshCw className="h-3.5 w-3.5" />
                <span>Distancia desde ultima regeneracion</span>
              </div>
              <div className="text-lg font-mono">
                {dpf.status.distance_since_regen_km !== null
                  ? `${dpf.status.distance_since_regen_km.toFixed(0)} km`
                  : "---"
                }
              </div>
            </div>

            {/* Regen Active */}
            {dpf.status.regen_active && (
              <div className="col-span-2 bg-amber-900/30 border border-amber-800 rounded-lg p-3 flex items-center gap-2">
                <Flame className="h-5 w-5 text-amber-500 animate-pulse" />
                <span className="text-amber-400 font-medium">Regeneracion en curso</span>
              </div>
            )}
          </div>
        ) : (
          <div className="text-center py-6 text-zinc-500 text-sm">
            <p>Pulsa "Leer" para obtener el estado del DPF</p>
          </div>
        )}
      </div>

      {/* Reset Functions */}
      <div className="rounded-lg border border-zinc-800 bg-zinc-950 p-4">
        <h3 className="text-sm font-semibold mb-3 text-zinc-400">Funciones de Reset</h3>

        <div className="space-y-3">
          {/* Reset Ash */}
          <div className="flex items-center justify-between p-3 bg-zinc-900 rounded-lg">
            <div>
              <p className="text-sm font-medium">Reset Cenizas</p>
              <p className="text-xs text-zinc-500">Reinicia el contador de cenizas acumuladas</p>
            </div>
            {confirmAction === "ash" ? (
              <div className="flex gap-2">
                <Button
                  size="sm"
                  onClick={handleResetAsh}
                  disabled={dpf.isLoading}
                  className="bg-red-600 hover:bg-red-700 text-xs"
                >
                  Confirmar
                </Button>
                <Button
                  size="sm"
                  variant="outline"
                  onClick={cancelConfirm}
                  className="border-zinc-700 text-xs"
                >
                  Cancelar
                </Button>
              </div>
            ) : (
              <Button
                size="sm"
                variant="outline"
                onClick={handleResetAsh}
                disabled={dpf.isLoading}
                className="border-amber-800 text-amber-400 hover:bg-amber-950 text-xs"
              >
                {dpf.isLoading ? (
                  <Loader2 className="h-3 w-3 animate-spin" />
                ) : (
                  <Trash2 className="h-3 w-3" />
                )}
                Reset
              </Button>
            )}
          </div>

          {/* Reset Learned Values */}
          <div className="flex items-center justify-between p-3 bg-zinc-900 rounded-lg">
            <div>
              <p className="text-sm font-medium">Reset Modelo DPF</p>
              <p className="text-xs text-zinc-500">Reinicia los valores de adaptacion aprendidos</p>
            </div>
            {confirmAction === "learned" ? (
              <div className="flex gap-2">
                <Button
                  size="sm"
                  onClick={handleResetLearned}
                  disabled={dpf.isLoading}
                  className="bg-red-600 hover:bg-red-700 text-xs"
                >
                  Confirmar
                </Button>
                <Button
                  size="sm"
                  variant="outline"
                  onClick={cancelConfirm}
                  className="border-zinc-700 text-xs"
                >
                  Cancelar
                </Button>
              </div>
            ) : (
              <Button
                size="sm"
                variant="outline"
                onClick={handleResetLearned}
                disabled={dpf.isLoading}
                className="border-amber-800 text-amber-400 hover:bg-amber-950 text-xs"
              >
                {dpf.isLoading ? (
                  <Loader2 className="h-3 w-3 animate-spin" />
                ) : (
                  <RotateCcw className="h-3 w-3" />
                )}
                Reset
              </Button>
            )}
          </div>

          {/* New DPF Installed */}
          <div className="flex items-center justify-between p-3 bg-zinc-900 rounded-lg">
            <div>
              <p className="text-sm font-medium">DPF Nuevo Instalado</p>
              <p className="text-xs text-zinc-500">Registra la instalacion de un DPF nuevo</p>
            </div>
            {confirmAction === "new" ? (
              <div className="flex gap-2">
                <Button
                  size="sm"
                  onClick={handleNewDPF}
                  disabled={dpf.isLoading}
                  className="bg-red-600 hover:bg-red-700 text-xs"
                >
                  Confirmar
                </Button>
                <Button
                  size="sm"
                  variant="outline"
                  onClick={cancelConfirm}
                  className="border-zinc-700 text-xs"
                >
                  Cancelar
                </Button>
              </div>
            ) : (
              <Button
                size="sm"
                variant="outline"
                onClick={handleNewDPF}
                disabled={dpf.isLoading}
                className="border-green-800 text-green-400 hover:bg-green-950 text-xs"
              >
                {dpf.isLoading ? (
                  <Loader2 className="h-3 w-3 animate-spin" />
                ) : (
                  <PackagePlus className="h-3 w-3" />
                )}
                Registrar
              </Button>
            )}
          </div>
        </div>
      </div>

      {/* Forced Regeneration */}
      <div className="rounded-lg border border-red-900/50 bg-zinc-950 p-4">
        <div className="flex items-center gap-2 mb-3">
          <AlertTriangle className="h-4 w-4 text-red-500" />
          <h3 className="text-sm font-semibold text-red-400">Regeneracion Forzada</h3>
        </div>
        <p className="text-xs text-zinc-500 mb-3">
          ADVERTENCIA: El vehiculo debe estar parado con el motor en marcha.
          Las temperaturas del escape pueden superar los 600C.
        </p>

        <div className="flex gap-2">
          {confirmAction === "regen" ? (
            <>
              <Button
                size="sm"
                onClick={handleStartRegen}
                disabled={dpf.isLoading}
                className="bg-red-600 hover:bg-red-700 text-xs"
              >
                <Flame className="h-3 w-3" />
                Confirmar Inicio
              </Button>
              <Button
                size="sm"
                variant="outline"
                onClick={cancelConfirm}
                className="border-zinc-700 text-xs"
              >
                Cancelar
              </Button>
            </>
          ) : (
            <>
              <Button
                size="sm"
                onClick={handleStartRegen}
                disabled={dpf.isLoading || (dpf.status?.regen_active ?? false)}
                className="bg-red-900 hover:bg-red-800 text-xs"
              >
                {dpf.isLoading ? (
                  <Loader2 className="h-3 w-3 animate-spin" />
                ) : (
                  <Flame className="h-3 w-3" />
                )}
                Iniciar Regeneracion
              </Button>
              <Button
                size="sm"
                variant="outline"
                onClick={handleStopRegen}
                disabled={dpf.isLoading || !(dpf.status?.regen_active ?? false)}
                className="border-zinc-700 text-xs"
              >
                <Square className="h-3 w-3" />
                Detener
              </Button>
            </>
          )}
        </div>
      </div>
    </div>
  )
}
