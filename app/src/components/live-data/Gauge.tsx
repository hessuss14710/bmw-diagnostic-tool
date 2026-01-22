import Chart from "react-apexcharts"
import type { ApexOptions } from "apexcharts"

interface GaugeProps {
  value: number
  min: number
  max: number
  unit: string
  label: string
  format?: "temperature" | "rpm" | "percent" | "speed" | "voltage" | "flow" | "pressure"
}

export function Gauge({ value, min, max, unit, label, format }: GaugeProps) {
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
        return "#ef4444" // Hot - red
      case "rpm":
        if (value < 1000) return "#3b82f6"
        if (value < 5000) return "#22c55e"
        return "#ef4444"
      case "percent":
        if (value < 30) return "#ef4444"
        if (value < 70) return "#22c55e"
        return "#3b82f6"
      case "voltage":
        if (value < 11.5) return "#ef4444"
        if (value < 14.5) return "#22c55e"
        return "#f59e0b"
      default:
        return "#3b82f6"
    }
  }

  const options: ApexOptions = {
    chart: {
      type: "radialBar",
      sparkline: {
        enabled: true,
      },
      background: "transparent",
    },
    plotOptions: {
      radialBar: {
        startAngle: -135,
        endAngle: 135,
        hollow: {
          size: "60%",
        },
        track: {
          background: "#27272a",
          strokeWidth: "100%",
        },
        dataLabels: {
          name: {
            show: true,
            fontSize: "12px",
            fontWeight: 400,
            color: "#a1a1aa",
            offsetY: 25,
          },
          value: {
            show: true,
            fontSize: "24px",
            fontWeight: 600,
            color: "#ffffff",
            offsetY: -10,
            formatter: () => `${value.toFixed(format === "voltage" ? 1 : 0)}`,
          },
        },
      },
    },
    fill: {
      colors: [getColor()],
    },
    stroke: {
      lineCap: "round",
    },
    labels: [unit],
  }

  return (
    <div className="flex flex-col items-center">
      <Chart
        options={options}
        series={[percentage]}
        type="radialBar"
        height={180}
        width={180}
      />
      <span className="text-xs text-zinc-400 -mt-2">{label}</span>
    </div>
  )
}
