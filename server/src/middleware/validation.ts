/**
 * Input validation middleware using Zod
 *
 * Provides type-safe validation schemas for all API endpoints
 */

import { z, ZodSchema, ZodError } from "zod"
import type { Request, Response, NextFunction } from "express"

// ============================================================================
// SCHEMAS
// ============================================================================

/**
 * DTC (Diagnostic Trouble Code) schema
 */
export const dtcSchema = z.object({
  code: z
    .string()
    .min(4)
    .max(10)
    .regex(/^[PCBU][0-9A-Fa-f]{4,5}$/i, "Invalid DTC format (e.g., P0123, B1234)"),
  status: z
    .object({
      confirmed: z.boolean().optional(),
      pending: z.boolean().optional(),
      raw: z.number().int().min(0).max(255).optional(),
    })
    .optional(),
  description: z.string().max(500).optional(),
})

/**
 * Vehicle info schema
 */
export const vehicleInfoSchema = z.object({
  model: z.string().max(100).optional(),
  year: z.string().max(10).optional(),
  mileage: z.string().max(20).optional(),
  vin: z
    .string()
    .regex(/^[A-HJ-NPR-Z0-9]{17}$/i, "Invalid VIN format")
    .optional(),
})

/**
 * DTC analysis request schema
 */
export const analyzeRequestSchema = z.object({
  dtcs: z.array(dtcSchema).min(1).max(50),
  vehicle: vehicleInfoSchema.optional(),
})

/**
 * Session data schema for history
 */
export const sessionDataSchema = z.object({
  id: z.string().min(1).max(100),
  createdAt: z.number().int().positive(),
  vehicleInfo: vehicleInfoSchema,
  dtcs: z.array(dtcSchema).optional(),
  liveDataSnapshots: z
    .array(
      z.object({
        timestamp: z.number().int().positive(),
        data: z.record(
          z.string(),
          z.object({
            value: z.number(),
            unit: z.string().max(20),
          })
        ),
      })
    )
    .optional(),
  alerts: z
    .array(
      z.object({
        timestamp: z.number().int().positive(),
        type: z.enum(["warning", "critical"]),
        parameter: z.string().max(100),
        value: z.number(),
        threshold: z.number(),
        message: z.string().max(500),
      })
    )
    .optional(),
  notes: z.string().max(5000).optional(),
  aiAnalysis: z.string().max(10000).optional(),
})

/**
 * Compare sessions request schema
 */
export const compareSessionsSchema = z.object({
  sessionId1: z.string().min(1).max(100),
  sessionId2: z.string().min(1).max(100),
})

/**
 * Live data schema (for alerts check)
 */
export const liveDataSchema = z.record(
  z.string(),
  z.object({
    value: z.number(),
    unit: z.string().max(20),
  })
)

/**
 * DTC code parameter schema (for lookup)
 */
export const dtcCodeParamSchema = z.object({
  code: z
    .string()
    .min(4)
    .max(10)
    .regex(/^[PCBU][0-9A-Fa-f]{4,5}$/i, "Invalid DTC format"),
})

/**
 * Session ID parameter schema
 */
export const sessionIdParamSchema = z.object({
  id: z.string().min(1).max(100),
})

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/**
 * Format Zod errors for API response
 */
function formatZodErrors(error: ZodError): Array<{ path: string; message: string }> {
  return error.issues.map((issue) => ({
    path: issue.path.join("."),
    message: issue.message,
  }))
}

// ============================================================================
// MIDDLEWARE FACTORY
// ============================================================================

/**
 * Creates a validation middleware for request body
 */
export function validateBody(schema: ZodSchema) {
  return (req: Request, res: Response, next: NextFunction): void => {
    const result = schema.safeParse(req.body)

    if (!result.success) {
      res.status(400).json({
        error: "Validation error",
        details: formatZodErrors(result.error),
      })
      return
    }

    // Replace body with validated/transformed data
    req.body = result.data
    next()
  }
}

/**
 * Creates a validation middleware for URL parameters
 */
export function validateParams(schema: ZodSchema) {
  return (req: Request, res: Response, next: NextFunction): void => {
    const result = schema.safeParse(req.params)

    if (!result.success) {
      res.status(400).json({
        error: "Invalid parameters",
        details: formatZodErrors(result.error),
      })
      return
    }

    // Merge validated params (don't replace to preserve Express functionality)
    Object.assign(req.params, result.data)
    next()
  }
}

/**
 * Creates a validation middleware for query parameters
 */
export function validateQuery(schema: ZodSchema) {
  return (req: Request, res: Response, next: NextFunction): void => {
    const result = schema.safeParse(req.query)

    if (!result.success) {
      res.status(400).json({
        error: "Invalid query parameters",
        details: formatZodErrors(result.error),
      })
      return
    }

    // Merge validated query params
    Object.assign(req.query, result.data)
    next()
  }
}

// ============================================================================
// TYPE EXPORTS
// ============================================================================

export type DtcInput = z.infer<typeof dtcSchema>
export type VehicleInfo = z.infer<typeof vehicleInfoSchema>
export type AnalyzeRequest = z.infer<typeof analyzeRequestSchema>
export type SessionData = z.infer<typeof sessionDataSchema>
export type CompareSessionsRequest = z.infer<typeof compareSessionsSchema>
export type LiveData = z.infer<typeof liveDataSchema>
