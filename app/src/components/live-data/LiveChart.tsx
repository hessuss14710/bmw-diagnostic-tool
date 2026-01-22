import Chart from "react-apexcharts"
import type { ApexOptions } from "apexcharts"
import type { PidHistory, PidDefinition } from "@/hooks/useLiveData"

interface LiveChartProps {
  history: PidHistory
  definition: PidDefinition
}

export function LiveChart({ history, definition }: LiveChartProps) {
  const data = history.values.map((v) => ({
    x: v.timestamp,
    y: v.value,
  }))

  const options: ApexOptions = {
    chart: {
      type: "line",
      background: "transparent",
      toolbar: {
        show: false,
      },
      animations: {
        enabled: true,
        dynamicAnimation: {
          speed: 200,
        },
      },
      zoom: {
        enabled: false,
      },
    },
    stroke: {
      curve: "smooth",
      width: 2,
    },
    colors: ["#3b82f6"],
    grid: {
      borderColor: "#27272a",
      strokeDashArray: 3,
    },
    xaxis: {
      type: "datetime",
      labels: {
        show: false,
      },
      axisBorder: {
        show: false,
      },
      axisTicks: {
        show: false,
      },
    },
    yaxis: {
      min: definition.min,
      max: definition.max,
      labels: {
        style: {
          colors: "#a1a1aa",
          fontSize: "10px",
        },
        formatter: (val) => val.toFixed(0),
      },
    },
    tooltip: {
      enabled: true,
      theme: "dark",
      x: {
        format: "HH:mm:ss",
      },
      y: {
        formatter: (val) => `${val.toFixed(1)} ${definition.unit}`,
      },
    },
    dataLabels: {
      enabled: false,
    },
  }

  const series = [
    {
      name: definition.short_name,
      data,
    },
  ]

  return (
    <div className="rounded-lg border border-zinc-800 bg-zinc-950 p-3">
      <div className="flex items-center justify-between mb-2">
        <span className="text-sm font-medium text-white">{definition.name}</span>
        <span className="text-xs text-zinc-400">
          {data.length > 0
            ? `${data[data.length - 1].y.toFixed(1)} ${definition.unit}`
            : "-"}
        </span>
      </div>
      <Chart options={options} series={series} type="line" height={120} />
    </div>
  )
}
