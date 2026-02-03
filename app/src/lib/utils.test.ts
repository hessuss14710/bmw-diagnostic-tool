/**
 * Tests for utility functions
 */
import { describe, it, expect } from "vitest"
import { cn } from "./utils"

describe("cn (className utility)", () => {
  it("merges class names", () => {
    expect(cn("foo", "bar")).toBe("foo bar")
  })

  it("handles conditional classes", () => {
    expect(cn("base", true && "active", false && "inactive")).toBe("base active")
  })

  it("handles arrays", () => {
    expect(cn(["foo", "bar"])).toBe("foo bar")
  })

  it("handles objects", () => {
    expect(cn({ foo: true, bar: false, baz: true })).toBe("foo baz")
  })

  it("merges tailwind classes correctly", () => {
    // tailwind-merge should dedupe conflicting classes
    expect(cn("px-4", "px-6")).toBe("px-6")
    expect(cn("text-red-500", "text-blue-500")).toBe("text-blue-500")
  })

  it("handles empty inputs", () => {
    expect(cn()).toBe("")
    expect(cn("", null, undefined)).toBe("")
  })

  it("handles complex combinations", () => {
    const result = cn(
      "base-class",
      true && "conditional",
      { active: true, disabled: false },
      ["array-class"],
      "final"
    )
    expect(result).toBe("base-class conditional active array-class final")
  })
})
