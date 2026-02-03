/**
 * Tests for useDatabase hook
 */
import { describe, it, expect, vi, beforeEach } from "vitest"
import { renderHook, act, waitFor } from "@testing-library/react"
import { useDatabase } from "./useDatabase"
import { invoke } from "@tauri-apps/api/core"
import { mockVehicles, mockVehicle, mockStats } from "@/test/mocks"

// Mock the Tauri invoke function
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}))

const mockInvoke = vi.mocked(invoke)

describe("useDatabase", () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  describe("initial state", () => {
    it("starts with isLoading false", () => {
      const { result } = renderHook(() => useDatabase())
      expect(result.current.isLoading).toBe(false)
    })

    it("starts with no error", () => {
      const { result } = renderHook(() => useDatabase())
      expect(result.current.error).toBeNull()
    })
  })

  describe("getVehicles", () => {
    it("fetches vehicles successfully", async () => {
      mockInvoke.mockResolvedValueOnce(mockVehicles)

      const { result } = renderHook(() => useDatabase())

      let vehicles
      await act(async () => {
        vehicles = await result.current.getVehicles()
      })

      expect(mockInvoke).toHaveBeenCalledWith("db_get_vehicles")
      expect(vehicles).toEqual(mockVehicles)
    })

    it("handles errors", async () => {
      mockInvoke.mockRejectedValueOnce(new Error("Database error"))

      const { result } = renderHook(() => useDatabase())

      await act(async () => {
        try {
          await result.current.getVehicles()
        } catch {
          // Expected to throw
        }
      })

      expect(result.current.error).toBe("Database error")
    })

    it("sets loading state during fetch", async () => {
      let resolvePromise: (value: unknown) => void
      const promise = new Promise((resolve) => {
        resolvePromise = resolve
      })
      mockInvoke.mockReturnValueOnce(promise as Promise<unknown>)

      const { result } = renderHook(() => useDatabase())

      // Start the fetch
      let fetchPromise: Promise<unknown>
      act(() => {
        fetchPromise = result.current.getVehicles()
      })

      // Should be loading
      expect(result.current.isLoading).toBe(true)

      // Resolve the promise
      await act(async () => {
        resolvePromise!(mockVehicles)
        await fetchPromise
      })

      // Should no longer be loading
      expect(result.current.isLoading).toBe(false)
    })
  })

  describe("createVehicle", () => {
    it("creates a vehicle successfully", async () => {
      mockInvoke.mockResolvedValueOnce(3)

      const { result } = renderHook(() => useDatabase())
      const newVehicle = {
        make: "BMW",
        model: "530d E60",
        year: 2007,
      }

      let id
      await act(async () => {
        id = await result.current.createVehicle(newVehicle)
      })

      expect(mockInvoke).toHaveBeenCalledWith("db_create_vehicle", { vehicle: newVehicle })
      expect(id).toBe(3)
    })
  })

  describe("updateVehicle", () => {
    it("updates a vehicle successfully", async () => {
      mockInvoke.mockResolvedValueOnce(true)

      const { result } = renderHook(() => useDatabase())
      const updates = { make: "BMW", model: "520d E60 LCI", year: 2008 }

      let success
      await act(async () => {
        success = await result.current.updateVehicle(1, updates)
      })

      expect(mockInvoke).toHaveBeenCalledWith("db_update_vehicle", { id: 1, vehicle: updates })
      expect(success).toBe(true)
    })
  })

  describe("deleteVehicle", () => {
    it("deletes a vehicle successfully", async () => {
      mockInvoke.mockResolvedValueOnce(true)

      const { result } = renderHook(() => useDatabase())

      let success
      await act(async () => {
        success = await result.current.deleteVehicle(1)
      })

      expect(mockInvoke).toHaveBeenCalledWith("db_delete_vehicle", { id: 1 })
      expect(success).toBe(true)
    })
  })

  describe("getStats", () => {
    it("fetches database stats", async () => {
      mockInvoke.mockResolvedValueOnce(mockStats)

      const { result } = renderHook(() => useDatabase())

      let stats
      await act(async () => {
        stats = await result.current.getStats()
      })

      expect(mockInvoke).toHaveBeenCalledWith("db_get_stats")
      expect(stats).toEqual(mockStats)
    })
  })

  describe("exportAll", () => {
    it("exports all data as JSON", async () => {
      const exportData = JSON.stringify({ vehicles: mockVehicles })
      mockInvoke.mockResolvedValueOnce(exportData)

      const { result } = renderHook(() => useDatabase())

      let data
      await act(async () => {
        data = await result.current.exportAll()
      })

      expect(mockInvoke).toHaveBeenCalledWith("db_export_all")
      expect(data).toBe(exportData)
    })
  })

  describe("clearError", () => {
    it("clears the error state", async () => {
      mockInvoke.mockRejectedValueOnce(new Error("Test error"))

      const { result } = renderHook(() => useDatabase())

      // Cause an error
      await act(async () => {
        try {
          await result.current.getVehicles()
        } catch {
          // Expected
        }
      })

      expect(result.current.error).toBe("Test error")

      // Clear the error
      act(() => {
        result.current.clearError()
      })

      expect(result.current.error).toBeNull()
    })
  })

  describe("session operations", () => {
    it("creates a session", async () => {
      mockInvoke.mockResolvedValueOnce(1)

      const { result } = renderHook(() => useDatabase())
      const session = {
        vehicle_id: 1,
        ecu_id: "0x12",
        ecu_name: "DME",
        protocol: "K-Line",
      }

      let id
      await act(async () => {
        id = await result.current.createSession(session)
      })

      expect(mockInvoke).toHaveBeenCalledWith("db_create_session", { session })
      expect(id).toBe(1)
    })

    it("gets sessions for vehicle", async () => {
      const sessions = [{ id: 1, vehicle_id: 1, ecu_name: "DME" }]
      mockInvoke.mockResolvedValueOnce(sessions)

      const { result } = renderHook(() => useDatabase())

      let data
      await act(async () => {
        data = await result.current.getSessionsForVehicle(1)
      })

      expect(mockInvoke).toHaveBeenCalledWith("db_get_sessions_for_vehicle", { vehicleId: 1 })
      expect(data).toEqual(sessions)
    })
  })

  describe("DTC operations", () => {
    it("adds DTCs", async () => {
      mockInvoke.mockResolvedValueOnce(undefined)

      const { result } = renderHook(() => useDatabase())
      const dtcs = [
        { session_id: 1, code: "2AAF", status: "0x24", is_pending: false, is_confirmed: true },
      ]

      await act(async () => {
        await result.current.addDtcs(dtcs)
      })

      expect(mockInvoke).toHaveBeenCalledWith("db_add_dtcs", { dtcs })
    })

    it("gets DTC history for vehicle", async () => {
      const dtcs = [{ id: 1, code: "2AAF", status: "0x24" }]
      mockInvoke.mockResolvedValueOnce(dtcs)

      const { result } = renderHook(() => useDatabase())

      let data
      await act(async () => {
        data = await result.current.getDtcHistory(1)
      })

      expect(mockInvoke).toHaveBeenCalledWith("db_get_dtc_history", { vehicleId: 1 })
      expect(data).toEqual(dtcs)
    })
  })

  describe("settings operations", () => {
    it("gets a setting", async () => {
      mockInvoke.mockResolvedValueOnce("value")

      const { result } = renderHook(() => useDatabase())

      let value
      await act(async () => {
        value = await result.current.getSetting("key")
      })

      expect(mockInvoke).toHaveBeenCalledWith("db_get_setting", { key: "key" })
      expect(value).toBe("value")
    })

    it("sets a setting", async () => {
      mockInvoke.mockResolvedValueOnce(undefined)

      const { result } = renderHook(() => useDatabase())

      await act(async () => {
        await result.current.setSetting("key", "value")
      })

      expect(mockInvoke).toHaveBeenCalledWith("db_set_setting", { key: "key", value: "value" })
    })
  })
})
