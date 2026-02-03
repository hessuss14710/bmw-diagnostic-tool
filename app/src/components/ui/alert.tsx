import { forwardRef, type HTMLAttributes } from "react"
import { cva, type VariantProps } from "class-variance-authority"
import { CheckCircle, Info, XCircle, AlertTriangle, X } from "lucide-react"
import { cn } from "@/lib/utils"

const alertVariants = cva(
  "relative rounded-lg border p-4 animate-fade-in-up",
  {
    variants: {
      variant: {
        default: "bg-zinc-900 border-zinc-800 text-zinc-300",
        info: "bg-cyan-950/50 border-cyan-800/50 text-cyan-200",
        success: "bg-emerald-950/50 border-emerald-800/50 text-emerald-200",
        warning: "bg-amber-950/50 border-amber-800/50 text-amber-200",
        error: "bg-red-950/50 border-red-800/50 text-red-200",
      },
    },
    defaultVariants: {
      variant: "default",
    },
  }
)

const iconMap = {
  default: Info,
  info: Info,
  success: CheckCircle,
  warning: AlertTriangle,
  error: XCircle,
}

const iconColors = {
  default: "text-zinc-400",
  info: "text-cyan-400",
  success: "text-emerald-400",
  warning: "text-amber-400",
  error: "text-red-400",
}

export interface AlertProps
  extends HTMLAttributes<HTMLDivElement>,
    VariantProps<typeof alertVariants> {
  title?: string
  onClose?: () => void
  icon?: boolean
}

const Alert = forwardRef<HTMLDivElement, AlertProps>(
  ({ className, variant = "default", title, onClose, icon = true, children, ...props }, ref) => {
    const IconComponent = iconMap[variant || "default"]

    return (
      <div
        ref={ref}
        role="alert"
        className={cn(alertVariants({ variant }), className)}
        {...props}
      >
        <div className="flex gap-3">
          {icon && (
            <IconComponent className={cn("h-5 w-5 shrink-0 mt-0.5", iconColors[variant || "default"])} />
          )}
          <div className="flex-1 min-w-0">
            {title && (
              <h5 className="font-semibold text-white mb-1">{title}</h5>
            )}
            <div className="text-sm">{children}</div>
          </div>
          {onClose && (
            <button
              onClick={onClose}
              className="shrink-0 p-1 rounded-md hover:bg-white/10 transition-colors"
              aria-label="Dismiss"
            >
              <X className="h-4 w-4" />
            </button>
          )}
        </div>
      </div>
    )
  }
)
Alert.displayName = "Alert"

export { Alert, alertVariants }
