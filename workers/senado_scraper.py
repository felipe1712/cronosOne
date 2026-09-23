"""
ExposureIQ — Módulo de Navegación y Scraper Automatizado: Senado de la República
Portal: https://comunicacionsocial.senado.gob.mx/sintesiss

Reglas:
1. Extrae las 10 secciones del menú izquierdo:
   - Síntesis Digital Informativa (SINTESIS.pdf)
   - Primeras Planas (PRIMERASPLANAS.pdf)
   - Primeras Planas Internacionales (PRIMERASPLANASINTERNACIONALES.pdf)
   - Redes (REDES.pdf)
   - Senado (SENADO.pdf)
   - Senadores Escriben (SENADORESESCRIBEN.pdf)
   - Columnas Senado (COLUMNAS_S.pdf)
   - Diputados (DIPUTADOS.pdf)
   - Panorama Nacional (PANORAMANACIONAL.pdf)
   - Columnas (COLUMNAS.pdf)
   * EXCLUIDA: Cartones (CARTONES.pdf)
2. Descarga los archivos en backend/uploads/auto_ingest/{YYYY-MM-DD}/
3. Procesa OCR con Surya / pypdf
4. Genera síntesis ejecutiva con Claude LLM aplicando CLAUDE_SYSTEM_PROMPT
5. Deja el estado en 'sintesis_lista' para el visto bueno humano en el panel (Opción A).
"""

import os
import hashlib
import json
import traceback
from datetime import datetime, timedelta
from typing import Optional, Dict, Any, List
import httpx
from playwright.async_api import async_playwright, Browser, BrowserContext

from config import settings
from db import get_db_connection
from surya_client import resolve_file_path

# Las 10 secciones solicitadas del menú lateral izquierdo (Cartones EXCLUIDA)
SECCIONES_SENADO = [
    {
        "id": "portada",
        "nombre": "Síntesis Digital Informativa",
        "archivo": "SINTESIS.pdf",
        "pestaña": "#Portada",
        "orden": 1
    },
    {
        "id": "primeras_planas",
        "nombre": "Primeras Planas",
        "archivo": "PRIMERASPLANAS.pdf",
        "pestaña": "#PrimerasPlanas",
        "orden": 2
    },
    {
        "id": "primeras_planas_int",
        "nombre": "Primeras Planas Internacionales",
        "archivo": "PRIMERASPLANASINTERNACIONALES.pdf",
        "pestaña": "#PrimerasPlanasInternacionales",
        "orden": 3
    },
    {
        "id": "redes",
        "nombre": "Redes",
        "archivo": "REDES.pdf",
        "pestaña": "#Redes",
        "orden": 4
    },
    {
        "id": "senado",
        "nombre": "Senado",
        "archivo": "SENADO.pdf",
        "pestaña": "#Senado",
        "orden": 5
    },
    {
        "id": "senadores_escriben",
        "nombre": "Senadores Escriben",
        "archivo": "SENADORESESCRIBEN.pdf",
        "pestaña": "#SenadoresEscriben",
        "orden": 6
    },
    {
        "id": "columnas_senado",
        "nombre": "Columnas Senado",
        "archivo": "COLUMNAS_S.pdf",
        "pestaña": "#ColumnasSenado",
        "orden": 7
    },
    {
        "id": "diputados",
        "nombre": "Diputados",
        "archivo": "DIPUTADOS.pdf",
        "pestaña": "#Diputados",
        "orden": 8
    },
    {
        "id": "panorama_nacional",
        "nombre": "Panorama Nacional",
        "archivo": "PANORAMANACIONAL.pdf",
        "pestaña": "#PanoramaNacional",
        "orden": 9
    },
    {
        "id": "columnas",
        "nombre": "Columnas",
        "archivo": "COLUMNAS.pdf",
        "pestaña": "#Columnas",
        "orden": 10
    }
]

# Estado global en memoria para reportar progreso en tiempo real al frontend
scraper_status: Dict[str, Any] = {
    "en_progreso": False,
    "fecha_objetivo": None,
    "navegador_usado": None,
    "archivos_descargados": [],
    "archivos_procesados": [],
    "errores": [],
    "ultimo_inicio": None,
    "ultimo_fin": None,
    "mensaje": "Scraper inactivo"
}

def get_scraper_status() -> Dict[str, Any]:
    return scraper_status

def compute_sha256(file_bytes: bytes) -> str:
    return hashlib.sha256(file_bytes).hexdigest()

def get_target_upload_dir(fecha_str: str) -> str:
    """Calcula y crea la ruta absoluta de almacenamiento para las descargas de auto_ingest."""
    clean_date = fecha_str.replace("/", "-")
    base_dir = settings.auto_ingest_dir
    if not base_dir:
        # Resolver relativo al backend del proyecto
        base_dir = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "backend", "uploads", "auto_ingest"))
    target_dir = os.path.join(base_dir, clean_date)
    os.makedirs(target_dir, exist_ok=True)
    return target_dir

async def acquire_browser(p) -> tuple[Browser, BrowserContext, str]:
    """
    Intenta conexión con Lightpanda por CDP (puerto 9222).
    Si no responde, realiza fallback automático a Chromium Headless local con Playwright.
    """
    cdp_url = settings.lightpanda_cdp_url
    if cdp_url:
        try:
            browser = await p.chromium.connect_over_cdp(cdp_url, timeout=3000)
            print(f"[Scraper] Conectado exitosamente a Lightpanda Browser vía CDP en {cdp_url}")
            ctx = browser.contexts[0] if browser.contexts else await browser.new_context()
            return browser, ctx, "lightpanda"
        except Exception as e:
            print(f"[Scraper] Lightpanda CDP no disponible en {cdp_url} ({e}). Utilizando Chromium Headless...")

    # Fallback local con Playwright Chromium
    browser = await p.chromium.launch(
        headless=True,
        args=[
            "--disable-blink-features=AutomationControlled",
            "--no-sandbox",
            "--disable-setuid-sandbox",
            "--disable-dev-shm-usage"
        ]
    )
    ctx = await browser.new_context(
        user_agent="Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36",
        locale="es-MX",
        viewport={"width": 1440, "height": 900}
    )
    return browser, ctx, "chromium_headless"

async def run_senado_scraper_pipeline(fecha_param: Optional[str] = None) -> Dict[str, Any]:
    """
    Ejecuta el ciclo completo de descarga del Senado:
    1. Resuelve la fecha objetivo (YYYY-MM-DD).
    2. Inicia el Headless Browser (Lightpanda o Chromium).
    3. Descarga las 10 secciones de PDF (excluyendo Cartones).
    4. Guarda los archivos en disco.
    5. Inserta en la base de datos PostgreSQL.
    6. Dispara el procesamiento OCR y la síntesis Claude.
    7. Marca el estado en 'sintesis_lista' para el visto bueno humano.
    """
    global scraper_status
    if scraper_status["en_progreso"]:
        return {"status": "en_progreso", "mensaje": "Un trabajo de scraping ya está en ejecución"}

    # Determinar fecha objetivo
    if fecha_param:
        try:
            fecha_dt = datetime.strptime(fecha_param, "%Y-%m-%d")
        except ValueError:
            fecha_dt = datetime.now()
    else:
        # Por defecto la fecha actual, o el viernes anterior si es fin de semana
        fecha_dt = datetime.now()
        if fecha_dt.weekday() == 5: # Sábado
            fecha_dt -= timedelta(days=1)
        elif fecha_dt.weekday() == 6: # Domingo
            fecha_dt -= timedelta(days=2)

    yyyy = str(fecha_dt.year)
    mm = f"{fecha_dt.month:02d}"
    dd = f"{fecha_dt.day:02d}"
    fecha_iso = f"{yyyy}-{mm}-{dd}"
    fecha_slash = f"{yyyy}/{mm}/{dd}"

    scraper_status["en_progreso"] = True
    scraper_status["fecha_objetivo"] = fecha_iso
    scraper_status["archivos_descargados"] = []
    scraper_status["archivos_procesados"] = []
    scraper_status["errores"] = []
    scraper_status["ultimo_inicio"] = datetime.now().isoformat()
    scraper_status["mensaje"] = f"Iniciando descarga de síntesis del Senado para {fecha_iso}..."

    print(f"\n[Scraper Senado] ========================================================")
    print(f"[Scraper Senado] Iniciando trabajo para la fecha: {fecha_iso} ({fecha_slash})")
    print(f"[Scraper Senado] ========================================================")

    upload_dir = get_target_upload_dir(fecha_iso)
    from main import run_bulletin_pipeline

    try:
        async with async_playwright() as p:
            browser, context, browser_type = await acquire_browser(p)
            scraper_status["navegador_usado"] = browser_type

            page = await context.new_page()

            # 1. Visitar el portal para inicializar sesión y resolver el WAF Incapsula
            print(f"[Scraper Senado] Navegando a {settings.senado_sintesis_url}...")
            scraper_status["mensaje"] = "Accediendo al portal del Senado y validando seguridad WAF..."
            try:
                await page.goto(
                    "https://comunicacionsocial.senado.gob.mx/sintesis/sintesis.html",
                    wait_until="domcontentloaded",
                    timeout=30000
                )
                await page.wait_for_timeout(3000)
            except Exception as nav_err:
                print(f"[Scraper Senado] Advertencia en navegación inicial: {nav_err}")

            # Obtener cookies del navegador para peticiones autenticadas
            cookies = await context.cookies()
            cookie_header = "; ".join([f"{c['name']}={c['value']}" for c in cookies])

            headers = {
                "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36",
                "Referer": "https://comunicacionsocial.senado.gob.mx/sintesis/sintesis.html",
                "Cookie": cookie_header
            }

            # 2. Descargar cada una de las 10 secciones
            async with httpx.AsyncClient(headers=headers, timeout=60.0, follow_redirects=True) as http_client:
                for sec in SECCIONES_SENADO:
                    sec_id = sec["id"]
                    sec_nombre = sec["nombre"]
                    sec_archivo = sec["archivo"]

                    scraper_status["mensaje"] = f"Descargando sección [{sec['orden']}/10]: {sec_nombre}..."
                    url_pdf = f"https://comunicacionsocial.senado.gob.mx/sintesis/book/{fecha_slash}/SINTESIS/{sec_archivo}"
                    print(f"[Scraper Senado] Consultando {sec_nombre} -> {url_pdf}")

                    file_bytes = b""
                    download_success = False

                    # Intento 1: Descarga HTTP con cookies de sesión
                    try:
                        resp = await http_client.get(url_pdf)
                        if resp.status_code == 200 and resp.content.startswith(b"%PDF-"):
                            file_bytes = resp.content
                            download_success = True
                            print(f"[Scraper Senado] ✅ Descarga exitosa vía HTTP: {len(file_bytes)} bytes")
                    except Exception as http_err:
                        print(f"[Scraper Senado] Error HTTP al descargar {sec_archivo}: {http_err}")

                    # Intento 2 (Fallback): Navegar y descargar a través del contexto del navegador
                    if not download_success:
                        try:
                            page_resp = await page.request.get(url_pdf)
                            if page_resp.status == 200:
                                b_content = await page_resp.body()
                                if b_content.startswith(b"%PDF-"):
                                    file_bytes = b_content
                                    download_success = True
                                    print(f"[Scraper Senado] ✅ Descarga exitosa vía Browser Context: {len(file_bytes)} bytes")
                        except Exception as br_err:
                            print(f"[Scraper Senado] Error Browser al descargar {sec_archivo}: {br_err}")

                    if not download_success or len(file_bytes) < 100:
                        warn_msg = f"Sección '{sec_nombre}' ({sec_archivo}) no encontrada o no disponible para la fecha {fecha_iso}."
                        print(f"[Scraper Senado] ⚠️ {warn_msg}")
                        scraper_status["errores"].append(warn_msg)
                        continue

                    # Guardar archivo en disco
                    safe_filename = f"senado_{fecha_iso}_{sec_id}_{sec_archivo}"
                    file_path = os.path.join(upload_dir, safe_filename)
                    with open(file_path, "wb") as f_out:
                        f_out.write(file_bytes)

                    file_hash = compute_sha256(file_bytes)
                    print(f"[Scraper Senado] Archivo guardado en: {file_path} (SHA-256: {file_hash[:12]}...)")

                    scraper_status["archivos_descargados"].append({
                        "seccion": sec_nombre,
                        "archivo": safe_filename,
                        "bytes": len(file_bytes),
                        "hash": file_hash
                    })

                    # Registrar en PostgreSQL y encolar pipeline OCR + Claude
                    try:
                        conn = get_db_connection()
                        cur = conn.cursor()

                        # Verificar si ya existe este archivo para no duplicar procesamiento
                        cur.execute(
                            "SELECT id FROM boletines WHERE fecha_boletin = %s AND nombre_archivo = %s",
                            (fecha_iso, safe_filename)
                        )
                        existente = cur.fetchone()

                        if existente:
                            boletin_id = str(existente["id"])
                            print(f"[Scraper Senado] Boletín ya registrado previamente (ID: {boletin_id}). Reprocesando...")
                        else:
                            import uuid
                            boletin_id = str(uuid.uuid4())
                            nombre_descriptivo = f"Senado — {sec_nombre} ({fecha_iso})"
                            cur.execute(
                                """
                                INSERT INTO boletines (id, fecha_boletin, nombre_archivo, ruta_archivo, estado)
                                VALUES (%s, %s, %s, %s, 'pendiente_ocr')
                                RETURNING id
                                """,
                                (boletin_id, fecha_iso, nombre_descriptivo, os.path.abspath(file_path))
                            )
                            conn.commit()
                            print(f"[Scraper Senado] ✅ Registrado en BD con ID: {boletin_id}")

                        cur.close()
                        conn.close()

                        # Disparar pipeline OCR Surya y Síntesis Claude
                        scraper_status["mensaje"] = f"Procesando OCR y Síntesis para: {sec_nombre}..."
                        await run_bulletin_pipeline(boletin_id, os.path.abspath(file_path))

                        scraper_status["archivos_procesados"].append({
                            "boletin_id": boletin_id,
                            "seccion": sec_nombre,
                            "estado": "sintesis_lista"
                        })

                    except Exception as db_pipe_err:
                        err_det = f"Error en pipeline de {sec_nombre}: {str(db_pipe_err)}"
                        print(f"[Scraper Senado] ❌ {err_det}")
                        scraper_status["errores"].append(err_det)

            await browser.close()

        total_descargados = len(scraper_status["archivos_descargados"])
        total_procesados = len(scraper_status["archivos_procesados"])
        scraper_status["mensaje"] = f"Finalizado con éxito. {total_descargados} secciones descargadas y {total_procesados} listas para Visto Bueno."
        print(f"\n[Scraper Senado] 🎉 Trabajo concluido. {scraper_status['mensaje']}")

    except Exception as e:
        err_msg = f"Fallo crítico en scraper del Senado: {str(e)}\n{traceback.format_exc()}"
        print(f"[Scraper Senado] ❌ {err_msg}")
        scraper_status["errores"].append(err_msg)
        scraper_status["mensaje"] = f"Error durante la ejecución: {str(e)[:200]}"
    finally:
        scraper_status["en_progreso"] = False
        scraper_status["ultimo_fin"] = datetime.now().isoformat()

    return scraper_status
