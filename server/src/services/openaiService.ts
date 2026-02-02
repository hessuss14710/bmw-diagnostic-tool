import OpenAI from "openai"
import genericCodes from "../data/dtc-codes-generic.json" with { type: "json" }
import bmwCodes from "../data/dtc-codes-bmw.json" with { type: "json" }

// Lazy initialization of OpenAI client
let openai: OpenAI | null = null

function getOpenAI(): OpenAI | null {
  if (!openai && process.env.OPENAI_API_KEY) {
    openai = new OpenAI({
      apiKey: process.env.OPENAI_API_KEY,
    })
  }
  return openai
}

export interface DtcInput {
  code: string
  status: {
    confirmed: boolean
    pending: boolean
    test_failed: boolean
  }
}

export interface DtcAnalysis {
  code: string
  description: string
  severity: "low" | "moderate" | "high" | "critical"
  category: string
  symptoms: string[]
  causes: string[]
  repairs: string[]
  bmw_specific: boolean
  estimated_cost?: string
}

export interface FullAnalysis {
  dtcs: DtcAnalysis[]
  summary: string
  priority_order: string[]
  combined_diagnosis: string
  recommended_actions: string[]
  safety_warning?: string
}

// Lookup DTC in database
function lookupDtc(code: string): DtcAnalysis | null {
  // Filter out _metadata from bmwCodes
  const { _metadata, ...bmwCodesFiltered } = bmwCodes as Record<string, unknown>

  const allCodes = { ...genericCodes, ...bmwCodesFiltered } as Record<string, {
    description: string
    category: string
    severity: string
    symptoms: string[]
    causes: string[]
    repairs: string[]
    bmw_specific?: boolean
  }>

  const entry = allCodes[code]
  if (entry) {
    return {
      code,
      description: entry.description,
      severity: entry.severity as DtcAnalysis["severity"],
      category: entry.category,
      symptoms: entry.symptoms,
      causes: entry.causes,
      repairs: entry.repairs,
      bmw_specific: entry.bmw_specific ?? false,
    }
  }
  return null
}

// Analyze DTCs with AI
export async function analyzeDtcs(
  dtcs: DtcInput[],
  vehicleInfo?: { model?: string; year?: string; mileage?: string }
): Promise<FullAnalysis> {
  // First, lookup all codes in database
  const knownDtcs: DtcAnalysis[] = []
  const unknownCodes: string[] = []

  for (const dtc of dtcs) {
    const lookup = lookupDtc(dtc.code)
    if (lookup) {
      knownDtcs.push(lookup)
    } else {
      unknownCodes.push(dtc.code)
    }
  }

  // Build context for AI
  const dtcContext = dtcs.map((dtc) => {
    const lookup = lookupDtc(dtc.code)
    return {
      code: dtc.code,
      status: dtc.status.confirmed ? "confirmed" : dtc.status.pending ? "pending" : "stored",
      known_info: lookup || "Unknown code - analyze based on OBD-II standards",
    }
  })

  const vehicleContext = vehicleInfo
    ? `Vehicle: BMW ${vehicleInfo.model || "E60"} ${vehicleInfo.year || ""} ${vehicleInfo.mileage ? `(${vehicleInfo.mileage} km)` : ""}`
    : "Vehicle: BMW E60 (2003-2010)"

  const prompt = `You are an expert BMW mechanic and diagnostic specialist. Analyze these diagnostic trouble codes (DTCs) from a ${vehicleContext}.

DTCs Found:
${JSON.stringify(dtcContext, null, 2)}

Provide a comprehensive analysis in Spanish including:
1. A brief summary of the overall vehicle condition
2. Priority order for addressing the codes (most critical first)
3. Combined diagnosis - how these codes might be related to each other
4. Specific recommended actions
5. If there are any safety concerns, mention them

Focus on:
- BMW-specific issues and common failure points
- Cost-effective repair strategies
- Which repairs might fix multiple codes
- Whether the car is safe to drive

Respond in JSON format matching this structure:
{
  "summary": "Brief overall assessment in Spanish",
  "priority_order": ["code1", "code2"],
  "combined_diagnosis": "How codes relate to each other in Spanish",
  "recommended_actions": ["action1", "action2"],
  "safety_warning": "Only if there are safety concerns, in Spanish"
}

Be concise but thorough. Use technical terminology appropriately.`

  const client = getOpenAI()
  if (!client) {
    // Return basic analysis without AI if API key not configured
    return {
      dtcs: knownDtcs,
      summary: `Se encontraron ${dtcs.length} código(s) de error. API de IA no configurada.`,
      priority_order: dtcs.map((d) => d.code),
      combined_diagnosis: "Análisis basado en base de datos local (sin IA).",
      recommended_actions: [
        "Revisar los códigos individualmente en la base de datos",
        "Consultar con un mecánico especializado en BMW",
      ],
    }
  }

  try {
    const completion = await client.chat.completions.create({
      model: "gpt-4o-mini",
      messages: [
        {
          role: "system",
          content:
            "You are an expert BMW diagnostic specialist. Always respond with valid JSON. Provide analysis in Spanish.",
        },
        { role: "user", content: prompt },
      ],
      response_format: { type: "json_object" },
      temperature: 0.7,
      max_tokens: 1500,
    })

    const aiResponse = JSON.parse(
      completion.choices[0].message.content || "{}"
    ) as {
      summary: string
      priority_order: string[]
      combined_diagnosis: string
      recommended_actions: string[]
      safety_warning?: string
    }

    return {
      dtcs: knownDtcs,
      summary: aiResponse.summary || "No se pudo generar un resumen.",
      priority_order: aiResponse.priority_order || dtcs.map((d) => d.code),
      combined_diagnosis: aiResponse.combined_diagnosis || "",
      recommended_actions: aiResponse.recommended_actions || [],
      safety_warning: aiResponse.safety_warning,
    }
  } catch (error) {
    console.error("OpenAI API error:", error)

    // Return basic analysis without AI if API fails
    return {
      dtcs: knownDtcs,
      summary: `Se encontraron ${dtcs.length} código(s) de error. No se pudo conectar con el servicio de IA para análisis detallado.`,
      priority_order: dtcs
        .sort((a, b) => {
          // Sort by confirmed > pending > stored
          if (a.status.confirmed !== b.status.confirmed)
            return a.status.confirmed ? -1 : 1
          if (a.status.pending !== b.status.pending)
            return a.status.pending ? -1 : 1
          return 0
        })
        .map((d) => d.code),
      combined_diagnosis:
        "Análisis automático basado en base de datos local.",
      recommended_actions: [
        "Revisar los códigos individualmente",
        "Consultar con un mecánico especializado en BMW",
      ],
    }
  }
}

// Simple code lookup without AI
export function lookupDtcCode(code: string): DtcAnalysis | null {
  return lookupDtc(code)
}

// Get all known codes
export function getAllKnownCodes(): string[] {
  return [
    ...Object.keys(genericCodes),
    ...Object.keys(bmwCodes),
  ]
}
