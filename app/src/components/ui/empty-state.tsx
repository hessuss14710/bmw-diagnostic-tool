import { type ReactNode } from "react"
import { cn } from "@/lib/utils"
import { Button } from "./button"
import {
  FileQuestion,
  Search,
  WifiOff,
  AlertCircle,
  Inbox,
  type LucideIcon,
} from "lucide-react"

type EmptyStateVariant = "default" | "search" | "offline" | "error" | "empty"

interface EmptyStateProps {
  variant?: EmptyStateVariant
  icon?: LucideIcon
  title: string
  description?: string
  action?: {
    label: string
    onClick: () => void
    variant?: "default" | "outline" | "ghost"
  }
  className?: string
  children?: ReactNode
}

const variantIcons: Record<EmptyStateVariant, LucideIcon> = {
  default: FileQuestion,
  search: Search,
  offline: WifiOff,
  error: AlertCircle,
  empty: Inbox,
}

const variantColors: Record<EmptyStateVariant, string> = {
  default: "text-zinc-500 bg-zinc-800/30",
  search: "text-blue-400 bg-blue-900/20",
  offline: "text-amber-400 bg-amber-900/20",
  error: "text-red-400 bg-red-900/20",
  empty: "text-zinc-500 bg-zinc-800/30",
}

export function EmptyState({
  variant = "default",
  icon,
  title,
  description,
  action,
  className,
  children,
}: EmptyStateProps) {
  const Icon = icon || variantIcons[variant]

  return (
    <div
      className={cn(
        "flex flex-col items-center justify-center py-12 px-6 text-center animate-fade-in",
        className
      )}
    >
      <div className={cn("p-4 rounded-full mb-4", variantColors[variant])}>
        <Icon className="h-8 w-8" />
      </div>
      <h3 className="text-lg font-medium text-zinc-200 mb-1">{title}</h3>
      {description && (
        <p className="text-sm text-zinc-500 max-w-sm mb-4">{description}</p>
      )}
      {action && (
        <Button
          variant={action.variant || "outline"}
          size="sm"
          onClick={action.onClick}
        >
          {action.label}
        </Button>
      )}
      {children}
    </div>
  )
}

export function NoConnection({ onRetry }: { onRetry?: () => void }) {
  return (
    <EmptyState
      variant="offline"
      title="No Connection"
      description="Connect to your K+DCAN cable to start diagnostics"
      action={onRetry ? { label: "Retry Connection", onClick: onRetry } : undefined}
    />
  )
}

export function NoData({ message = "No data available" }: { message?: string }) {
  return (
    <EmptyState
      variant="empty"
      title="No Data"
      description={message}
    />
  )
}

export function NoResults({ query, onClear }: { query?: string; onClear?: () => void }) {
  return (
    <EmptyState
      variant="search"
      title="No Results Found"
      description={query ? `No results for "${query}"` : "Try adjusting your search"}
      action={onClear ? { label: "Clear Search", onClick: onClear } : undefined}
    />
  )
}
