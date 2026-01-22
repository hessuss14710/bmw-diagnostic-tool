import express from "express"
import { createServer } from "http"
import { Server } from "socket.io"
import cors from "cors"
import dotenv from "dotenv"
import dtcRoutes from "./routes/dtcRoutes.js"
import { analyzeDtcs, type DtcInput } from "./services/openaiService.js"
import OpenAI from "openai"

// Load environment variables
dotenv.config()

const app = express()
const httpServer = createServer(app)
const PORT = process.env.PORT || 3002

// Socket.IO with CORS
const io = new Server(httpServer, {
  cors: {
    origin: "*",
    methods: ["GET", "POST"],
  },
})

// Store active sessions
interface DiagnosticSession {
  id: string
  appSocketId: string | null
  vehicleInfo: {
    model?: string
    vin?: string
    mileage?: string
  }
  dtcs: DtcInput[]
  liveData: Record<string, { value: number; unit: string; timestamp: number }>
  connectedDashboards: Set<string>
  lastActivity: number
}

const sessions = new Map<string, DiagnosticSession>()

// OpenAI client for chat
let openai: OpenAI | null = null
function getOpenAI(): OpenAI | null {
  if (!openai && process.env.OPENAI_API_KEY) {
    openai = new OpenAI({ apiKey: process.env.OPENAI_API_KEY })
  }
  return openai
}

// Middleware
app.use(cors())
app.use(express.json())

// Request logging
app.use((req, _res, next) => {
  console.log(`${new Date().toISOString()} ${req.method} ${req.path}`)
  next()
})

// Routes
app.use("/api", dtcRoutes)

// Active sessions endpoint
app.get("/api/sessions", (_req, res) => {
  const sessionList = Array.from(sessions.values()).map((s) => ({
    id: s.id,
    vehicleInfo: s.vehicleInfo,
    dtcCount: s.dtcs.length,
    liveDataCount: Object.keys(s.liveData).length,
    dashboardCount: s.connectedDashboards.size,
    hasApp: s.appSocketId !== null,
    lastActivity: s.lastActivity,
  }))
  res.json(sessionList)
})

// Root endpoint
app.get("/", (_req, res) => {
  res.json({
    name: "BMW Diagnostic AI Server",
    version: "2.0.0",
    features: ["REST API", "WebSocket", "AI Chat"],
    endpoints: {
      health: "GET /api/health",
      analyze: "POST /api/dtc/analyze",
      lookup: "GET /api/dtc/lookup/:code",
      sessions: "GET /api/sessions",
    },
    websocket: {
      app: "Tauri app connection",
      dashboard: "Web dashboard connection",
    },
  })
})

// =============================================
// SOCKET.IO - App namespace (Tauri app)
// =============================================
const appNamespace = io.of("/app")

appNamespace.on("connection", (socket) => {
  console.log(`[App] Connected: ${socket.id}`)

  // Create or join session
  socket.on("session:create", (data: { sessionId: string; vehicleInfo?: object }) => {
    const sessionId = data.sessionId || `session-${Date.now()}`

    let session = sessions.get(sessionId)
    if (!session) {
      session = {
        id: sessionId,
        appSocketId: socket.id,
        vehicleInfo: data.vehicleInfo || {},
        dtcs: [],
        liveData: {},
        connectedDashboards: new Set(),
        lastActivity: Date.now(),
      }
      sessions.set(sessionId, session)
    } else {
      session.appSocketId = socket.id
      if (data.vehicleInfo) {
        session.vehicleInfo = { ...session.vehicleInfo, ...data.vehicleInfo }
      }
    }

    socket.join(sessionId)
    socket.data.sessionId = sessionId

    console.log(`[App] Session created/joined: ${sessionId}`)
    socket.emit("session:joined", { sessionId, session: sanitizeSession(session) })

    // Notify dashboards
    dashboardNamespace.to(sessionId).emit("session:updated", sanitizeSession(session))
  })

  // Receive DTCs from app
  socket.on("dtcs:update", (data: { dtcs: DtcInput[] }) => {
    const sessionId = socket.data.sessionId
    const session = sessions.get(sessionId)
    if (!session) return

    session.dtcs = data.dtcs
    session.lastActivity = Date.now()

    console.log(`[App] DTCs updated: ${data.dtcs.length} codes`)

    // Broadcast to dashboards
    dashboardNamespace.to(sessionId).emit("dtcs:updated", {
      dtcs: data.dtcs,
      timestamp: Date.now(),
    })
  })

  // Receive live data from app
  socket.on("livedata:update", (data: Record<string, { value: number; unit: string }>) => {
    const sessionId = socket.data.sessionId
    const session = sessions.get(sessionId)
    if (!session) return

    const timestamp = Date.now()
    for (const [key, value] of Object.entries(data)) {
      // Validate data before storing - skip invalid entries
      if (value && typeof value.value === "number" && !isNaN(value.value)) {
        session.liveData[key] = {
          value: value.value,
          unit: value.unit || "",
          timestamp,
        }
      }
    }
    session.lastActivity = timestamp

    // Broadcast to dashboards
    dashboardNamespace.to(sessionId).emit("livedata:updated", {
      data: session.liveData,
      timestamp,
    })
  })

  // ECU connected/disconnected
  socket.on("ecu:status", (data: { connected: boolean; ecu?: string; protocol?: string }) => {
    const sessionId = socket.data.sessionId
    dashboardNamespace.to(sessionId).emit("ecu:status", data)
  })

  socket.on("disconnect", () => {
    const sessionId = socket.data.sessionId
    const session = sessions.get(sessionId)
    if (session) {
      session.appSocketId = null
      dashboardNamespace.to(sessionId).emit("app:disconnected")
    }
    console.log(`[App] Disconnected: ${socket.id}`)
  })
})

// =============================================
// SOCKET.IO - Dashboard namespace (Web)
// =============================================
const dashboardNamespace = io.of("/dashboard")

dashboardNamespace.on("connection", (socket) => {
  console.log(`[Dashboard] Connected: ${socket.id}`)

  // List available sessions
  socket.on("sessions:list", () => {
    const sessionList = Array.from(sessions.values()).map(sanitizeSession)
    socket.emit("sessions:list", sessionList)
  })

  // Join a session
  socket.on("session:join", (data: { sessionId: string }) => {
    const session = sessions.get(data.sessionId)
    if (!session) {
      socket.emit("error", { message: "Session not found" })
      return
    }

    socket.join(data.sessionId)
    socket.data.sessionId = data.sessionId
    session.connectedDashboards.add(socket.id)

    console.log(`[Dashboard] Joined session: ${data.sessionId}`)

    // Send current state
    socket.emit("session:state", {
      session: sanitizeSession(session),
      dtcs: session.dtcs,
      liveData: session.liveData,
    })

    // Notify app
    if (session.appSocketId) {
      appNamespace.to(session.appSocketId).emit("dashboard:connected", {
        dashboardCount: session.connectedDashboards.size,
      })
    }
  })

  // Request AI analysis
  socket.on("ai:analyze", async (data: { dtcs?: DtcInput[]; vehicleInfo?: object }) => {
    const sessionId = socket.data.sessionId
    const session = sessions.get(sessionId)

    const dtcsToAnalyze = data.dtcs || session?.dtcs || []
    const vehicleInfo = data.vehicleInfo || session?.vehicleInfo

    if (dtcsToAnalyze.length === 0) {
      socket.emit("ai:error", { message: "No hay códigos de error para analizar" })
      return
    }

    socket.emit("ai:analyzing", { message: "Analizando códigos con IA..." })

    try {
      const analysis = await analyzeDtcs(dtcsToAnalyze, vehicleInfo as { model?: string; year?: string; mileage?: string })

      // Send to requesting socket
      socket.emit("ai:analysis", analysis)

      // Broadcast to OTHER dashboards in session (not the sender)
      if (sessionId) {
        socket.broadcast.to(sessionId).emit("ai:analysis", analysis)
      }
    } catch (error) {
      socket.emit("ai:error", {
        message: error instanceof Error ? error.message : "Error en análisis",
      })
    }
  })

  // AI Chat
  socket.on("ai:chat", async (data: { message: string; context?: object }) => {
    const sessionId = socket.data.sessionId
    const session = sessions.get(sessionId)
    const client = getOpenAI()

    if (!client) {
      socket.emit("ai:chat:response", {
        role: "assistant",
        content: "Lo siento, el servicio de IA no está disponible en este momento.",
      })
      return
    }

    // Build context from session
    const contextInfo = session
      ? `Vehículo: BMW ${session.vehicleInfo.model || "E60"}
VIN: ${session.vehicleInfo.vin || "No disponible"}
Kilometraje: ${session.vehicleInfo.mileage || "No disponible"}
Códigos de error actuales: ${session.dtcs.map((d) => d.code).join(", ") || "Ninguno"}
Datos en vivo: ${Object.entries(session.liveData)
          .map(([k, v]) => `${k}: ${v.value} ${v.unit}`)
          .join(", ") || "No disponible"}`
      : "No hay sesión de diagnóstico activa."

    try {
      const completion = await client.chat.completions.create({
        model: "gpt-4o-mini",
        messages: [
          {
            role: "system",
            content: `Eres un mecánico experto en BMW. Responde en español de forma concisa y útil.

Contexto del diagnóstico actual:
${contextInfo}

Ayuda al usuario con sus preguntas sobre el vehículo, los códigos de error, o recomendaciones de reparación.`,
          },
          { role: "user", content: data.message },
        ],
        max_tokens: 500,
        temperature: 0.7,
      })

      const response = completion.choices[0].message.content || "No pude generar una respuesta."

      socket.emit("ai:chat:response", {
        role: "assistant",
        content: response,
      })
    } catch (error) {
      socket.emit("ai:chat:response", {
        role: "assistant",
        content: "Error al procesar tu mensaje. Inténtalo de nuevo.",
      })
    }
  })

  // Request live data start/stop (forward to app)
  socket.on("livedata:request", (data: { action: "start" | "stop"; pids?: number[] }) => {
    const sessionId = socket.data.sessionId
    const session = sessions.get(sessionId)
    if (session?.appSocketId) {
      appNamespace.to(session.appSocketId).emit("livedata:request", data)
    }
  })

  socket.on("disconnect", () => {
    const sessionId = socket.data.sessionId
    const session = sessions.get(sessionId)
    if (session) {
      session.connectedDashboards.delete(socket.id)
    }
    console.log(`[Dashboard] Disconnected: ${socket.id}`)
  })
})

// Helper to sanitize session for sending
function sanitizeSession(session: DiagnosticSession) {
  return {
    id: session.id,
    vehicleInfo: session.vehicleInfo,
    dtcCount: session.dtcs.length,
    liveDataKeys: Object.keys(session.liveData),
    hasApp: session.appSocketId !== null,
    dashboardCount: session.connectedDashboards.size,
    lastActivity: session.lastActivity,
  }
}

// Cleanup old sessions every 5 minutes
setInterval(() => {
  const now = Date.now()
  const timeout = 30 * 60 * 1000 // 30 minutes

  for (const [id, session] of sessions) {
    if (now - session.lastActivity > timeout && !session.appSocketId && session.connectedDashboards.size === 0) {
      sessions.delete(id)
      console.log(`[Cleanup] Removed inactive session: ${id}`)
    }
  }
}, 5 * 60 * 1000)

// Error handling
app.use(
  (
    err: Error,
    _req: express.Request,
    res: express.Response,
    _next: express.NextFunction
  ) => {
    console.error("Unhandled error:", err)
    res.status(500).json({
      error: "Internal server error",
      message: err.message,
    })
  }
)

// Start server
httpServer.listen(PORT, () => {
  console.log(`BMW Diagnostic AI Server running on port ${PORT}`)
  console.log(`API: http://localhost:${PORT}/api`)
  console.log(`WebSocket: ws://localhost:${PORT}`)
  console.log(`  - App namespace: /app`)
  console.log(`  - Dashboard namespace: /dashboard`)

  if (!process.env.OPENAI_API_KEY) {
    console.warn("WARNING: OPENAI_API_KEY not set. AI features will be limited.")
  }
})
