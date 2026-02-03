/**
 * Test mocks and utilities
 */
import { vi } from "vitest"
import type { Vehicle, DiagnosticSession, StoredDtc, DatabaseStats } from "@/hooks/useDatabase"

// Mock vehicle data
export const mockVehicle: Vehicle = {
  id: 1,
  vin: "WBAPH5C55BA123456",
  make: "BMW",
  model: "520d E60",
  year: 2008,
  engine_code: "M47TU2D20",
  mileage_km: 245000,
  notes: "Test vehicle",
  created_at: "2024-01-15T10:30:00Z",
  updated_at: "2024-01-15T10:30:00Z",
}

export const mockVehicles: Vehicle[] = [
  mockVehicle,
  {
    id: 2,
    vin: null,
    make: "BMW",
    model: "320d E90",
    year: 2010,
    engine_code: "N47D20",
    mileage_km: 180000,
    notes: null,
    created_at: "2024-02-20T14:00:00Z",
    updated_at: "2024-02-20T14:00:00Z",
  },
]

// Mock session data
export const mockSession: DiagnosticSession = {
  id: 1,
  vehicle_id: 1,
  ecu_id: "0x12",
  ecu_name: "DME/DDE",
  protocol: "K-Line",
  mileage_km: 245000,
  notes: "Initial diagnostic",
  created_at: "2024-01-15T11:00:00Z",
}

// Mock DTC data
export const mockDtcs: StoredDtc[] = [
  {
    id: 1,
    session_id: 1,
    code: "2AAF",
    status: "0x24",
    description: "DPF pressure sensor - circuit open",
    is_pending: false,
    is_confirmed: true,
    created_at: "2024-01-15T11:05:00Z",
  },
  {
    id: 2,
    session_id: 1,
    code: "2AB0",
    status: "0x24",
    description: "DPF soot mass - limit exceeded",
    is_pending: true,
    is_confirmed: false,
    created_at: "2024-01-15T11:05:00Z",
  },
]

// Mock database stats
export const mockStats: DatabaseStats = {
  vehicle_count: 2,
  session_count: 5,
  dtc_count: 12,
}

// Create a mock for Tauri invoke
export function createTauriMock() {
  const mockInvoke = vi.fn()

  // Default implementations
  mockInvoke.mockImplementation((command: string) => {
    switch (command) {
      case "db_get_vehicles":
        return Promise.resolve(mockVehicles)
      case "db_get_vehicle":
        return Promise.resolve(mockVehicle)
      case "db_create_vehicle":
        return Promise.resolve(3)
      case "db_update_vehicle":
        return Promise.resolve(true)
      case "db_delete_vehicle":
        return Promise.resolve(true)
      case "db_get_stats":
        return Promise.resolve(mockStats)
      case "db_export_all":
        return Promise.resolve(JSON.stringify({ vehicles: mockVehicles }))
      case "list_serial_ports":
        return Promise.resolve(["/dev/ttyUSB0", "/dev/ttyUSB1"])
      default:
        return Promise.reject(new Error(`Unknown command: ${command}`))
    }
  })

  return mockInvoke
}

// Render wrapper with providers
import React, { type ReactNode } from "react"
import { ToastProvider } from "@/components/ui/toast"

export function TestWrapper({ children }: { children: ReactNode }) {
  return React.createElement(ToastProvider, null, children)
}
