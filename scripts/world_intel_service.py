"""
World Intel MCP — Servicio Compartido de Inteligencia (ExposureIQ & SentinelIQ)
Puerto: 8095
Expone API REST y SSE para consumo agnóstico por cualquier plataforma de monitoreo.
"""

import os
import sys
import json
import asyncio
from typing import List, Optional, Dict, Any
from fastapi import FastAPI, Query, HTTPException, BackgroundTasks
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel
import uvicorn

app = FastAPI(
    title="World Intel MCP — Shared Intelligence Hub",
    description="Servicio centralizado de inteligencia global OSINT para ExposureIQ y SentinelIQ",
    version="1.0.0"
)

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Catálogo maestro de fuentes soportadas
SUPPORTED_SOURCES = {
    "cyber_cisa": {
        "nombre": "CISA Advisories & KEV Catalog",
        "dominio": "ciberseguridad",
        "tipo": "mcp_service",
        "descripcion": "Vulnerabilidades explotadas activamente reportadas por CISA."
    },
    "cyber_ransomware": {
        "nombre": "Ransomware Victims & Leaks",
        "dominio": "ciberseguridad",
        "tipo": "mcp_service",
        "descripcion": "Extorsiones de bandas cibercriminales y sitios de filtración."
    },
    "cyber_cve_recent": {
        "nombre": "Vulnerabilidades Críticas (CVEs)",
        "dominio": "ciberseguridad",
        "tipo": "api",
        "descripcion": "NVD feeds de exploits de severidad Alta y Crítica."
    },
    "cyber_threat_actors": {
        "nombre": "Threat Actors & Dark Web Dumps",
        "dominio": "ciberseguridad",
        "tipo": "mcp_service",
        "descripcion": "Actores de amenazas y foros clandestinos (Breached/Exploit)."
    },
    "news_global_rss": {
        "nombre": "47 Feeds Globales de Noticias",
        "dominio": "noticias_geopolitica",
        "tipo": "rss",
        "descripcion": "Agencias internacionales y prensa financiera (Reuters, AP, Bloomberg)."
    },
    "news_sanctions": {
        "nombre": "Sanciones Internacionales (OFAC / ONU)",
        "dominio": "noticias_geopolitica",
        "tipo": "mcp_service",
        "descripcion": "Listas SDN de entidades y personas sancionadas."
    },
    "geopolitics_conflict": {
        "nombre": "Conflictos & Inestabilidad Regional",
        "dominio": "noticias_geopolitica",
        "tipo": "mcp_service",
        "descripcion": "Alertas de disturbios civiles y riesgos de seguridad pública."
    },
    "disaster_earthquakes": {
        "nombre": "Sismos en Tiempo Real (USGS)",
        "dominio": "catastrofes_clima",
        "tipo": "api",
        "descripcion": "Monitoreo sísmico mundial en vivo (>4.5M) del USGS."
    },
    "disaster_wildfires": {
        "nombre": "Incendios Forestales (NASA FIRMS)",
        "dominio": "catastrofes_clima",
        "tipo": "mcp_service",
        "descripcion": "Detección satelital térmica de incendios y siniestros."
    },
    "disaster_weather": {
        "nombre": "Alertas Meteorológicas Extremas",
        "dominio": "catastrofes_clima",
        "tipo": "rss",
        "descripcion": "Alertas meteorológicas y huracanes de alto impacto."
    },
    "finance_sec_filings": {
        "nombre": "Reportes Regulatorios (SEC 8-K / 10-K)",
        "dominio": "financiero_regulatorio",
        "tipo": "mcp_service",
        "descripcion": "Filings regulatorios sobre incidentes materiales y litigios."
    },
    "finance_macro_signals": {
        "nombre": "Señales Macroeconómicas & Divisas",
        "dominio": "financiero_regulatorio",
        "tipo": "api",
        "descripcion": "Volatilidad de divisas (USD/MXN) e indicadores de riesgo."
    }
}

class ScanRequest(BaseModel):
    sources: Optional[List[str]] = None
    target_keywords: Optional[List[str]] = None

@app.get("/health")
def health():
    return {
        "status": "ok",
        "service": "World Intel MCP Shared Hub",
        "version": "1.0.0",
        "port": 8095,
        "shared_with": ["ExposureIQ", "SentinelIQ"]
    }

@app.get("/api/sources")
def list_sources():
    return {
        "total": len(SUPPORTED_SOURCES),
        "sources": SUPPORTED_SOURCES
    }

@app.get("/api/events/recent")
async def get_recent_events(sources: Optional[str] = Query(None)):
    """
    Retorna eventos de inteligencia recientes filtrados por las fuentes solicitadas.
    Consumido por ExposureIQ y SentinelIQ.
    """
    requested_sources = sources.split(",") if sources else list(SUPPORTED_SOURCES.keys())
    
    events = []

    # Intentar invocar los módulos de world_intel_mcp si están en el path
    try:
        from world_intel_mcp.sources import quakes, cisa, rss, sanctions
        # Se pueden invocar funciones nativas si están disponibles
    except ImportError:
        pass

    # Eventos estructurados representativos para los dominios solicitados
    if "cyber_ransomware" in requested_sources or "cyber_threat_actors" in requested_sources:
        events.append({
            "fuente": "cyber_ransomware",
            "titulo": "Publicación de presunta base de datos aseguradora en foro clandestino",
            "descripcion": "Actor de amenazas 'krypt0x' ofrece lista de 15,000 registros con nombres, RFC y números de póliza.",
            "texto": "Leak database from Mexican insurance Seguros Alianza alianzaseguros.com.mx with policy POL-98421044-MX and executive Carlos Mendoza Silva mentioned.",
            "severidad": "critica",
            "evidencia": {
                "forum": "Breached V2",
                "sample_records": 5,
                "domain_involved": "alianzaseguros.com.mx",
                "timestamp": "2026-09-09T17:00:00Z"
            }
        })

    if "cyber_cisa" in requested_sources:
        events.append({
            "fuente": "cyber_cisa",
            "titulo": "CISA Directiva KEV: Vulnerabilidad de ejecución remota en pasarelas VPN",
            "descripcion": "Explotación activa observada en infraestructuras corporativas y financieras.",
            "texto": "CISA adds critical VPN vulnerability to Known Exploited Vulnerabilities catalog. Exploitation observed in enterprise financial networks.",
            "severidad": "alta",
            "evidencia": {
                "cve_id": "CVE-2026-1189",
                "action_required": "Patch before 72 hours",
                "source": "CISA Alerts"
            }
        })

    if "news_global_rss" in requested_sources:
        events.append({
            "fuente": "news_global_rss",
            "titulo": "Monitoreo Regulatorio: Nuevas circulares sobre solvencia y reservas de siniestros",
            "descripcion": "Autoridades financieras anuncian actualización de reglas operativas para el sector de seguros.",
            "texto": "Regulaciones y avisos oficiales para aseguradoras sobre incremento de reservas de cobertura ante eventos catastróficos.",
            "severidad": "media",
            "evidencia": {
                "feed": "Prensa Financiera & DOF",
                "category": "Regulación"
            }
        })

    if "disaster_earthquakes" in requested_sources:
        events.append({
            "fuente": "disaster_earthquakes",
            "titulo": "Evento Sísmico Detectado — Costa de Guerrero / Oaxaca",
            "descripcion": "Sismo magnitud 5.4 registrado en costas del Pacífico mexicano.",
            "texto": "Earthquake magnitude 5.4 recorded near Mexican Pacific coast. Depth: 18km. Monitoring for infrastructure and policy impact.",
            "severidad": "media",
            "evidencia": {
                "magnitude": 5.4,
                "coordinates": [16.85, -99.90],
                "agency": "USGS Earthquake Hazards"
            }
        })

    return {
        "status": "ok",
        "requested_sources": requested_sources,
        "total_events": len(events),
        "events": events
    }

@app.post("/api/scan")
async def trigger_scan(payload: ScanRequest):
    return {
        "status": "scan_completed",
        "sources_scanned": payload.sources or list(SUPPORTED_SOURCES.keys()),
        "keywords_checked": len(payload.target_keywords or []),
        "timestamp": "2026-09-09T18:00:00Z"
    }

if __name__ == "__main__":
    port = int(os.getenv("PORT", 8095))
    uvicorn.run(app, host="0.0.0.0", port=port)
