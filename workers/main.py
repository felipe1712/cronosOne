import os
import json
import traceback
from fastapi import FastAPI, BackgroundTasks, HTTPException
from pydantic import BaseModel
from typing import Optional
from config import settings
from db import get_db_connection, ensure_database_schema
from surya_client import process_pdf_ocr
from segmentation import segment_bulletin
from llm_synthesis import generate_executive_brief
from osint_worker import evaluate_rules_and_record

app = FastAPI(
    title="ExposureIQ Workers API",
    description="Servicios de fondo: Ingesta OCR, Síntesis LLM y Motor OSINT",
    version="1.0.0"
)

@app.on_event("startup")
async def on_startup():
    ensure_database_schema()

class ProcessBoletinRequest(BaseModel):
    boletin_id: str
    ruta_archivo: str

async def run_bulletin_pipeline(boletin_id: str, ruta_archivo: str):
    conn = get_db_connection()
    cur = conn.cursor()

    try:
        # 1. Marcar estado en_ocr
        cur.execute(
            "UPDATE boletines SET estado = 'en_ocr', actualizado_en = now() WHERE id = %s RETURNING fecha_boletin",
            (boletin_id,)
        )
        row = cur.fetchone()
        fecha_str = str(row["fecha_boletin"]) if row else "hoy"
        conn.commit()

        # 2. Extracción OCR (Surya o fallback)
        print(f"[Pipeline] Iniciando OCR para boletín {boletin_id}...")
        pages = await process_pdf_ocr(ruta_archivo)
        total_paginas = len(pages)

        # 3. Segmentación temática
        print(f"[Pipeline] Segmentando {total_paginas} páginas...")
        secciones = segment_bulletin(pages)

        # 4. Guardar secciones en PostgreSQL
        for sec in secciones:
            cur.execute(
                """
                INSERT INTO secciones (boletin_id, orden, tema, pagina_inicio, pagina_fin, contenido)
                VALUES (%s, %s, %s, %s, %s, %s)
                """,
                (
                    boletin_id,
                    sec["orden"],
                    sec["tema"],
                    sec["pagina_inicio"],
                    sec["pagina_fin"],
                    json.dumps(sec["contenido"])
                )
            )

        cur.execute(
            "UPDATE boletines SET estado = 'ocr_completo', total_paginas = %s, actualizado_en = now() WHERE id = %s",
            (total_paginas, boletin_id)
        )
        conn.commit()

        # 5. Síntesis LLM con Claude API
        print(f"[Pipeline] Generando síntesis ejecutiva con Claude API...")
        brief_texto, temas = await generate_executive_brief(secciones, fecha_str)

        # 6. Guardar síntesis
        cur.execute(
            """
            INSERT INTO sintesis_generadas (boletin_id, texto, temas, modelo_usado)
            VALUES (%s, %s, %s, %s)
            RETURNING id
            """,
            (boletin_id, brief_texto, temas, settings.claude_model)
        )
        conn.commit()

        # 7. Marcar síntesis lista para revisión y visto bueno humano
        cur.execute(
            "UPDATE boletines SET estado = 'sintesis_lista', actualizado_en = now() WHERE id = %s",
            (boletin_id,)
        )
        conn.commit()
        print(f"[Pipeline] ✅ Boletín {boletin_id} procesado exitosamente. Síntesis generada; esperando visto bueno en la interfaz web.")

    except Exception as e:
        conn.rollback()
        err_msg = f"Error en pipeline: {str(e)}\n{traceback.format_exc()}"
        print(f"[Pipeline] ❌ {err_msg}")
        cur.execute(
            "UPDATE boletines SET estado = 'error_sintesis', error_mensaje = %s, actualizado_en = now() WHERE id = %s",
            (str(e)[:500], boletin_id)
        )
        conn.commit()
    finally:
        cur.close()
        conn.close()

@app.get("/health")
def health():
    return {"status": "ok", "service": "ExposureIQ Python Workers"}

@app.post("/api/process-boletin")
async def process_boletin_endpoint(payload: ProcessBoletinRequest, background_tasks: BackgroundTasks):
    background_tasks.add_task(run_bulletin_pipeline, payload.boletin_id, payload.ruta_archivo)
    return {
        "status": "encolado",
        "boletin_id": payload.boletin_id,
        "mensaje": "Pipeline de OCR y síntesis iniciado en segundo plano"
    }

@app.post("/api/run-osint-scan")
def run_osint_scan_endpoint(background_tasks: BackgroundTasks):
    background_tasks.add_task(evaluate_rules_and_record)
    return {
        "status": "encolado",
        "mensaje": "Escaneo de inteligencia OSINT iniciado contra entidades vigiladas"
    }

class TestClaudeRequest(BaseModel):
    model: Optional[str] = None

@app.post("/api/test-claude")
async def test_claude_endpoint(payload: TestClaudeRequest):
    import time
    from llm_synthesis import test_claude_model, get_configured_model
    target_model = payload.model or get_configured_model()
    start_time = time.time()
    try:
        reply = await test_claude_model(target_model)
        latency = int((time.time() - start_time) * 1000)
        return {
            "ok": True,
            "model": target_model,
            "latency_ms": latency,
            "reply": reply,
            "mensaje": f"Conexión exitosa con Claude ({target_model}) en {latency}ms."
        }
    except Exception as e:
        latency = int((time.time() - start_time) * 1000)
        return {
            "ok": False,
            "model": target_model,
            "latency_ms": latency,
            "error": str(e),
            "mensaje": f"Fallo al conectar con {target_model}: {str(e)}"
        }

class ScrapeSenadoRequest(BaseModel):
    fecha: Optional[str] = None
    secciones: Optional[list[str]] = None

@app.post("/api/scrape-senado")
async def scrape_senado_endpoint(payload: Optional[ScrapeSenadoRequest] = None, background_tasks: BackgroundTasks = None):
    from senado_scraper import run_senado_scraper_pipeline, get_scraper_status
    status = get_scraper_status()
    if status["en_progreso"]:
        return {
            "status": "en_progreso",
            "mensaje": "El scraper del Senado ya se encuentra en ejecución",
            "detalles": status
        }
    fecha = payload.fecha if payload else None
    secciones = payload.secciones if payload else None
    if background_tasks:
        background_tasks.add_task(run_senado_scraper_pipeline, fecha, secciones)
    else:
        import asyncio
        asyncio.create_task(run_senado_scraper_pipeline(fecha, secciones))

    return {
        "status": "encolado",
        "mensaje": f"Scraper del Senado iniciado en segundo plano para la fecha {fecha or 'más reciente'}",
        "fecha": fecha,
        "secciones": secciones
    }

@app.get("/api/scrape-senado/status")
def scrape_senado_status_endpoint():
    from senado_scraper import get_scraper_status
    return get_scraper_status()

@app.get("/api/senado/secciones")
def get_senado_secciones_endpoint():
    from senado_scraper import SECCIONES_SENADO
    return SECCIONES_SENADO

class ConsolidarSintesisDiariaRequest(BaseModel):
    fecha: str
    documentos_ids: Optional[list[str]] = None

@app.post("/api/consolidar-sintesis-diaria")
async def consolidar_sintesis_diaria_endpoint(payload: ConsolidarSintesisDiariaRequest):
    """
    Consolida las secciones de todos los boletines seleccionados para una fecha dada
    y genera una única síntesis ejecutiva del día mediante Claude LLM.
    Guarda el resultado en la tabla 'sintesis_diarias'.
    """
    fecha = payload.fecha.strip()
    conn = get_db_connection()
    cur = conn.cursor()
    try:
        # Asegurar esquema de sintesis_diarias y columnas requeridas
        cur.execute("""
            CREATE TABLE IF NOT EXISTS sintesis_diarias (
                id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                fecha           DATE UNIQUE NOT NULL,
                texto           TEXT NOT NULL DEFAULT '',
                temas           TEXT[] DEFAULT '{}',
                documentos_ids  UUID[] DEFAULT '{}',
                estado          VARCHAR(30) NOT NULL DEFAULT 'borrador',
                modelo_usado    VARCHAR(50) DEFAULT 'claude-sonnet-4-5-20250929',
                tokens_usados   INT,
                aprobado_por    UUID,
                creado_en       TIMESTAMPTZ NOT NULL DEFAULT now(),
                actualizado_en  TIMESTAMPTZ NOT NULL DEFAULT now()
            );
            ALTER TABLE boletines ADD COLUMN IF NOT EXISTS incluido_en_sintesis BOOLEAN NOT NULL DEFAULT TRUE;
            ALTER TABLE boletines ADD COLUMN IF NOT EXISTS origen VARCHAR(50) NOT NULL DEFAULT 'manual';
            ALTER TABLE sintesis_diarias ADD COLUMN IF NOT EXISTS temas TEXT[] DEFAULT '{}';
        """)
        conn.commit()

        # Obtener los boletines objetivo
        if payload.documentos_ids and len(payload.documentos_ids) > 0:
            clean_ids = [str(i).strip() for i in payload.documentos_ids if str(i).strip()]
            cur.execute(
                """
                SELECT id, nombre_archivo, ruta_archivo, origen, estado, total_paginas
                FROM boletines
                WHERE id::text = ANY(%s)
                ORDER BY creado_en ASC
                """,
                (clean_ids,)
            )
            boletines = cur.fetchall()
        else:
            boletines = []

        # Si no se encontraron por IDs específicos o no se pasaron, consultar por fecha e inclusión
        if not boletines:
            cur.execute(
                """
                SELECT id, nombre_archivo, ruta_archivo, origen, estado, total_paginas
                FROM boletines
                WHERE fecha_boletin = %s AND (incluido_en_sintesis = TRUE OR incluido_en_sintesis IS NULL)
                ORDER BY creado_en ASC
                """,
                (fecha,)
            )
            boletines = cur.fetchall()

        # Si aún no hay con inclusión = true, traer todos los de la fecha
        if not boletines:
            cur.execute(
                """
                SELECT id, nombre_archivo, ruta_archivo, origen, estado, total_paginas
                FROM boletines
                WHERE fecha_boletin = %s
                ORDER BY creado_en ASC
                """,
                (fecha,)
            )
            boletines = cur.fetchall()

        if not boletines:
            raise HTTPException(
                status_code=400,
                detail=f"No hay boletines seleccionados o disponibles para la fecha {fecha}"
            )

        # Preparar documentos y sus secciones
        documentos_data = []
        doc_uuids = []

        from surya_client import resolve_file_path

        for b in boletines:
            b_id = str(b["id"])
            doc_uuids.append(b_id)
            doc_nombre = b["nombre_archivo"] or f"Boletín {b_id[:8]}"
            doc_origen = b.get("origen", "manual") or "manual"

            # Buscar secciones ya procesadas
            cur.execute(
                """
                SELECT tema, contenido, pagina_inicio, pagina_fin, orden
                FROM secciones
                WHERE boletin_id = %s
                ORDER BY orden ASC
                """,
                (b_id,)
            )
            secs = cur.fetchall()

            # Si el documento aún no tiene secciones y el archivo existe, intentar procesar OCR
            raw_path = b.get("ruta_archivo", "")
            resolved_path = resolve_file_path(raw_path) if raw_path else None

            if not secs and resolved_path and os.path.exists(resolved_path):
                try:
                    print(f"[Consolidar] Procesando OCR para {doc_nombre} ({b_id})...")
                    pages = await process_pdf_ocr(resolved_path)
                    if pages:
                        secs_parsed = segment_bulletin(pages)
                        for sec in secs_parsed:
                            cur.execute(
                                """
                                INSERT INTO secciones (boletin_id, orden, tema, pagina_inicio, pagina_fin, contenido)
                                VALUES (%s, %s, %s, %s, %s, %s)
                                """,
                                (b_id, sec["orden"], sec["tema"], sec["pagina_inicio"], sec["pagina_fin"], json.dumps(sec["contenido"]))
                            )
                        cur.execute(
                            "UPDATE boletines SET estado = 'ocr_completo', total_paginas = %s WHERE id = %s",
                            (len(pages), b_id)
                        )
                        conn.commit()
                        secs = secs_parsed
                except Exception as proc_err:
                    print(f"[Consolidar] Advertencia: Error en OCR de {doc_nombre}: {proc_err}")

            documentos_data.append({
                "id": b_id,
                "nombre": doc_nombre,
                "origen": doc_origen,
                "secciones": secs or []
            })

        from llm_synthesis import generate_consolidated_daily_brief, get_configured_model
        active_model = get_configured_model()
        print(f"[Consolidar] Generando síntesis consolidada para {len(documentos_data)} documentos de {fecha} con {active_model}...")
        brief_texto, detected_topics = await generate_consolidated_daily_brief(documentos_data, fecha)

        # Preparar y validar UUIDs de documentos para PostgreSQL
        import uuid as uuid_lib
        valid_uuids = []
        for uid in doc_uuids:
            try:
                valid_uuids.append(str(uuid_lib.UUID(str(uid).strip())))
            except Exception:
                pass

        # Upsert en sintesis_diarias con fallback seguro
        row = None
        try:
            cur.execute(
                """
                INSERT INTO sintesis_diarias (fecha, texto, temas, documentos_ids, estado, modelo_usado, actualizado_en)
                VALUES (%s, %s, %s, %s::uuid[], 'sintesis_lista', %s, now())
                ON CONFLICT (fecha) DO UPDATE
                SET texto = EXCLUDED.texto,
                    temas = EXCLUDED.temas,
                    documentos_ids = EXCLUDED.documentos_ids,
                    estado = 'sintesis_lista',
                    modelo_usado = EXCLUDED.modelo_usado,
                    actualizado_en = now()
                RETURNING id, fecha, texto, temas, documentos_ids, estado, modelo_usado, creado_en, actualizado_en
                """,
                (fecha, brief_texto, detected_topics, valid_uuids, active_model)
            )
            row = cur.fetchone()
            conn.commit()
        except Exception as upsert_err:
            print(f"[Consolidar] Aviso: Upsert con cast uuid[] falló ({upsert_err}), intentando fallback...")
            conn.rollback()
            cur = conn.cursor()
            cur.execute(
                """
                INSERT INTO sintesis_diarias (fecha, texto, temas, estado, modelo_usado, actualizado_en)
                VALUES (%s, %s, %s, 'sintesis_lista', %s, now())
                ON CONFLICT (fecha) DO UPDATE
                SET texto = EXCLUDED.texto,
                    temas = EXCLUDED.temas,
                    estado = 'sintesis_lista',
                    modelo_usado = EXCLUDED.modelo_usado,
                    actualizado_en = now()
                RETURNING id, fecha, texto, temas, documentos_ids, estado, modelo_usado, creado_en, actualizado_en
                """,
                (fecha, brief_texto, detected_topics, active_model)
            )
            row = cur.fetchone()
            conn.commit()

        return {
            "status": "completado",
            "sintesis": {
                "id": str(row["id"]),
                "fecha": str(row["fecha"]),
                "texto": row["texto"],
                "temas": row["temas"] or [],
                "documentos_ids": [str(u) for u in (row["documentos_ids"] or [])],
                "estado": row["estado"],
                "modelo_usado": row["modelo_usado"],
                "creado_en": row["creado_en"].isoformat() if row.get("creado_en") else None,
                "actualizado_en": row["actualizado_en"].isoformat() if row.get("actualizado_en") else None,
            },
            "documentos_procesados": len(documentos_data)
        }

    except HTTPException:
        raise
    except Exception as e:
        conn.rollback()
        err_msg = f"Error al consolidar síntesis del día: {str(e)}\n{traceback.format_exc()}"
        print(f"[Consolidar] ❌ {err_msg}")
        raise HTTPException(status_code=500, detail=str(e))
    finally:
        cur.close()
        conn.close()


if __name__ == "__main__":
    import uvicorn
    uvicorn.run("main:app", host=settings.host, port=settings.port, reload=True)
