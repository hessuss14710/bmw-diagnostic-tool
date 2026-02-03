import { cn } from "@/lib/utils"

type LedStatus = "off" | "success" | "warning" | "error" | "info" | "connecting"

interface LedIndicatorProps {
  status: LedStatus
  size?: "sm" | "md" | "lg"
  label?: string
  pulse?: boolean
  className?: string
}

const sizeStyles = {
  sm: "w-2 h-2",
  md: "w-3 h-3",
  lg: "w-4 h-4",
}

const statusStyles = {
  off: "bg-zinc-600",
  success: "bg-emerald-500 shadow-[0_0_8px_rgba(16,185,129,0.6)]",
  warning: "bg-amber-500 shadow-[0_0_8px_rgba(245,158,11,0.6)]",
  error: "bg-red-500 shadow-[0_0_8px_rgba(239,68,68,0.6)]",
  info: "bg-cyan-500 shadow-[0_0_8px_rgba(6,182,212,0.6)]",
  connecting: "bg-blue-500 shadow-[0_0_8px_rgba(59,130,246,0.6)]",
}

export function LedIndicator({
  status,
  size = "md",
  label,
  pulse = false,
  className,
}: LedIndicatorProps) {
  const shouldPulse = pulse || status === "connecting"

  return (
    <div className={cn("inline-flex items-center gap-2", className)}>
      <span
        className={cn(
          "rounded-full transition-all duration-300",
          sizeStyles[size],
          statusStyles[status],
          shouldPulse && "animate-led-blink"
        )}
        role="status"
        aria-label={`Status: ${status}`}
      />
      {label && (
        <span className="text-sm text-zinc-400">{label}</span>
      )}
    </div>
  )
}

interface LedGroupProps {
  leds: Array<{
    id: string
    label: string
    status: LedStatus
  }>
  orientation?: "horizontal" | "vertical"
  size?: "sm" | "md" | "lg"
  className?: string
}

export function LedGroup({
  leds,
  orientation = "horizontal",
  size = "sm",
  className,
}: LedGroupProps) {
  return (
    <div
      className={cn(
        "flex gap-4",
        orientation === "vertical" ? "flex-col" : "flex-row flex-wrap",
        className
      )}
    >
      {leds.map((led) => (
        <LedIndicator
          key={led.id}
          status={led.status}
          label={led.label}
          size={size}
        />
      ))}
    </div>
  )
}

interface StatusBarProps {
  items: Array<{
    label: string
    value: string | number
    status?: LedStatus
    unit?: string
  }>
  className?: string
}

export function StatusBar({ items, className }: StatusBarProps) {
  return (
    <div
      className={cn(
        "flex flex-wrap gap-4 p-3 bg-zinc-900/50 rounded-lg border border-zinc-800",
        className
      )}
    >
      {items.map((item, index) => (
        <div key={index} className="flex items-center gap-2">
          {item.status && <LedIndicator status={item.status} size="sm" />}
          <span className="text-xs text-zinc-500">{item.label}:</span>
          <span className="text-sm font-mono text-white">
            {item.value}
            {item.unit && <span className="text-zinc-500 ml-1">{item.unit}</span>}
          </span>
        </div>
      ))}
    </div>
  )
}
