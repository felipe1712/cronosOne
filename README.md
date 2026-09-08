# ExposureIQ — Servicio de Monitoreo Ejecutivo e Inteligencia de Riesgo

Servicio de monitoreo continuo de entorno de negocio y exposición digital diseñado para el **Director de Operaciones de una Aseguradora en México**.

- **Inteligencia de Entorno**: Ingesta diaria de boletines Coparmex (PDF 40+ páginas), procesamiento con Surya OCR, segmentación temática por riesgos y marco normativo, y síntesis ejecutiva con Claude API (Anthropic).
- **Inteligencia de Exposición (OSINT)**: Monitoreo continuo de menciones y posibles filtraciones en redes y foros/marketplaces de Dark Web (mediante `world-intel-mcp`), con bandeja de validación humana para analistas de seguridad.
- **Entrega Ejecutiva Unificada**: Notificaciones matutinas y alertas críticas enviadas directamente por WhatsApp a través de **WAHA** orquestado con **n8n** por sondeo periódico (`GET /api/mensajes/pendientes`).

---

## Estructura del Repositorio

- `backend/`: API principal en **Rust (Axum + SQLx + PostgreSQL)** con autenticación JWT, subida de PDFs y cola unificada de mensajes.
- `workers/`: Workers en **Python (FastAPI)** para orquestación de OCR Surya, segmentación temática, síntesis Claude y motor de reglas OSINT.
- `frontend/`: Panel web administrativo en **Next.js 15 (App Router, TypeScript, Material-UI v7)** con tema ejecutivo oscuro/claro.
- `n8n/`: Flujo exportable `exposureiq_waha_polling_workflow.json` para importar en n8n.
- `docker-compose.yml`: Configuración multi-contenedor para desarrollo y producción.
- `DEPLOYMENT_GUIDE.md`: Instructivo completo paso a paso para desplegar en `monitoreo.causer.com.mx` compartiendo infraestructura con SentinelIQ.
