import { createContext, useContext, useState, type ReactNode } from "react"
import { cn } from "@/lib/utils"

interface TabsContextType {
  activeTab: string
  setActiveTab: (id: string) => void
}

const TabsContext = createContext<TabsContextType | null>(null)

function useTabs() {
  const context = useContext(TabsContext)
  if (!context) {
    throw new Error("Tabs components must be used within a Tabs provider")
  }
  return context
}

interface TabsProps {
  defaultValue: string
  value?: string
  onChange?: (value: string) => void
  children: ReactNode
  className?: string
}

export function Tabs({
  defaultValue,
  value,
  onChange,
  children,
  className,
}: TabsProps) {
  const [internalValue, setInternalValue] = useState(defaultValue)
  const activeTab = value ?? internalValue

  const setActiveTab = (id: string) => {
    if (!value) {
      setInternalValue(id)
    }
    onChange?.(id)
  }

  return (
    <TabsContext.Provider value={{ activeTab, setActiveTab }}>
      <div className={className}>{children}</div>
    </TabsContext.Provider>
  )
}

interface TabsListProps {
  children: ReactNode
  className?: string
}

export function TabsList({ children, className }: TabsListProps) {
  return (
    <div
      className={cn(
        "scroll-container flex gap-1 p-1 bg-zinc-900/50 rounded-xl border border-zinc-800",
        className
      )}
      role="tablist"
    >
      {children}
    </div>
  )
}

interface TabsTriggerProps {
  value: string
  children: ReactNode
  icon?: ReactNode
  disabled?: boolean
  className?: string
}

export function TabsTrigger({
  value,
  children,
  icon,
  disabled = false,
  className,
}: TabsTriggerProps) {
  const { activeTab, setActiveTab } = useTabs()
  const isActive = activeTab === value

  return (
    <button
      role="tab"
      aria-selected={isActive}
      aria-controls={`panel-${value}`}
      disabled={disabled}
      onClick={() => setActiveTab(value)}
      className={cn(
        "relative flex items-center gap-2 px-4 py-2.5 text-sm font-medium rounded-lg whitespace-nowrap transition-all duration-200",
        "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-500 focus-visible:ring-offset-2 focus-visible:ring-offset-zinc-900",
        isActive
          ? "bg-blue-600 text-white shadow-lg shadow-blue-600/25"
          : "text-zinc-400 hover:text-white hover:bg-zinc-800",
        disabled && "opacity-50 cursor-not-allowed",
        className
      )}
    >
      {icon && <span className="shrink-0">{icon}</span>}
      <span>{children}</span>
      {isActive && (
        <span className="absolute inset-0 rounded-lg ring-1 ring-inset ring-white/10" />
      )}
    </button>
  )
}

interface TabsContentProps {
  value: string
  children: ReactNode
  className?: string
  forceMount?: boolean
}

export function TabsContent({
  value,
  children,
  className,
  forceMount = false,
}: TabsContentProps) {
  const { activeTab } = useTabs()
  const isActive = activeTab === value

  if (!isActive && !forceMount) {
    return null
  }

  return (
    <div
      role="tabpanel"
      id={`panel-${value}`}
      aria-labelledby={value}
      hidden={!isActive}
      className={cn(
        "mt-4 animate-fade-in-up",
        !isActive && "hidden",
        className
      )}
    >
      {children}
    </div>
  )
}
