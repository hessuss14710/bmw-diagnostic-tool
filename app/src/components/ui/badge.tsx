import { forwardRef, type HTMLAttributes } from "react"
import { cva, type VariantProps } from "class-variance-authority"
import { cn } from "@/lib/utils"

const badgeVariants = cva(
  "inline-flex items-center gap-1 rounded-full font-medium transition-colors",
  {
    variants: {
      variant: {
        default: "bg-zinc-800 text-zinc-300",
        primary: "bg-blue-600/20 text-blue-400 border border-blue-600/30",
        success: "bg-emerald-600/20 text-emerald-400 border border-emerald-600/30",
        warning: "bg-amber-600/20 text-amber-400 border border-amber-600/30",
        error: "bg-red-600/20 text-red-400 border border-red-600/30",
        info: "bg-cyan-600/20 text-cyan-400 border border-cyan-600/30",
        purple: "bg-purple-600/20 text-purple-400 border border-purple-600/30",
        outline: "border border-zinc-700 text-zinc-400",
      },
      size: {
        sm: "text-xs px-2 py-0.5",
        md: "text-xs px-2.5 py-1",
        lg: "text-sm px-3 py-1",
      },
      pulse: {
        true: "animate-pulse",
        false: "",
      },
    },
    defaultVariants: {
      variant: "default",
      size: "md",
      pulse: false,
    },
  }
)

export interface BadgeProps
  extends HTMLAttributes<HTMLSpanElement>,
    VariantProps<typeof badgeVariants> {
  dot?: boolean
  dotColor?: "success" | "warning" | "error" | "info" | "default"
}

const Badge = forwardRef<HTMLSpanElement, BadgeProps>(
  ({ className, variant, size, pulse, dot, dotColor = "default", children, ...props }, ref) => {
    const dotColors = {
      success: "bg-emerald-500",
      warning: "bg-amber-500",
      error: "bg-red-500",
      info: "bg-cyan-500",
      default: "bg-zinc-500",
    }

    return (
      <span
        ref={ref}
        className={cn(badgeVariants({ variant, size, pulse }), className)}
        {...props}
      >
        {dot && (
          <span className={cn("w-1.5 h-1.5 rounded-full", dotColors[dotColor])} />
        )}
        {children}
      </span>
    )
  }
)
Badge.displayName = "Badge"

export { Badge, badgeVariants }
