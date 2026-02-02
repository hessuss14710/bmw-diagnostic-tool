/**
 * Alert Service - Monitors live data and generates alerts based on thresholds
 *
 * Defines warning and critical thresholds for BMW E60 520d diesel parameters.
 */

export interface ParameterThreshold {
  parameter: string
  displayName: string
  unit: string
  warningMin?: number
  warningMax?: number
  criticalMin?: number
  criticalMax?: number
  description: string
}

export interface Alert {
  id: string
  timestamp: number
  type: "warning" | "critical"
  parameter: string
  displayName: string
  value: number
  unit: string
  threshold: number
  thresholdType: "min" | "max"
  message: string
}

// Define thresholds for BMW E60 520d M47N2/N47 diesel
export const DIESEL_THRESHOLDS: ParameterThreshold[] = [
  // Fuel System
  {
    parameter: "fuel_rail_pressure",
    displayName: "Fuel Rail Pressure",
    unit: "bar",
    warningMin: 200,
    warningMax: 1800,
    criticalMin: 150,
    criticalMax: 2000,
    description: "Common rail fuel pressure"
  },

  // Turbo
  {
    parameter: "boost_pressure",
    displayName: "Boost Pressure",
    unit: "mbar",
    warningMax: 2200,
    criticalMax: 2500,
    description: "Turbocharger boost pressure"
  },
  {
    parameter: "vnt_position",
    displayName: "VNT Position",
    unit: "%",
    warningMin: 5,
    warningMax: 95,
    criticalMin: 0,
    criticalMax: 100,
    description: "Variable turbo geometry position"
  },

  // EGR
  {
    parameter: "egr_position",
    displayName: "EGR Position",
    unit: "%",
    warningMax: 90,
    criticalMax: 100,
    description: "EGR valve position (high = stuck open)"
  },

  // Temperatures
  {
    parameter: "coolant_temp",
    displayName: "Coolant Temperature",
    unit: "°C",
    warningMax: 105,
    criticalMax: 115,
    description: "Engine coolant temperature"
  },
  {
    parameter: "oil_temp",
    displayName: "Oil Temperature",
    unit: "°C",
    warningMin: 50,
    warningMax: 130,
    criticalMax: 150,
    description: "Engine oil temperature"
  },
  {
    parameter: "exhaust_temp_dpf_inlet",
    displayName: "DPF Inlet Temperature",
    unit: "°C",
    warningMax: 700,
    criticalMax: 800,
    description: "Exhaust temp before DPF"
  },
  {
    parameter: "exhaust_temp_dpf_outlet",
    displayName: "DPF Outlet Temperature",
    unit: "°C",
    warningMax: 650,
    criticalMax: 750,
    description: "Exhaust temp after DPF"
  },
  {
    parameter: "fuel_temp",
    displayName: "Fuel Temperature",
    unit: "°C",
    warningMax: 60,
    criticalMax: 70,
    description: "Diesel fuel temperature"
  },

  // DPF
  {
    parameter: "dpf_soot_loading",
    displayName: "DPF Soot Loading",
    unit: "%",
    warningMax: 70,
    criticalMax: 90,
    description: "DPF soot saturation level"
  },
  {
    parameter: "dpf_ash_loading",
    displayName: "DPF Ash Loading",
    unit: "g",
    warningMax: 30,
    criticalMax: 45,
    description: "DPF ash accumulation"
  },
  {
    parameter: "dpf_differential_pressure",
    displayName: "DPF Differential Pressure",
    unit: "mbar",
    warningMax: 150,
    criticalMax: 200,
    description: "Pressure drop across DPF"
  },

  // Electrical
  {
    parameter: "battery_voltage",
    displayName: "Battery Voltage",
    unit: "V",
    warningMin: 11.5,
    warningMax: 15.0,
    criticalMin: 10.5,
    criticalMax: 16.0,
    description: "Vehicle battery voltage"
  },

  // Engine
  {
    parameter: "engine_rpm",
    displayName: "Engine RPM",
    unit: "rpm",
    warningMax: 5000,
    criticalMax: 5500,
    description: "Engine speed"
  },
  {
    parameter: "engine_load",
    displayName: "Engine Load",
    unit: "%",
    warningMax: 95,
    criticalMax: 100,
    description: "Engine load percentage"
  },

  // Transmission (EGS)
  {
    parameter: "transmission_oil_temp",
    displayName: "Transmission Oil Temp",
    unit: "°C",
    warningMin: 30,
    warningMax: 120,
    criticalMax: 140,
    description: "Automatic transmission oil temperature"
  }
]

// Create lookup map for quick access
const thresholdMap = new Map<string, ParameterThreshold>()
DIESEL_THRESHOLDS.forEach(t => thresholdMap.set(t.parameter, t))

/**
 * Check a single parameter value against thresholds
 */
export function checkParameter(
  parameter: string,
  value: number
): Alert | null {
  const threshold = thresholdMap.get(parameter)
  if (!threshold) return null

  const timestamp = Date.now()
  const id = `alert-${timestamp}-${Math.random().toString(36).substring(2, 6)}`

  // Check critical thresholds first
  if (threshold.criticalMin !== undefined && value < threshold.criticalMin) {
    return {
      id,
      timestamp,
      type: "critical",
      parameter,
      displayName: threshold.displayName,
      value,
      unit: threshold.unit,
      threshold: threshold.criticalMin,
      thresholdType: "min",
      message: `${threshold.displayName} critically low: ${value} ${threshold.unit} (min: ${threshold.criticalMin})`
    }
  }

  if (threshold.criticalMax !== undefined && value > threshold.criticalMax) {
    return {
      id,
      timestamp,
      type: "critical",
      parameter,
      displayName: threshold.displayName,
      value,
      unit: threshold.unit,
      threshold: threshold.criticalMax,
      thresholdType: "max",
      message: `${threshold.displayName} critically high: ${value} ${threshold.unit} (max: ${threshold.criticalMax})`
    }
  }

  // Check warning thresholds
  if (threshold.warningMin !== undefined && value < threshold.warningMin) {
    return {
      id,
      timestamp,
      type: "warning",
      parameter,
      displayName: threshold.displayName,
      value,
      unit: threshold.unit,
      threshold: threshold.warningMin,
      thresholdType: "min",
      message: `${threshold.displayName} low: ${value} ${threshold.unit} (min: ${threshold.warningMin})`
    }
  }

  if (threshold.warningMax !== undefined && value > threshold.warningMax) {
    return {
      id,
      timestamp,
      type: "warning",
      parameter,
      displayName: threshold.displayName,
      value,
      unit: threshold.unit,
      threshold: threshold.warningMax,
      thresholdType: "max",
      message: `${threshold.displayName} high: ${value} ${threshold.unit} (max: ${threshold.warningMax})`
    }
  }

  return null
}

/**
 * Check multiple parameters and return all alerts
 */
export function checkAllParameters(
  data: Record<string, { value: number; unit: string }>
): Alert[] {
  const alerts: Alert[] = []

  for (const [parameter, info] of Object.entries(data)) {
    // Normalize parameter name (convert display names to internal names)
    const normalizedParam = normalizeParameterName(parameter)
    const alert = checkParameter(normalizedParam, info.value)
    if (alert) {
      alerts.push(alert)
    }
  }

  return alerts
}

/**
 * Normalize parameter names from various formats to internal names
 */
function normalizeParameterName(name: string): string {
  // Convert from display names or various formats
  const mapping: Record<string, string> = {
    "Fuel Rail Pressure": "fuel_rail_pressure",
    "Rail Pressure": "fuel_rail_pressure",
    "Boost Pressure": "boost_pressure",
    "Turbo Boost": "boost_pressure",
    "VNT Position": "vnt_position",
    "Turbo VNT": "vnt_position",
    "EGR Position": "egr_position",
    "EGR Valve": "egr_position",
    "Coolant Temp": "coolant_temp",
    "Coolant Temperature": "coolant_temp",
    "Oil Temp": "oil_temp",
    "Oil Temperature": "oil_temp",
    "DPF Inlet Temp": "exhaust_temp_dpf_inlet",
    "DPF Outlet Temp": "exhaust_temp_dpf_outlet",
    "Fuel Temp": "fuel_temp",
    "Fuel Temperature": "fuel_temp",
    "DPF Soot": "dpf_soot_loading",
    "Soot Loading": "dpf_soot_loading",
    "DPF Ash": "dpf_ash_loading",
    "Ash Loading": "dpf_ash_loading",
    "DPF Pressure": "dpf_differential_pressure",
    "Battery": "battery_voltage",
    "Battery Voltage": "battery_voltage",
    "Engine RPM": "engine_rpm",
    "RPM": "engine_rpm",
    "Engine Load": "engine_load",
    "Load": "engine_load",
    "Trans Oil Temp": "transmission_oil_temp",
    "Transmission Oil": "transmission_oil_temp"
  }

  // Check mapping first
  if (mapping[name]) {
    return mapping[name]
  }

  // Convert to snake_case if not found
  return name
    .toLowerCase()
    .replace(/\s+/g, "_")
    .replace(/[^a-z0-9_]/g, "")
}

/**
 * Get threshold info for a parameter
 */
export function getThresholdInfo(parameter: string): ParameterThreshold | null {
  return thresholdMap.get(normalizeParameterName(parameter)) || null
}

/**
 * Get all defined thresholds
 */
export function getAllThresholds(): ParameterThreshold[] {
  return DIESEL_THRESHOLDS
}

/**
 * Format alert for display
 */
export function formatAlertMessage(alert: Alert, lang: "es" | "en" = "es"): string {
  const direction = alert.thresholdType === "max"
    ? (lang === "es" ? "alto" : "high")
    : (lang === "es" ? "bajo" : "low")

  const severity = alert.type === "critical"
    ? (lang === "es" ? "CRITICO" : "CRITICAL")
    : (lang === "es" ? "Aviso" : "Warning")

  return `[${severity}] ${alert.displayName} ${direction}: ${alert.value.toFixed(1)} ${alert.unit}`
}

/**
 * Categorize alerts by severity
 */
export function categorizeAlerts(alerts: Alert[]): {
  critical: Alert[]
  warning: Alert[]
} {
  return {
    critical: alerts.filter(a => a.type === "critical"),
    warning: alerts.filter(a => a.type === "warning")
  }
}
