import { cn } from "@/lib/utils"

interface ProgressProps {
  value: number
  max?: number
  size?: "sm" | "md" | "lg"
  variant?: "default" | "success" | "warning" | "error" | "gradient"
  showValue?: boolean
  label?: string
  className?: string
}

const sizeStyles = {
  sm: "h-1.5",
  md: "h-2.5",
  lg: "h-4",
}

const variantStyles = {
  default: "bg-blue-600",
  success: "bg-emerald-500",
  warning: "bg-amber-500",
  error: "bg-red-500",
  gradient: "bg-gradient-to-r from-blue-600 via-blue-500 to-cyan-500",
}

export function Progress({
  value,
  max = 100,
  size = "md",
  variant = "default",
  showValue = false,
  label,
  className,
}: ProgressProps) {
  const percentage = Math.min(100, Math.max(0, (value / max) * 100))

  return (
    <div className={cn("w-full", className)}>
      {(label || showValue) && (
        <div className="flex justify-between items-center mb-2">
          {label && <span className="text-sm text-zinc-400">{label}</span>}
          {showValue && (
            <span className="text-sm font-mono text-zinc-300">
              {Math.round(percentage)}%
            </span>
          )}
        </div>
      )}
      <div
        className={cn(
          "w-full bg-zinc-800 rounded-full overflow-hidden",
          sizeStyles[size]
        )}
        role="progressbar"
        aria-valuenow={value}
        aria-valuemin={0}
        aria-valuemax={max}
      >
        <div
          className={cn(
            "h-full rounded-full transition-all duration-500 ease-out",
            variantStyles[variant]
          )}
          style={{ width: `${percentage}%` }}
        />
      </div>
    </div>
  )
}

interface CircularProgressProps {
  value: number
  max?: number
  size?: number
  strokeWidth?: number
  variant?: "default" | "success" | "warning" | "error"
  showValue?: boolean
  label?: string
  className?: string
}

const circularVariantStyles = {
  default: "text-blue-600",
  success: "text-emerald-500",
  warning: "text-amber-500",
  error: "text-red-500",
}

export function CircularProgress({
  value,
  max = 100,
  size = 80,
  strokeWidth = 8,
  variant = "default",
  showValue = true,
  label,
  className,
}: CircularProgressProps) {
  const percentage = Math.min(100, Math.max(0, (value / max) * 100))
  const radius = (size - strokeWidth) / 2
  const circumference = radius * 2 * Math.PI
  const offset = circumference - (percentage / 100) * circumference

  return (
    <div className={cn("inline-flex flex-col items-center gap-2", className)}>
      <div className="relative" style={{ width: size, height: size }}>
        <svg className="transform -rotate-90" width={size} height={size}>
          {/* Background circle */}
          <circle
            className="text-zinc-800"
            strokeWidth={strokeWidth}
            stroke="currentColor"
            fill="transparent"
            r={radius}
            cx={size / 2}
            cy={size / 2}
          />
          {/* Progress circle */}
          <circle
            className={cn("transition-all duration-500 ease-out", circularVariantStyles[variant])}
            strokeWidth={strokeWidth}
            strokeDasharray={circumference}
            strokeDashoffset={offset}
            strokeLinecap="round"
            stroke="currentColor"
            fill="transparent"
            r={radius}
            cx={size / 2}
            cy={size / 2}
          />
        </svg>
        {showValue && (
          <div className="absolute inset-0 flex items-center justify-center">
            <span className="text-lg font-semibold text-white">
              {Math.round(percentage)}%
            </span>
          </div>
        )}
      </div>
      {label && <span className="text-sm text-zinc-400">{label}</span>}
    </div>
  )
}
