import { Router } from "express"
import {
  analyzeHandler,
  lookupHandler,
  listCodesHandler,
  healthHandler,
} from "../controllers/dtcController.js"

const router = Router()

// Health check
router.get("/health", healthHandler)

// DTC endpoints
router.post("/dtc/analyze", analyzeHandler)
router.get("/dtc/lookup/:code", lookupHandler)
router.get("/dtc/codes", listCodesHandler)

export default router
