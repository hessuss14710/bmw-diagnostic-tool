/**
 * Tests for Button component
 */
import { describe, it, expect, vi } from "vitest"
import { render, screen, fireEvent } from "@testing-library/react"
import { Button } from "./button"
import { Save } from "lucide-react"

describe("Button", () => {
  it("renders with children", () => {
    render(<Button>Click me</Button>)
    expect(screen.getByRole("button", { name: /click me/i })).toBeInTheDocument()
  })

  it("handles click events", () => {
    const handleClick = vi.fn()
    render(<Button onClick={handleClick}>Click</Button>)

    fireEvent.click(screen.getByRole("button"))
    expect(handleClick).toHaveBeenCalledTimes(1)
  })

  it("is disabled when disabled prop is true", () => {
    render(<Button disabled>Disabled</Button>)
    expect(screen.getByRole("button")).toBeDisabled()
  })

  it("is disabled when loading", () => {
    render(<Button loading>Loading</Button>)
    expect(screen.getByRole("button")).toBeDisabled()
  })

  it("shows loading spinner when loading", () => {
    render(<Button loading>Submit</Button>)
    // The Loader2 icon should be present (animate-spin class)
    const button = screen.getByRole("button")
    expect(button.querySelector(".animate-spin")).toBeInTheDocument()
  })

  it("renders left icon", () => {
    render(<Button leftIcon={<Save data-testid="save-icon" />}>Save</Button>)
    expect(screen.getByTestId("save-icon")).toBeInTheDocument()
  })

  it("renders right icon", () => {
    render(<Button rightIcon={<Save data-testid="save-icon" />}>Save</Button>)
    expect(screen.getByTestId("save-icon")).toBeInTheDocument()
  })

  it("hides right icon when loading", () => {
    render(<Button loading rightIcon={<Save data-testid="save-icon" />}>Save</Button>)
    expect(screen.queryByTestId("save-icon")).not.toBeInTheDocument()
  })

  describe("variants", () => {
    it("renders default variant", () => {
      render(<Button>Default</Button>)
      const button = screen.getByRole("button")
      expect(button).toHaveClass("bg-blue-600")
    })

    it("renders destructive variant", () => {
      render(<Button variant="destructive">Delete</Button>)
      const button = screen.getByRole("button")
      expect(button).toHaveClass("bg-red-600")
    })

    it("renders success variant", () => {
      render(<Button variant="success">Success</Button>)
      const button = screen.getByRole("button")
      expect(button).toHaveClass("bg-emerald-600")
    })

    it("renders outline variant", () => {
      render(<Button variant="outline">Outline</Button>)
      const button = screen.getByRole("button")
      expect(button).toHaveClass("border")
      expect(button).toHaveClass("bg-transparent")
    })

    it("renders ghost variant", () => {
      render(<Button variant="ghost">Ghost</Button>)
      const button = screen.getByRole("button")
      expect(button).toHaveClass("text-zinc-400")
    })

    it("renders bmw variant", () => {
      render(<Button variant="bmw">BMW</Button>)
      const button = screen.getByRole("button")
      expect(button).toHaveClass("bg-gradient-to-r")
    })
  })

  describe("sizes", () => {
    it("renders default size", () => {
      render(<Button>Default Size</Button>)
      const button = screen.getByRole("button")
      expect(button).toHaveClass("h-10")
    })

    it("renders small size", () => {
      render(<Button size="sm">Small</Button>)
      const button = screen.getByRole("button")
      expect(button).toHaveClass("h-8")
    })

    it("renders large size", () => {
      render(<Button size="lg">Large</Button>)
      const button = screen.getByRole("button")
      expect(button).toHaveClass("h-12")
    })

    it("renders icon size", () => {
      render(<Button size="icon">+</Button>)
      const button = screen.getByRole("button")
      expect(button).toHaveClass("h-10")
      expect(button).toHaveClass("w-10")
    })
  })

  it("accepts custom className", () => {
    render(<Button className="custom-class">Custom</Button>)
    expect(screen.getByRole("button")).toHaveClass("custom-class")
  })

  it("forwards ref", () => {
    const ref = vi.fn()
    render(<Button ref={ref}>Ref</Button>)
    expect(ref).toHaveBeenCalled()
  })
})
