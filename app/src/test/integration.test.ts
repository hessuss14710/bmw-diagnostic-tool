/**
 * Integration tests with realistic BMW diagnostic data
 */
import { describe, it, expect, vi, beforeEach } from "vitest"
import { renderHook, act } from "@testing-library/react"
import { useDatabase } from "@/hooks/useDatabase"
import { invoke } from "@tauri-apps/api/core"

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}))

const mockInvoke = vi.mocked(invoke)

// ============================================================================
// REALISTIC BMW E60 520d TEST DATA
// ============================================================================

const bmwE60Vehicle = {
  id: 1,
  vin: "WBANE71000B123456",
  make: "BMW",
  model: "520d E60",
  year: 2008,
  engine_code: "M47TU2D20",
  mileage_km: 245000,
  notes: "2.0L Diesel, 163HP, Manual 6-speed",
  created_at: "2024-01-15T10:30:00Z",
  updated_at: "2024-01-15T10:30:00Z",
}

const bmwE90Vehicle = {
  id: 2,
  vin: "WBAPH5C55BA654321",
  make: "BMW",
  model: "320d E90",
  year: 2010,
  engine_code: "N47D20",
  mileage_km: 180000,
  notes: "2.0L Diesel, 177HP, Auto",
  created_at: "2024-02-20T14:00:00Z",
  updated_at: "2024-02-20T14:00:00Z",
}

// Real BMW DTC codes for diesel engines
const realDTCs = {
  dpf: [
    { code: "2AAF", status: "0x24", description: "Differential pressure sensor - circuit open", is_pending: false, is_confirmed: true },
    { code: "2AB0", status: "0x24", description: "DPF soot mass - limit exceeded", is_pending: true, is_confirmed: false },
    { code: "4B93", status: "0x27", description: "DPF regeneration - unsuccessful", is_pending: false, is_confirmed: true },
    { code: "480A", status: "0x24", description: "Exhaust gas temperature sensor 1 - signal implausible", is_pending: false, is_confirmed: true },
  ],
  egr: [
    { code: "2A00", status: "0x24", description: "EGR valve - stuck open", is_pending: false, is_confirmed: true },
    { code: "2A01", status: "0x27", description: "EGR valve - stuck closed", is_pending: true, is_confirmed: false },
    { code: "2A82", status: "0x24", description: "EGR cooler bypass valve - malfunction", is_pending: false, is_confirmed: true },
  ],
  turbo: [
    { code: "2A45", status: "0x24", description: "Boost pressure sensor - circuit low", is_pending: false, is_confirmed: true },
    { code: "2A47", status: "0x27", description: "Boost pressure - overboost condition", is_pending: true, is_confirmed: false },
    { code: "2FBF", status: "0x24", description: "Turbo wastegate position sensor - signal range", is_pending: false, is_confirmed: true },
  ],
  injectors: [
    { code: "2BAE", status: "0x24", description: "Injector cylinder 1 - circuit malfunction", is_pending: false, is_confirmed: true },
    { code: "2BBE", status: "0x24", description: "Injector cylinder 2 - circuit malfunction", is_pending: false, is_confirmed: true },
    { code: "2BCE", status: "0x24", description: "Injector cylinder 3 - circuit malfunction", is_pending: true, is_confirmed: false },
    { code: "2BDE", status: "0x24", description: "Injector cylinder 4 - circuit malfunction", is_pending: false, is_confirmed: true },
  ],
  glow: [
    { code: "29CC", status: "0x24", description: "Glow plug cylinder 1 - circuit", is_pending: false, is_confirmed: true },
    { code: "29CD", status: "0x24", description: "Glow plug cylinder 2 - circuit", is_pending: true, is_confirmed: false },
  ],
  dsc: [
    { code: "5E17", status: "0x24", description: "ABS wheel speed sensor front left - signal missing", is_pending: false, is_confirmed: true },
    { code: "5E20", status: "0x27", description: "Steering angle sensor - not calibrated", is_pending: true, is_confirmed: false },
  ],
  airbag: [
    { code: "9353", status: "0x24", description: "Passenger seat occupancy sensor - malfunction", is_pending: false, is_confirmed: true },
    { code: "9395", status: "0x27", description: "Driver seatbelt tensioner - resistance high", is_pending: true, is_confirmed: false },
  ],
}

// Diagnostic sessions
const diagnosticSessions = [
  {
    id: 1,
    vehicle_id: 1,
    ecu_id: "0x12",
    ecu_name: "DDE (Digital Diesel Electronics)",
    protocol: "K-Line KWP2000",
    mileage_km: 245000,
    notes: "DPF warning light on, rough idle",
    created_at: "2024-01-15T11:00:00Z",
  },
  {
    id: 2,
    vehicle_id: 1,
    ecu_id: "0x18",
    ecu_name: "DME (Digital Motor Electronics)",
    protocol: "D-CAN ISO 15765",
    mileage_km: 245000,
    notes: "Follow-up scan after DPF regen",
    created_at: "2024-01-15T14:30:00Z",
  },
  {
    id: 3,
    vehicle_id: 1,
    ecu_id: "0x40",
    ecu_name: "KOMBI (Instrument Cluster)",
    protocol: "K-Line KWP2000",
    mileage_km: 245000,
    notes: "Service reset after oil change",
    created_at: "2024-01-20T09:00:00Z",
  },
]

// Live data readings
const liveDataReadings = {
  engine: {
    coolant_temp: { value: 92, unit: "°C", min: -40, max: 130 },
    oil_temp: { value: 105, unit: "°C", min: -40, max: 150 },
    rpm: { value: 850, unit: "RPM", min: 0, max: 6000 },
    intake_air_temp: { value: 35, unit: "°C", min: -40, max: 80 },
    fuel_rail_pressure: { value: 285, unit: "bar", min: 0, max: 1800 },
    boost_pressure: { value: 1.2, unit: "bar", min: 0, max: 2.5 },
  },
  dpf: {
    soot_mass: { value: 28.5, unit: "g", min: 0, max: 50 },
    ash_mass: { value: 45.2, unit: "g", min: 0, max: 100 },
    differential_pressure: { value: 85, unit: "mbar", min: 0, max: 300 },
    exhaust_temp_pre: { value: 380, unit: "°C", min: 0, max: 800 },
    exhaust_temp_post: { value: 420, unit: "°C", min: 0, max: 800 },
    distance_since_regen: { value: 485, unit: "km", min: 0, max: 1000 },
    regen_count: { value: 156, unit: "", min: 0, max: 9999 },
  },
  turbo: {
    vgt_position: { value: 65, unit: "%", min: 0, max: 100 },
    boost_actual: { value: 1.15, unit: "bar", min: 0, max: 2.5 },
    boost_target: { value: 1.20, unit: "bar", min: 0, max: 2.5 },
  },
}

// Database stats
const databaseStats = {
  vehicle_count: 2,
  session_count: 5,
  dtc_count: 18,
}

// ============================================================================
// INTEGRATION TESTS
// ============================================================================

describe("BMW Diagnostic Integration Tests", () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  describe("Vehicle Management Workflow", () => {
    it("should create and retrieve a BMW E60 520d vehicle", async () => {
      mockInvoke
        .mockResolvedValueOnce(1) // create vehicle
        .mockResolvedValueOnce(bmwE60Vehicle) // get vehicle

      const { result } = renderHook(() => useDatabase())

      // Create vehicle
      let vehicleId: number = 0
      await act(async () => {
        vehicleId = await result.current.createVehicle({
          vin: bmwE60Vehicle.vin,
          make: bmwE60Vehicle.make,
          model: bmwE60Vehicle.model,
          year: bmwE60Vehicle.year,
          engine_code: bmwE60Vehicle.engine_code,
          mileage_km: bmwE60Vehicle.mileage_km,
          notes: bmwE60Vehicle.notes,
        })
      })

      expect(vehicleId).toBe(1)
      expect(mockInvoke).toHaveBeenCalledWith("db_create_vehicle", expect.any(Object))

      // Retrieve vehicle
      let vehicle
      await act(async () => {
        vehicle = await result.current.getVehicle(1)
      })

      expect(vehicle).toEqual(bmwE60Vehicle)
    })

    it("should list multiple BMW vehicles", async () => {
      mockInvoke.mockResolvedValueOnce([bmwE60Vehicle, bmwE90Vehicle])

      const { result } = renderHook(() => useDatabase())

      let vehicles
      await act(async () => {
        vehicles = await result.current.getVehicles()
      })

      expect(vehicles).toHaveLength(2)
      expect(vehicles[0].engine_code).toBe("M47TU2D20")
      expect(vehicles[1].engine_code).toBe("N47D20")
    })

    it("should find vehicle by VIN", async () => {
      mockInvoke.mockResolvedValueOnce(bmwE60Vehicle)

      const { result } = renderHook(() => useDatabase())

      let vehicle
      await act(async () => {
        vehicle = await result.current.getVehicleByVin("WBANE71000B123456")
      })

      expect(vehicle?.model).toBe("520d E60")
      expect(vehicle?.engine_code).toBe("M47TU2D20")
    })
  })

  describe("Diagnostic Session Workflow", () => {
    it("should create a DDE diagnostic session", async () => {
      mockInvoke.mockResolvedValueOnce(1)

      const { result } = renderHook(() => useDatabase())

      let sessionId
      await act(async () => {
        sessionId = await result.current.createSession({
          vehicle_id: 1,
          ecu_id: "0x12",
          ecu_name: "DDE (Digital Diesel Electronics)",
          protocol: "K-Line KWP2000",
          mileage_km: 245000,
          notes: "DPF warning light on, rough idle",
        })
      })

      expect(sessionId).toBe(1)
      expect(mockInvoke).toHaveBeenCalledWith("db_create_session", {
        session: expect.objectContaining({
          ecu_id: "0x12",
          ecu_name: "DDE (Digital Diesel Electronics)",
        }),
      })
    })

    it("should retrieve sessions for a vehicle", async () => {
      mockInvoke.mockResolvedValueOnce(diagnosticSessions)

      const { result } = renderHook(() => useDatabase())

      let sessions
      await act(async () => {
        sessions = await result.current.getSessionsForVehicle(1)
      })

      expect(sessions).toHaveLength(3)
      expect(sessions[0].ecu_name).toContain("DDE")
      expect(sessions[1].ecu_name).toContain("DME")
      expect(sessions[2].ecu_name).toContain("KOMBI")
    })
  })

  describe("DTC Storage Workflow", () => {
    it("should store DPF-related DTCs", async () => {
      mockInvoke.mockResolvedValueOnce(undefined)

      const { result } = renderHook(() => useDatabase())

      const dtcsToStore = realDTCs.dpf.map((dtc) => ({
        session_id: 1,
        ...dtc,
      }))

      await act(async () => {
        await result.current.addDtcs(dtcsToStore)
      })

      expect(mockInvoke).toHaveBeenCalledWith("db_add_dtcs", {
        dtcs: expect.arrayContaining([
          expect.objectContaining({ code: "2AAF" }),
          expect.objectContaining({ code: "2AB0" }),
          expect.objectContaining({ code: "4B93" }),
        ]),
      })
    })

    it("should retrieve DTC history for vehicle", async () => {
      const allDtcs = [
        ...realDTCs.dpf.map((dtc, i) => ({ id: i + 1, session_id: 1, created_at: "2024-01-15T11:05:00Z", ...dtc })),
        ...realDTCs.egr.map((dtc, i) => ({ id: i + 5, session_id: 1, created_at: "2024-01-15T11:05:00Z", ...dtc })),
      ]
      mockInvoke.mockResolvedValueOnce(allDtcs)

      const { result } = renderHook(() => useDatabase())

      let history
      await act(async () => {
        history = await result.current.getDtcHistory(1)
      })

      expect(history).toHaveLength(7)

      // Check DPF codes
      const dpfCodes = history.filter((d: any) => d.code.startsWith("2A") || d.code.startsWith("4"))
      expect(dpfCodes.length).toBeGreaterThan(0)
    })
  })

  describe("Complete Diagnostic Workflow", () => {
    it("should perform full diagnostic workflow", async () => {
      // Mock sequence for complete workflow
      mockInvoke
        .mockResolvedValueOnce(1) // create vehicle
        .mockResolvedValueOnce(1) // create session
        .mockResolvedValueOnce(undefined) // add DTCs
        .mockResolvedValueOnce(realDTCs.dpf.map((d, i) => ({ id: i + 1, session_id: 1, created_at: new Date().toISOString(), ...d }))) // get DTCs
        .mockResolvedValueOnce(databaseStats) // get stats

      const { result } = renderHook(() => useDatabase())

      // 1. Create vehicle
      let vehicleId
      await act(async () => {
        vehicleId = await result.current.createVehicle({
          make: "BMW",
          model: "520d E60",
          year: 2008,
          engine_code: "M47TU2D20",
        })
      })
      expect(vehicleId).toBe(1)

      // 2. Create diagnostic session
      let sessionId
      await act(async () => {
        sessionId = await result.current.createSession({
          vehicle_id: vehicleId!,
          ecu_id: "0x12",
          ecu_name: "DDE",
          protocol: "K-Line",
        })
      })
      expect(sessionId).toBe(1)

      // 3. Store DTCs
      await act(async () => {
        await result.current.addDtcs(
          realDTCs.dpf.map((dtc) => ({
            session_id: sessionId!,
            ...dtc,
          }))
        )
      })

      // 4. Retrieve DTCs
      let dtcs
      await act(async () => {
        dtcs = await result.current.getDtcsForSession(sessionId!)
      })
      expect(dtcs).toHaveLength(4)

      // 5. Get stats
      let stats
      await act(async () => {
        stats = await result.current.getStats()
      })
      expect(stats.dtc_count).toBe(18)
    })
  })

  describe("Data Export", () => {
    it("should export all diagnostic data as JSON", async () => {
      const exportData = JSON.stringify({
        version: "1.0",
        exported_at: new Date().toISOString(),
        vehicles: [bmwE60Vehicle, bmwE90Vehicle],
        sessions: diagnosticSessions.map((s) => ({
          session: s,
          dtcs: realDTCs.dpf,
        })),
        settings: [
          { key: "theme", value: "dark" },
          { key: "language", value: "es" },
        ],
      })

      mockInvoke.mockResolvedValueOnce(exportData)

      const { result } = renderHook(() => useDatabase())

      let data
      await act(async () => {
        data = await result.current.exportAll()
      })

      const parsed = JSON.parse(data as string)
      expect(parsed.version).toBe("1.0")
      expect(parsed.vehicles).toHaveLength(2)
      expect(parsed.sessions).toHaveLength(3)
    })
  })

  describe("Settings Management", () => {
    it("should store and retrieve app settings", async () => {
      mockInvoke
        .mockResolvedValueOnce(undefined) // set theme
        .mockResolvedValueOnce(undefined) // set language
        .mockResolvedValueOnce(undefined) // set units
        .mockResolvedValueOnce([
          { key: "theme", value: "dark" },
          { key: "language", value: "es" },
          { key: "units", value: "metric" },
        ])

      const { result } = renderHook(() => useDatabase())

      // Set settings
      await act(async () => {
        await result.current.setSetting("theme", "dark")
        await result.current.setSetting("language", "es")
        await result.current.setSetting("units", "metric")
      })

      // Get all settings
      let settings
      await act(async () => {
        settings = await result.current.getAllSettings()
      })

      expect(settings).toHaveLength(3)
      expect(settings).toContainEqual({ key: "theme", value: "dark" })
      expect(settings).toContainEqual({ key: "language", value: "es" })
    })
  })

  describe("Error Handling", () => {
    it("should handle database errors gracefully", async () => {
      mockInvoke.mockRejectedValueOnce(new Error("Database connection failed"))

      const { result } = renderHook(() => useDatabase())

      await act(async () => {
        try {
          await result.current.getVehicles()
        } catch {
          // Expected
        }
      })

      expect(result.current.error).toBe("Database connection failed")

      // Clear error
      act(() => {
        result.current.clearError()
      })

      expect(result.current.error).toBeNull()
    })

    it("should handle vehicle not found", async () => {
      mockInvoke.mockResolvedValueOnce(null)

      const { result } = renderHook(() => useDatabase())

      let vehicle
      await act(async () => {
        vehicle = await result.current.getVehicle(999)
      })

      expect(vehicle).toBeNull()
    })
  })
})

// ============================================================================
// DTC CODE VALIDATION TESTS
// ============================================================================

describe("BMW DTC Code Validation", () => {
  it("should have valid DTC code format", () => {
    const allCodes = [
      ...realDTCs.dpf,
      ...realDTCs.egr,
      ...realDTCs.turbo,
      ...realDTCs.injectors,
      ...realDTCs.glow,
      ...realDTCs.dsc,
      ...realDTCs.airbag,
    ]

    allCodes.forEach((dtc) => {
      // BMW DTCs are typically 4 hex characters
      expect(dtc.code).toMatch(/^[0-9A-F]{4}$/i)
      // Status byte should be hex format
      expect(dtc.status).toMatch(/^0x[0-9A-F]{2}$/i)
      // Should have description
      expect(dtc.description.length).toBeGreaterThan(0)
    })
  })

  it("should categorize DTC codes by system", () => {
    // DPF codes typically start with 2A, 4B, 48
    realDTCs.dpf.forEach((dtc) => {
      expect(["2A", "4B", "48"].some((prefix) => dtc.code.startsWith(prefix))).toBe(true)
    })

    // Injector codes typically start with 2B
    realDTCs.injectors.forEach((dtc) => {
      expect(dtc.code.startsWith("2B")).toBe(true)
    })

    // Glow plug codes typically start with 29
    realDTCs.glow.forEach((dtc) => {
      expect(dtc.code.startsWith("29")).toBe(true)
    })

    // DSC codes typically start with 5E
    realDTCs.dsc.forEach((dtc) => {
      expect(dtc.code.startsWith("5E")).toBe(true)
    })

    // Airbag codes typically start with 93
    realDTCs.airbag.forEach((dtc) => {
      expect(dtc.code.startsWith("93")).toBe(true)
    })
  })
})

// ============================================================================
// LIVE DATA VALIDATION TESTS
// ============================================================================

describe("BMW Live Data Validation", () => {
  it("should have valid engine temperature readings", () => {
    const { coolant_temp, oil_temp } = liveDataReadings.engine

    // Coolant temp should be within operating range
    expect(coolant_temp.value).toBeGreaterThanOrEqual(coolant_temp.min)
    expect(coolant_temp.value).toBeLessThanOrEqual(coolant_temp.max)
    expect(coolant_temp.unit).toBe("°C")

    // Oil temp should be higher than coolant when warm
    expect(oil_temp.value).toBeGreaterThan(coolant_temp.value)
  })

  it("should have valid DPF readings", () => {
    const { soot_mass, ash_mass, differential_pressure } = liveDataReadings.dpf

    // Soot should be within normal range
    expect(soot_mass.value).toBeGreaterThanOrEqual(0)
    expect(soot_mass.value).toBeLessThanOrEqual(50)
    expect(soot_mass.unit).toBe("g")

    // Ash accumulates over time
    expect(ash_mass.value).toBeGreaterThanOrEqual(0)
    expect(ash_mass.unit).toBe("g")

    // Differential pressure indicates filter loading
    expect(differential_pressure.value).toBeGreaterThanOrEqual(0)
    expect(differential_pressure.unit).toBe("mbar")
  })

  it("should have valid turbo readings", () => {
    const { boost_actual, boost_target, vgt_position } = liveDataReadings.turbo

    // Boost should be close to target
    expect(Math.abs(boost_actual.value - boost_target.value)).toBeLessThan(0.2)

    // VGT position should be percentage
    expect(vgt_position.value).toBeGreaterThanOrEqual(0)
    expect(vgt_position.value).toBeLessThanOrEqual(100)
    expect(vgt_position.unit).toBe("%")
  })
})
