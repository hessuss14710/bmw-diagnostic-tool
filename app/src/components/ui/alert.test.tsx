/**
 * Tests for Alert component
 */
import { describe, it, expect, vi } from "vitest"
import { render, screen, fireEvent } from "@testing-library/react"
import { Alert } from "./alert"

describe("Alert", () => {
  it("renders children content", () => {
    render(<Alert>Alert message</Alert>)
    expect(screen.getByText("Alert message")).toBeInTheDocument()
  })

  it("renders with title", () => {
    render(<Alert title="Warning">Content</Alert>)
    expect(screen.getByText("Warning")).toBeInTheDocument()
    expect(screen.getByText("Content")).toBeInTheDocument()
  })

  describe("variants", () => {
    it("renders default variant", () => {
      render(<Alert>Default</Alert>)
      const alert = screen.getByRole("alert")
      expect(alert).toHaveClass("bg-zinc-900")
    })

    it("renders success variant", () => {
      render(<Alert variant="success">Success</Alert>)
      const alert = screen.getByRole("alert")
      expect(alert).toHaveClass("bg-emerald-950/50")
    })

    it("renders warning variant", () => {
      render(<Alert variant="warning">Warning</Alert>)
      const alert = screen.getByRole("alert")
      expect(alert).toHaveClass("bg-amber-950/50")
    })

    it("renders error variant", () => {
      render(<Alert variant="error">Error</Alert>)
      const alert = screen.getByRole("alert")
      expect(alert).toHaveClass("bg-red-950/50")
    })

    it("renders info variant", () => {
      render(<Alert variant="info">Info</Alert>)
      const alert = screen.getByRole("alert")
      expect(alert).toHaveClass("bg-cyan-950/50")
    })
  })

  it("renders appropriate icon for each variant", () => {
    // The icons are rendered inside the alert
    const { rerender } = render(<Alert variant="success">Success</Alert>)
    expect(screen.getByRole("alert").querySelector("svg")).toBeInTheDocument()

    rerender(<Alert variant="error">Error</Alert>)
    expect(screen.getByRole("alert").querySelector("svg")).toBeInTheDocument()

    rerender(<Alert variant="warning">Warning</Alert>)
    expect(screen.getByRole("alert").querySelector("svg")).toBeInTheDocument()

    rerender(<Alert variant="info">Info</Alert>)
    expect(screen.getByRole("alert").querySelector("svg")).toBeInTheDocument()
  })

  it("shows close button when onClose is provided", () => {
    const handleClose = vi.fn()
    render(<Alert onClose={handleClose}>Closeable</Alert>)

    const closeButton = screen.getByRole("button")
    expect(closeButton).toBeInTheDocument()
  })

  it("calls onClose when close button is clicked", () => {
    const handleClose = vi.fn()
    render(<Alert onClose={handleClose}>Closeable</Alert>)

    fireEvent.click(screen.getByRole("button"))
    expect(handleClose).toHaveBeenCalledTimes(1)
  })

  it("does not show close button when onClose is not provided", () => {
    render(<Alert>Not closeable</Alert>)
    expect(screen.queryByRole("button")).not.toBeInTheDocument()
  })

  it("accepts custom className", () => {
    render(<Alert className="custom-alert">Custom</Alert>)
    expect(screen.getByRole("alert")).toHaveClass("custom-alert")
  })
})
