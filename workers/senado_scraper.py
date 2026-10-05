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
    "detalle_secciones": [],
    "ultimo_inicio": None,
    "ultimo_fin": None,
    "mensaje": "Scraper inactivo"
}

def get_scraper_status() -> Dict[str, Any]:
    return scraper_status

def build_candidate_urls(sec: Dict[str, Any], fecha_slash: str, dom_links: Optional[Dict[str, str]] = None) -> List[str]:
    """
    Construye una lista ordenada de URLs candidatas para una sección, probando:
    1. Enlaces descubiertos dinámicamente en el DOM de la página.
    2. Ruta estándar con /SINTESIS/
    3. Ruta directa sin /SINTESIS/
    4. Nombres en minúsculas
    """
    sec_archivo = sec.get("archivo", "")
    sec_id = sec.get("id", "")
    candidates = []

    # 1. Enlaces descubiertos en DOM
    if dom_links:
        upper_file = sec_archivo.upper()
        if upper_file in dom_links:
            candidates.append(dom_links[upper_file])
        for k, v in dom_links.items():
            if (sec_archivo.lower() in k.lower() or sec_id.lower() in k.lower()) and v not in candidates:
                candidates.append(v)

    # 2. Rutas estándar conocidas
    base = "https://comunicacionsocial.senado.gob.mx/sintesis/book"
    std_urls = [
        f"{base}/{fecha_slash}/SINTESIS/{sec_archivo}",
        f"{base}/{fecha_slash}/{sec_archivo}",
        f"{base}/{fecha_slash}/SINTESIS/{sec_archivo.lower()}",
        f"{base}/{fecha_slash}/{sec_archivo.lower()}",
    ]
    for u in std_urls:
        if u not in candidates:
            candidates.append(u)

    return candidates

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

async def run_senado_scraper_pipeline(fecha_param: Optional[str] = None, secciones_param: Optional[List[str]] = None) -> Dict[str, Any]:
    """
    Ejecuta el ciclo completo de descarga del Senado:
    1. Resuelve la fecha objetivo (YYYY-MM-DD).
    2. Inicia el Headless Browser (Lightpanda o Chromium).
    3. Descarga las secciones seleccionadas de PDF (excluyendo Cartones).
    4. Guarda los archivos en disco.
    5. Inserta en la base de datos PostgreSQL con origen='senado' e incluido_en_sintesis=TRUE.
    6. Dispara el procesamiento OCR y la segmentación.
    7. Deja el documento listo para la síntesis consolidada del día.
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

    # Filtrar secciones si se especificaron
    secciones_a_descargar = SECCIONES_SENADO
    if secciones_param and len(secciones_param) > 0:
        sec_targets = {str(s).strip().lower() for s in secciones_param}
        filtered = [
            s for s in SECCIONES_SENADO
            if s["id"].lower() in sec_targets
            or s["archivo"].lower() in sec_targets
            or s["nombre"].lower() in sec_targets
        ]
        if filtered:
            secciones_a_descargar = filtered

    scraper_status["en_progreso"] = True
    scraper_status["fecha_objetivo"] = fecha_iso
    scraper_status["archivos_descargados"] = []
    scraper_status["archivos_procesados"] = []
    scraper_status["errores"] = []
    scraper_status["detalle_secciones"] = []
    scraper_status["ultimo_inicio"] = datetime.now().isoformat()
    scraper_status["mensaje"] = f"Iniciando descarga de {len(secciones_a_descargar)} secciones del Senado para {fecha_iso}..."

    print(f"\n[Scraper Senado] ========================================================")
    print(f"[Scraper Senado] Iniciando trabajo para la fecha: {fecha_iso} ({fecha_slash})")
    print(f"[Scraper Senado] Secciones a descargar: {len(secciones_a_descargar)} de {len(SECCIONES_SENADO)}")
    print(f"[Scraper Senado] ========================================================")

    upload_dir = get_target_upload_dir(fecha_iso)
    from db import ensure_database_schema
    ensure_database_schema()
    from main import run_bulletin_pipeline

    try:
        async with async_playwright() as p:
            browser, context, browser_type = await acquire_browser(p)
            scraper_status["navegador_usado"] = browser_type

            page = await context.new_page()

            # 1. Visitar el portal para inicializar sesión y resolver el WAF Incapsula
            print(f"[Scraper Senado] Navegando a {settings.senado_sintesis_url}...")
            scraper_status["mensaje"] = "Accediendo al portal del Senado y validando seguridad WAF..."
            dom_links = {}
            try:
                await page.goto(
                    "https://comunicacionsocial.senado.gob.mx/sintesis/sintesis.html",
                    wait_until="domcontentloaded",
                    timeout=30000
                )
                await page.wait_for_timeout(3000)
                # Extraer enlaces dinámicos reales disponibles en la página
                dom_links = await page.evaluate("""
                    () => {
                        const links = {};
                        const elements = document.querySelectorAll('a[href], iframe[src], embed[src], source[src]');
                        elements.forEach(el => {
                            const url = el.href || el.src;
                            if (url && (url.toLowerCase().includes('.pdf') || url.toLowerCase().includes('/book/'))) {
                                const filename = url.split('/').pop().split('?')[0].toUpperCase();
                                links[filename] = url;
                            }
                        });
                        return links;
                    }
                """)
                if dom_links:
                    print(f"[Scraper Senado] Enlaces dinámicos descubiertos en DOM: {list(dom_links.keys())}")
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

            # 2. Descargar las secciones seleccionadas
            async with httpx.AsyncClient(headers=headers, timeout=60.0, follow_redirects=True) as http_client:
                for sec in secciones_a_descargar:
                    sec_id = sec["id"]
                    sec_nombre = sec["nombre"]
                    sec_archivo = sec["archivo"]

                    scraper_status["mensaje"] = f"Descargando sección [{sec['orden']}/10]: {sec_nombre}..."
                    candidate_urls = build_candidate_urls(sec, fecha_slash, dom_links)
                    print(f"[Scraper Senado] Consultando {sec_nombre} ({len(candidate_urls)} URLs candidatas)...")

                    file_bytes = b""
                    download_success = False
                    successful_url = ""
                    last_status = 404
                    last_err_msg = ""

                    for url_pdf in candidate_urls:
                        # Intento 1: Descarga HTTP con cookies de sesión
                        try:
                            resp = await http_client.get(url_pdf)
                            last_status = resp.status_code
                            if resp.status_code == 200 and resp.content.startswith(b"%PDF-"):
                                file_bytes = resp.content
                                download_success = True
                                successful_url = url_pdf
                                print(f"[Scraper Senado] ✅ Descarga exitosa vía HTTP de {url_pdf}: {len(file_bytes)} bytes")
                                break
                            elif resp.status_code == 200:
                                last_err_msg = f"HTTP 200 pero contenido no es PDF ({resp.headers.get('content-type', 'desconocido')})"
                            else:
                                last_err_msg = f"HTTP {resp.status_code}"
                        except Exception as http_err:
                            last_err_msg = f"Error de red: {http_err}"
                            print(f"[Scraper Senado] Aviso HTTP al consultar {url_pdf}: {http_err}")

                        # Intento 2 (Fallback): Navegar y descargar a través del contexto del navegador
                        if not download_success:
                            try:
                                page_resp = await page.request.get(url_pdf)
                                last_status = page_resp.status
                                if page_resp.status == 200:
                                    b_content = await page_resp.body()
                                    if b_content.startswith(b"%PDF-"):
                                        file_bytes = b_content
                                        download_success = True
                                        successful_url = url_pdf
                                        print(f"[Scraper Senado] ✅ Descarga exitosa vía Browser Context de {url_pdf}: {len(file_bytes)} bytes")
                                        break
                                    else:
                                        last_err_msg = "Respuesta del navegador no es PDF"
                            except Exception as br_err:
                                last_err_msg = f"Error navegador: {br_err}"

                    if not download_success or len(file_bytes) < 100:
                        if last_status == 404:
                            motivo = f"No publicado por el Senado para la fecha {fecha_iso} (HTTP 404)."
                        elif last_status == 403:
                            motivo = f"Acceso restringido por WAF / Incapsula (HTTP 403)."
                        else:
                            motivo = f"No disponible en servidor del Senado ({last_err_msg or f'Código {last_status}'})."

                        warn_msg = f"Sección '{sec_nombre}' ({sec_archivo}): {motivo}"
                        print(f"[Scraper Senado] ⚠️ {warn_msg}")
                        scraper_status["errores"].append(warn_msg)
                        scraper_status["detalle_secciones"].append({
                            "id": sec_id,
                            "nombre": sec_nombre,
                            "archivo": sec_archivo,
                            "estado": "no_disponible" if last_status == 404 else "error",
                            "codigo_http": last_status,
                            "tamano_bytes": 0,
                            "url_probada": candidate_urls[0] if candidate_urls else "",
                            "motivo": motivo
                        })
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

                    scraper_status["detalle_secciones"].append({
                        "id": sec_id,
                        "nombre": sec_nombre,
                        "archivo": sec_archivo,
                        "estado": "descargado",
                        "codigo_http": 200,
                        "tamano_bytes": len(file_bytes),
                        "url_probada": successful_url,
                        "motivo": f"Descargado exitosamente ({len(file_bytes) // 1024} KB)"
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
                            print(f"[Scraper Senado] Boletín ya registrado previamente (ID: {boletin_id}). Actualizando origen e inclusión...")
                            cur.execute(
                                """
                                UPDATE boletines
                                SET origen = 'senado', incluido_en_sintesis = TRUE, actualizado_en = now()
                                WHERE id = %s
                                """,
                                (boletin_id,)
                            )
                            conn.commit()
                        else:
                            import uuid
                            boletin_id = str(uuid.uuid4())
                            nombre_descriptivo = f"Senado — {sec_nombre} ({fecha_iso})"
                            cur.execute(
                                """
                                INSERT INTO boletines (id, fecha_boletin, nombre_archivo, ruta_archivo, estado, origen, incluido_en_sintesis)
                                VALUES (%s, %s, %s, %s, 'pendiente_ocr', 'senado', TRUE)
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


async def check_senado_availability(fecha_param: Optional[str] = None, secciones_param: Optional[List[str]] = None) -> Dict[str, Any]:
    """
    Verifica rápidamente la disponibilidad en el portal del Senado para cada sección solicitada,
    sin procesar OCR ni guardar archivos pesados.
    Retorna el estado de disponibilidad y el código HTTP de cada sección para diagnóstico.
    """
    if fecha_param:
        try:
            fecha_dt = datetime.strptime(fecha_param.strip(), "%Y-%m-%d")
        except ValueError:
            fecha_dt = datetime.now()
    else:
        fecha_dt = datetime.now()
        if fecha_dt.weekday() == 5:
            fecha_dt -= timedelta(days=1)
        elif fecha_dt.weekday() == 6:
            fecha_dt -= timedelta(days=2)

    yyyy = str(fecha_dt.year)
    mm = f"{fecha_dt.month:02d}"
    dd = f"{fecha_dt.day:02d}"
    fecha_iso = f"{yyyy}-{mm}-{dd}"
    fecha_slash = f"{yyyy}/{mm}/{dd}"

    secciones_a_verificar = SECCIONES_SENADO
    if secciones_param and len(secciones_param) > 0:
        sec_targets = {str(s).strip().lower() for s in secciones_param}
        filtered = [
            s for s in SECCIONES_SENADO
            if s["id"].lower() in sec_targets
            or s["archivo"].lower() in sec_targets
            or s["nombre"].lower() in sec_targets
        ]
        if filtered:
            secciones_a_verificar = filtered

    detalles = []
    total_disponibles = 0

    try:
        async with async_playwright() as p:
            browser, context, _ = await acquire_browser(p)
            page = await context.new_page()

            dom_links = {}
            try:
                await page.goto(
                    "https://comunicacionsocial.senado.gob.mx/sintesis/sintesis.html",
                    wait_until="domcontentloaded",
                    timeout=20000
                )
                await page.wait_for_timeout(2500)
                dom_links = await page.evaluate("""
                    () => {
                        const links = {};
                        const elements = document.querySelectorAll('a[href], iframe[src], embed[src], source[src]');
                        elements.forEach(el => {
                            const url = el.href || el.src;
                            if (url && (url.toLowerCase().includes('.pdf') || url.toLowerCase().includes('/book/'))) {
                                const filename = url.split('/').pop().split('?')[0].toUpperCase();
                                links[filename] = url;
                            }
                        });
                        return links;
                    }
                """)
            except Exception as nav_e:
                print(f"[Check Senado] Aviso en navegación inicial: {nav_e}")

            cookies = await context.cookies()
            cookie_header = "; ".join([f"{c['name']}={c['value']}" for c in cookies])
            headers = {
                "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36",
                "Referer": "https://comunicacionsocial.senado.gob.mx/sintesis/sintesis.html",
                "Cookie": cookie_header
            }

            async with httpx.AsyncClient(headers=headers, timeout=15.0, follow_redirects=True) as http_client:
                for sec in secciones_a_verificar:
                    candidate_urls = build_candidate_urls(sec, fecha_slash, dom_links)
                    disponible = False
                    tamano = None
                    last_status = 404
                    tested_url = candidate_urls[0] if candidate_urls else ""

                    for curl in candidate_urls:
                        tested_url = curl
                        try:
                            resp = await http_client.get(curl, headers={"Range": "bytes=0-500"})
                            last_status = resp.status_code
                            if resp.status_code in [200, 206]:
                                content_head = resp.content
                                if content_head.startswith(b"%PDF-") or "pdf" in resp.headers.get("content-type", "").lower():
                                    disponible = True
                                    c_len = resp.headers.get("content-length") or resp.headers.get("content-range")
                                    if c_len:
                                        try:
                                            if "/" in str(c_len):
                                                tamano = int(str(c_len).split("/")[-1])
                                            else:
                                                tamano = int(c_len)
                                        except Exception:
                                            pass
                                    break
                        except Exception:
                            last_status = 500

                    # Fallback vía browser request si http_client no confirmó pero no es 404 definitivo
                    if not disponible and last_status not in [404]:
                        try:
                            page_resp = await page.request.get(tested_url)
                            last_status = page_resp.status
                            if page_resp.status == 200:
                                head_b = await page_resp.body()
                                if head_b.startswith(b"%PDF-"):
                                    disponible = True
                                    tamano = len(head_b)
                        except Exception:
                            pass

                    if disponible:
                        total_disponibles += 1
                        kb_str = f" (~{tamano // 1024} KB)" if tamano and tamano > 0 else ""
                        motivo = f"Publicado y disponible para descarga en el portal del Senado{kb_str}"
                    elif last_status == 404:
                        motivo = f"No publicado por el Senado para la fecha {fecha_iso} (HTTP 404 Not Found)"
                    elif last_status == 403:
                        motivo = "Acceso restringido temporalmente por WAF / Incapsula (HTTP 403)"
                    else:
                        motivo = f"No disponible en servidor del Senado (Código {last_status})"

                    detalles.append({
                        "id": sec["id"],
                        "nombre": sec["nombre"],
                        "archivo": sec["archivo"],
                        "disponible": disponible,
                        "codigo_http": 200 if disponible else last_status,
                        "tamano_bytes": tamano,
                        "url_probada": tested_url,
                        "motivo": motivo
                    })

            await browser.close()

    except Exception as e:
        print(f"[Check Senado] Error general comprobando disponibilidad: {e}")
        for sec in secciones_a_verificar:
            detalles.append({
                "id": sec["id"],
                "nombre": sec["nombre"],
                "archivo": sec["archivo"],
                "disponible": False,
                "codigo_http": 500,
                "tamano_bytes": None,
                "url_probada": f"https://comunicacionsocial.senado.gob.mx/sintesis/book/{fecha_slash}/SINTESIS/{sec['archivo']}",
                "motivo": f"Error conectando con el portal del Senado: {str(e)[:90]}"
            })

    return {
        "fecha": fecha_iso,
        "total_secciones": len(secciones_a_verificar),
        "disponibles": total_disponibles,
        "no_disponibles": len(secciones_a_verificar) - total_disponibles,
        "detalles": detalles
    }
