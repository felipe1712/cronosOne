import os
import json
import traceback
from fastapi import FastAPI, BackgroundTasks, HTTPException
from pydantic import BaseModel
from typing import Optional
from config import settings
from db import get_db_connection
from surya_client import process_pdf_ocr
from segmentation import segment_bulletin
from llm_synthesis import generate_executive_brief
from osint_worker import evaluate_rules_and_record

app = FastAPI(
    title="ExposureIQ Workers API",
    description="Servicios de fondo: Ingesta OCR, Síntesis LLM y Motor OSINT",
    version="1.0.0"
)

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


if __name__ == "__main__":
    import uvicorn
    uvicorn.run("main:app", host=settings.host, port=settings.port, reload=True)
