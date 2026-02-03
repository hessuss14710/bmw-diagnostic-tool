import { Router } from "express"
import {
  analyzeHandler,
  lookupHandler,
  listCodesHandler,
  healthHandler,
} from "../controllers/dtcController.js"
import {
  validateBody,
  validateParams,
  analyzeRequestSchema,
  dtcCodeParamSchema,
} from "../middleware/validation.js"

const router = Router()

// Health check
router.get("/health", healthHandler)

// DTC endpoints with validation
router.post("/dtc/analyze", validateBody(analyzeRequestSchema), analyzeHandler)
router.get("/dtc/lookup/:code", validateParams(dtcCodeParamSchema), lookupHandler)
router.get("/dtc/codes", listCodesHandler)

export default router
