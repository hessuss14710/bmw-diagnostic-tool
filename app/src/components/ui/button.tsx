import * as React from "react"
import { cva, type VariantProps } from "class-variance-authority"
import { Loader2 } from "lucide-react"
import { cn } from "@/lib/utils"

const buttonVariants = cva(
  "inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-lg text-sm font-medium transition-all duration-200 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-500 focus-visible:ring-offset-2 focus-visible:ring-offset-zinc-900 disabled:pointer-events-none disabled:opacity-50 [&_svg]:pointer-events-none [&_svg]:size-4 [&_svg]:shrink-0",
  {
    variants: {
      variant: {
        default:
          "bg-blue-600 text-white shadow-lg shadow-blue-600/25 hover:bg-blue-500 active:bg-blue-700",
        destructive:
          "bg-red-600 text-white shadow-lg shadow-red-600/25 hover:bg-red-500 active:bg-red-700",
        warning:
          "bg-amber-600 text-white shadow-lg shadow-amber-600/25 hover:bg-amber-500 active:bg-amber-700",
        success:
          "bg-emerald-600 text-white shadow-lg shadow-emerald-600/25 hover:bg-emerald-500 active:bg-emerald-700",
        outline:
          "border border-zinc-700 bg-transparent text-zinc-300 hover:bg-zinc-800 hover:text-white hover:border-zinc-600",
        secondary:
          "bg-zinc-800 text-zinc-300 hover:bg-zinc-700 hover:text-white",
        ghost:
          "text-zinc-400 hover:bg-zinc-800 hover:text-white",
        link:
          "text-blue-400 underline-offset-4 hover:underline hover:text-blue-300",
        bmw:
          "bg-gradient-to-r from-blue-700 to-blue-600 text-white shadow-lg shadow-blue-600/30 hover:from-blue-600 hover:to-blue-500 active:from-blue-800 active:to-blue-700",
      },
      size: {
        default: "h-10 px-4 py-2",
        sm: "h-8 px-3 text-xs",
        lg: "h-12 px-6 text-base",
        xl: "h-14 px-8 text-lg",
        icon: "h-10 w-10",
        "icon-sm": "h-8 w-8",
        "icon-lg": "h-12 w-12",
      },
    },
    defaultVariants: {
      variant: "default",
      size: "default",
    },
  }
)

export interface ButtonProps
  extends React.ButtonHTMLAttributes<HTMLButtonElement>,
    VariantProps<typeof buttonVariants> {
  loading?: boolean
  leftIcon?: React.ReactNode
  rightIcon?: React.ReactNode
}

const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
  ({ className, variant, size, loading, leftIcon, rightIcon, children, disabled, ...props }, ref) => {
    return (
      <button
        className={cn(buttonVariants({ variant, size, className }))}
        ref={ref}
        disabled={disabled || loading}
        {...props}
      >
        {loading ? (
          <Loader2 className="animate-spin" />
        ) : leftIcon ? (
          leftIcon
        ) : null}
        {children}
        {!loading && rightIcon}
      </button>
    )
  }
)
Button.displayName = "Button"

// eslint-disable-next-line react-refresh/only-export-components
export { Button, buttonVariants }
