import Chart from "react-apexcharts"
import type { ApexOptions } from "apexcharts"
import { cn } from "@/lib/utils"

interface GaugeProps {
  value: number
  min: number
  max: number
  unit: string
  label: string
  format?: "temperature" | "rpm" | "percent" | "speed" | "voltage" | "flow" | "pressure"
  size?: "sm" | "md" | "lg"
  className?: string
}

export function Gauge({ value, min, max, unit, label, format, size = "md", className }: GaugeProps) {
  // Calculate percentage for gauge
  const range = max - min
  const normalizedValue = Math.max(min, Math.min(max, value))
  const percentage = ((normalizedValue - min) / range) * 100

  // Get color based on format and value
  const getColor = () => {
    switch (format) {
      case "temperature":
        if (value < 60) return "#3b82f6" // Cold - blue
        if (value < 100) return "#22c55e" // Normal - green
        if (value < 110) return "#f59e0b" // Warm - amber
        return "#ef4444" // Hot - red
      case "rpm":
        if (value < 1000) return "#3b82f6"
        if (value < 5000) return "#22c55e"
        if (value < 6500) return "#f59e0b"
        return "#ef4444"
      case "percent":
        if (value < 30) return "#ef4444"
        if (value < 70) return "#22c55e"
        return "#3b82f6"
      case "voltage":
        if (value < 11.5) return "#ef4444"
        if (value < 14.5) return "#22c55e"
        return "#f59e0b"
      case "pressure":
        if (value < 0.5) return "#3b82f6"
        if (value < 2.5) return "#22c55e"
        return "#ef4444"
      default:
        return "#0066B1" // BMW Blue
    }
  }

  const sizeConfig = {
    sm: { height: 120, width: 120, valueSize: "18px", labelSize: "10px" },
    md: { height: 160, width: 160, valueSize: "22px", labelSize: "11px" },
    lg: { height: 200, width: 200, valueSize: "28px", labelSize: "12px" },
  }

  const config = sizeConfig[size]

  const options: ApexOptions = {
    chart: {
      type: "radialBar",
      sparkline: {
        enabled: true,
      },
      background: "transparent",
      animations: {
        enabled: true,
        speed: 500,
        dynamicAnimation: {
          enabled: true,
          speed: 200,
        },
      },
    },
    plotOptions: {
      radialBar: {
        startAngle: -135,
        endAngle: 135,
        hollow: {
          size: "58%",
          background: "transparent",
        },
        track: {
          background: "#1f1f23",
          strokeWidth: "100%",
          margin: 0,
          dropShadow: {
            enabled: true,
            top: 2,
            left: 0,
            blur: 4,
            opacity: 0.3,
          },
        },
        dataLabels: {
          name: {
            show: true,
            fontSize: config.labelSize,
            fontWeight: 500,
            color: "#71717a",
            offsetY: 22,
          },
          value: {
            show: true,
            fontSize: config.valueSize,
            fontWeight: 700,
            color: "#ffffff",
            offsetY: -8,
            formatter: () => `${value.toFixed(format === "voltage" ? 1 : 0)}`,
          },
        },
      },
    },
    fill: {
      type: "gradient",
      gradient: {
        shade: "dark",
        type: "horizontal",
        shadeIntensity: 0.5,
        gradientToColors: [getColor()],
        inverseColors: false,
        opacityFrom: 1,
        opacityTo: 1,
        stops: [0, 100],
      },
    },
    stroke: {
      lineCap: "round",
    },
    labels: [unit],
  }

  return (
    <div className={cn(
      "flex flex-col items-center p-2 rounded-xl bg-zinc-900/30 border border-zinc-800/50 transition-all duration-300 hover:border-zinc-700/50 hover:bg-zinc-900/50",
      className
    )}>
      <Chart
        options={options}
        series={[percentage]}
        type="radialBar"
        height={config.height}
        width={config.width}
      />
      <span className="text-xs text-zinc-400 font-medium -mt-1 text-center max-w-[100px] truncate">
        {label}
      </span>
    </div>
  )
}
