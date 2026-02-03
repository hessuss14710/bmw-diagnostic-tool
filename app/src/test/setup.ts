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
URL.createObjectURL = vi.fn(() => "blob:mock-url")
URL.revokeObjectURL = vi.fn()
