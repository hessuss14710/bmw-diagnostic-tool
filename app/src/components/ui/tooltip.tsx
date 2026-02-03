import { useState, useRef, useEffect, type ReactNode } from "react"
import { cn } from "@/lib/utils"

interface TooltipProps {
  content: ReactNode
  children: ReactNode
  position?: "top" | "bottom" | "left" | "right"
  delay?: number
  className?: string
}

export function Tooltip({
  content,
  children,
  position = "top",
  delay = 200,
  className,
}: TooltipProps) {
  const [isVisible, setIsVisible] = useState(false)
  const [coords, setCoords] = useState({ x: 0, y: 0 })
  const triggerRef = useRef<HTMLDivElement>(null)
  const tooltipRef = useRef<HTMLDivElement>(null)
  const timeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null)

  const showTooltip = () => {
    timeoutRef.current = setTimeout(() => {
      setIsVisible(true)
    }, delay)
  }

  const hideTooltip = () => {
    if (timeoutRef.current !== null) {
      clearTimeout(timeoutRef.current)
    }
    setIsVisible(false)
  }

  useEffect(() => {
    if (isVisible && triggerRef.current && tooltipRef.current) {
      const triggerRect = triggerRef.current.getBoundingClientRect()
      const tooltipRect = tooltipRef.current.getBoundingClientRect()

      let x = 0
      let y = 0

      switch (position) {
        case "top":
          x = triggerRect.left + triggerRect.width / 2 - tooltipRect.width / 2
          y = triggerRect.top - tooltipRect.height - 8
          break
        case "bottom":
          x = triggerRect.left + triggerRect.width / 2 - tooltipRect.width / 2
          y = triggerRect.bottom + 8
          break
        case "left":
          x = triggerRect.left - tooltipRect.width - 8
          y = triggerRect.top + triggerRect.height / 2 - tooltipRect.height / 2
          break
        case "right":
          x = triggerRect.right + 8
          y = triggerRect.top + triggerRect.height / 2 - tooltipRect.height / 2
          break
      }

      // Keep tooltip within viewport
      x = Math.max(8, Math.min(x, window.innerWidth - tooltipRect.width - 8))
      y = Math.max(8, Math.min(y, window.innerHeight - tooltipRect.height - 8))

      setCoords({ x, y })
    }
  }, [isVisible, position])

  useEffect(() => {
    return () => {
      if (timeoutRef.current !== null) {
        clearTimeout(timeoutRef.current)
      }
    }
  }, [])

  const arrowStyles = {
    top: "bottom-[-4px] left-1/2 -translate-x-1/2 border-l-transparent border-r-transparent border-b-transparent border-t-zinc-700",
    bottom: "top-[-4px] left-1/2 -translate-x-1/2 border-l-transparent border-r-transparent border-t-transparent border-b-zinc-700",
    left: "right-[-4px] top-1/2 -translate-y-1/2 border-t-transparent border-b-transparent border-r-transparent border-l-zinc-700",
    right: "left-[-4px] top-1/2 -translate-y-1/2 border-t-transparent border-b-transparent border-l-transparent border-r-zinc-700",
  }

  return (
    <>
      <div
        ref={triggerRef}
        onMouseEnter={showTooltip}
        onMouseLeave={hideTooltip}
        onFocus={showTooltip}
        onBlur={hideTooltip}
        className="inline-block"
      >
        {children}
      </div>
      {isVisible && (
        <div
          ref={tooltipRef}
          className={cn(
            "fixed z-[var(--z-tooltip)] px-3 py-2 text-sm bg-zinc-800 border border-zinc-700 text-zinc-200 rounded-lg shadow-xl animate-fade-in max-w-xs",
            className
          )}
          style={{
            left: coords.x,
            top: coords.y,
          }}
          role="tooltip"
        >
          {content}
          <div
            className={cn(
              "absolute w-0 h-0 border-4",
              arrowStyles[position]
            )}
          />
        </div>
      )}
    </>
  )
}

export function TooltipInfo({ text }: { text: string }) {
  return (
    <Tooltip content={text}>
      <span className="inline-flex items-center justify-center w-4 h-4 text-xs text-zinc-500 hover:text-zinc-300 cursor-help transition-colors">
        ?
      </span>
    </Tooltip>
  )
}
