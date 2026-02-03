/**
 * Vitest setup file
 */
import "@testing-library/jest-dom"
import { vi } from "vitest"

// Mock Tauri API
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}))

// Mock clipboard API
Object.assign(navigator, {
  clipboard: {
    writeText: vi.fn().mockResolvedValue(undefined),
    readText: vi.fn().mockResolvedValue(""),
  },
})

// Mock URL API for blob downloads
global.URL.createObjectURL = vi.fn(() => "blob:mock-url")
global.URL.revokeObjectURL = vi.fn()
