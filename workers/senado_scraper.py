"""
ExposureIQ — Módulo de Navegación y Scraper Automatizado: Senado de la República
Portal: https://comunicacionsocial.senado.gob.mx/sintesiss

Evasión de WAF / Incapsula:
- Utiliza Camoufox (motor anti-detección derivado de Firefox con spoofing de kernel C++).
- Fallback automático a Playwright (Lightpanda / Chromium Headless).
- Validación activa del reto de JavaScript (_Incapsula_Resource).
- Descarga directa a través del contexto del navegador y soporte para curl_cffi (impersonación de TLS JA3/JA4).
"""

import os
import hashlib
import json
import traceback
from contextlib import asynccontextmanager
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
    Construye una lista ordenada de URLs candidatas para una sección:
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
        base_dir = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "backend", "uploads", "auto_ingest"))
    target_dir = os.path.join(base_dir, clean_date)
    os.makedirs(target_dir, exist_ok=True)
    return target_dir

@asynccontextmanager
async def acquire_browser_session():
    """
    Context manager asíncrono para gestionar la sesión de navegación.
    Prioridad:
    1. Camoufox: Motor anti-detección basado en Firefox (núcleo de HeadlessX) con
       spoofing de huellas digitales a nivel C++ para sobrepasar Imperva Incapsula.
    2. Fallback a Lightpanda si se configuró un endpoint CDP accesible.
    3. Fallback a Chromium Headless estándar de Playwright.
    """
    # 1. Intentar Camoufox (motor anti-detección especializado)
    try:
        from camoufox.async_api import AsyncCamoufox
        camoufox = AsyncCamoufox(
            headless=True,
            humanize=True,
            geoip=False
        )
        browser = await camoufox.start()
        context = await browser.new_context(
            locale="es-MX",
            viewport={"width": 1440, "height": 900}
        )
        print("[Scraper Senado] 🦊 Navegador iniciado exitosamente con motor Camoufox (HeadlessX Anti-Detect).")
        try:
            yield browser, context, "camoufox"
        finally:
            try:
                await context.close()
            except Exception:
                pass
            try:
                await camoufox.close()
            except Exception:
                pass
        return
    except Exception as e:
        print(f"[Scraper Senado] Aviso: Camoufox no disponible en el entorno ({e}). Intentando Playwright fallback...")

    # 2. Fallback Playwright
    async with async_playwright() as p:
        browser = None
        context = None
        b_type = "chromium_headless"

        cdp_url = settings.lightpanda_cdp_url
        if cdp_url and cdp_url.strip():
            try:
                browser = await p.chromium.connect_over_cdp(cdp_url, timeout=3000)
                print(f"[Scraper Senado] Conectado a Lightpanda Browser vía CDP en {cdp_url}")
                context = browser.contexts[0] if browser.contexts else await browser.new_context()
                b_type = "lightpanda"
            except Exception as lp_err:
                print(f"[Scraper Senado] Lightpanda CDP no disponible en {cdp_url} ({lp_err}).")

        if not browser:
            browser = await p.chromium.launch(
                headless=True,
                args=[
                    "--disable-blink-features=AutomationControlled",
                    "--no-sandbox",
                    "--disable-setuid-sandbox",
                    "--disable-dev-shm-usage"
                ]
            )
            context = await browser.new_context(
                user_agent="Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36",
                locale="es-MX",
                viewport={"width": 1440, "height": 900}
            )

        try:
            yield browser, context, b_type
        finally:
            try:
                if context:
                    await context.close()
            except Exception:
                pass
            try:
                if browser:
                    await browser.close()
            except Exception:
                pass

async def resolve_senado_session(page, timeout_ms: int = 35000) -> Dict[str, str]:
    """
    Navega al portal de Comunicación Social del Senado y permite que el motor resuelva el
    reto de JavaScript de Incapsula (_Incapsula_Resource).
    Devuelve los enlaces dinámicos a PDFs encontrados en el DOM.
    """
    urls_to_try = [
        "https://comunicacionsocial.senado.gob.mx/sintesis/sintesis.html",
        "https://comunicacionsocial.senado.gob.mx/sintesis/",
        settings.senado_sintesis_url
    ]

    dom_links = {}
    for target_url in urls_to_try:
        if not target_url:
            continue
        try:
            print(f"[Scraper Senado] Navegando a {target_url}...")
            await page.goto(target_url, wait_until="load", timeout=timeout_ms)

            # Esperar resolución de reto Incapsula (el script _Incapsula_Resource procesa y recarga)
            for step in range(12):
                content = await page.content()
                if "_Incapsula_Resource" in content and ("main-iframe" in content or "incident_id" in content):
                    await page.wait_for_timeout(1000)
                    continue
                break

            await page.wait_for_timeout(2000)

            # Extraer enlaces dinámicos del DOM
            extracted = await page.evaluate("""
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
            if extracted:
                dom_links.update(extracted)
                print(f"[Scraper Senado] ✅ Sesión validada. Enlaces descubiertos: {list(extracted.keys())}")
                break
            else:
                print(f"[Scraper Senado] Navegación a {target_url} completada.")
                break
        except Exception as nav_e:
            print(f"[Scraper Senado] Aviso en navegación inicial a {target_url}: {nav_e}")

    return dom_links

async def fetch_pdf_content(page, context, url_pdf: str, http_client: Optional[httpx.AsyncClient] = None) -> tuple[bool, bytes, int, str]:
    """
    Intenta descargar el archivo PDF evadiendo el WAF en cascada:
    1. Browser context request (hereda cookies, TLS y contexto de Camoufox/Playwright).
    2. curl_cffi con impersonación de TLS Chrome124 si está instalado.
    3. Navegación directa en pestaña con page.goto(url_pdf).
    4. HTTP Client tradicional (httpx).
    Retorna: (exito, bytes, codigo_http, mensaje_error)
    """
    last_status = 404
    last_err_msg = ""

    # Intento 1: Browser Context Request (Playwright / Camoufox)
    try:
        page_resp = await page.request.get(url_pdf)
        last_status = page_resp.status
        if page_resp.status == 200:
            b_content = await page_resp.body()
            if b_content.startswith(b"%PDF-"):
                return True, b_content, 200, ""
            else:
                last_err_msg = f"HTTP 200 pero contenido no es PDF ({len(b_content)} bytes)"
        else:
            last_err_msg = f"HTTP {page_resp.status}"
    except Exception as e:
        last_err_msg = f"Error en context request: {e}"

    # Intento 2: curl_cffi con suplantación de huella TLS
    try:
        from curl_cffi.requests import AsyncSession
        cookies = await context.cookies()
        cookie_dict = {c["name"]: c["value"] for c in cookies}
        async with AsyncSession(impersonate="chrome124", timeout=30.0) as session:
            resp = await session.get(
                url_pdf,
                cookies=cookie_dict,
                headers={
                    "Referer": "https://comunicacionsocial.senado.gob.mx/sintesis/sintesis.html",
                    "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36"
                }
            )
            last_status = resp.status_code
            if resp.status_code == 200 and resp.content.startswith(b"%PDF-"):
                return True, resp.content, 200, ""
            elif resp.status_code == 200:
                last_err_msg = f"HTTP 200 pero contenido no es PDF ({len(resp.content)} bytes)"
            else:
                last_err_msg = f"HTTP {resp.status_code}"
    except ImportError:
        pass
    except Exception as e:
        last_err_msg = f"Error en curl_cffi: {e}"

    # Intento 3: Navegación directa en pestaña del navegador
    try:
        nav_resp = await page.goto(url_pdf, wait_until="load", timeout=25000)
        if nav_resp:
            last_status = nav_resp.status
            if nav_resp.status == 200:
                b_content = await nav_resp.body()
                if b_content.startswith(b"%PDF-"):
                    return True, b_content, 200, ""
                else:
                    last_err_msg = "Respuesta de navegación no es PDF"
            else:
                last_err_msg = f"HTTP {nav_resp.status}"
    except Exception as e:
        last_err_msg = f"Error en navegación directa a PDF: {e}"

    # Intento 4: Fallback con cliente HTTP estándar (httpx)
    if http_client:
        try:
            resp = await http_client.get(url_pdf)
            last_status = resp.status_code
            if resp.status_code == 200 and resp.content.startswith(b"%PDF-"):
                return True, resp.content, 200, ""
            elif resp.status_code == 200:
                last_err_msg = "HTTP 200 pero contenido no es PDF"
            else:
                last_err_msg = f"HTTP {resp.status_code}"
        except Exception as e:
            last_err_msg = f"Error en httpx fallback: {e}"

    return False, b"", last_status, last_err_msg

async def check_pdf_availability(page, context, url_pdf: str, http_client: Optional[httpx.AsyncClient] = None) -> tuple[bool, Optional[int], int]:
    """
    Comprueba de forma rápida si un PDF existe y es accesible sin descargar el archivo completo.
    Retorna: (disponible, tamano_bytes, codigo_http)
    """
    # 1. Probar vía page.request (Browser context con cookies y bypass de Incapsula)
    try:
        resp = await page.request.get(url_pdf, headers={"Range": "bytes=0-500"})
        if resp.status in [200, 206]:
            body = await resp.body()
            if body.startswith(b"%PDF-") or "pdf" in resp.headers.get("content-type", "").lower():
                c_len = resp.headers.get("content-length") or resp.headers.get("content-range")
                tamano = None
                if c_len:
                    try:
                        tamano = int(str(c_len).split("/")[-1]) if "/" in str(c_len) else int(c_len)
                    except Exception:
                        pass
                return True, tamano or len(body), 200
        elif resp.status == 404:
            return False, None, 404
        elif resp.status == 403:
            return False, None, 403
    except Exception:
        pass

    # 2. Probar con curl_cffi si está disponible
    try:
        from curl_cffi.requests import AsyncSession
        cookies = await context.cookies()
        cookie_dict = {c["name"]: c["value"] for c in cookies}
        async with AsyncSession(impersonate="chrome124", timeout=15.0) as session:
            resp = await session.get(
                url_pdf,
                headers={"Range": "bytes=0-500", "Referer": "https://comunicacionsocial.senado.gob.mx/sintesis/sintesis.html"},
                cookies=cookie_dict
            )
            if resp.status_code in [200, 206]:
                if resp.content.startswith(b"%PDF-") or "pdf" in resp.headers.get("content-type", "").lower():
                    c_len = resp.headers.get("content-length") or resp.headers.get("content-range")
                    tamano = None
                    if c_len:
                        try:
                            tamano = int(str(c_len).split("/")[-1]) if "/" in str(c_len) else int(c_len)
                        except Exception:
                            pass
                    return True, tamano or len(resp.content), 200
            elif resp.status_code in [403, 404]:
                return False, None, resp.status_code
    except Exception:
        pass

    # 3. Fallback httpx
    if http_client:
        try:
            resp = await http_client.get(url_pdf, headers={"Range": "bytes=0-500"})
            if resp.status_code in [200, 206] and (resp.content.startswith(b"%PDF-") or "pdf" in resp.headers.get("content-type", "").lower()):
                c_len = resp.headers.get("content-length") or resp.headers.get("content-range")
                tamano = None
                if c_len:
                    try:
                        tamano = int(str(c_len).split("/")[-1]) if "/" in str(c_len) else int(c_len)
                    except Exception:
                        pass
                return True, tamano or len(resp.content), 200
            return False, None, resp.status_code
        except Exception:
            pass

    return False, None, 500

async def run_senado_scraper_pipeline(fecha_param: Optional[str] = None, secciones_param: Optional[List[str]] = None) -> Dict[str, Any]:
    """
    Ejecuta el ciclo completo de descarga del Senado:
    1. Resuelve la fecha objetivo (YYYY-MM-DD).
    2. Inicia el navegador anti-detección (Camoufox o fallback).
    3. Resuelve el reto WAF de Incapsula y extrae enlaces del DOM.
    4. Descarga las secciones seleccionadas de PDF (excluyendo Cartones).
    5. Guarda los archivos en disco.
    6. Inserta en la base de datos PostgreSQL con origen='senado' e incluido_en_sintesis=TRUE.
    7. Dispara el procesamiento OCR y la síntesis ejecutiva.
    """
    global scraper_status
    if scraper_status["en_progreso"]:
        return {"status": "en_progreso", "mensaje": "Un trabajo de scraping ya está en ejecución"}

    if fecha_param:
        try:
            fecha_dt = datetime.strptime(fecha_param, "%Y-%m-%d")
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
        async with acquire_browser_session() as (browser, context, browser_type):
            scraper_status["navegador_usado"] = browser_type

            page = await context.new_page()

            # 1. Navegar y validar reto WAF
            scraper_status["mensaje"] = "Accediendo al portal del Senado y validando seguridad WAF..."
            dom_links = await resolve_senado_session(page)

            # Preparar headers y cliente HTTP de apoyo
            cookies = await context.cookies()
            cookie_header = "; ".join([f"{c['name']}={c['value']}" for c in cookies])
            headers = {
                "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36",
                "Referer": "https://comunicacionsocial.senado.gob.mx/sintesis/sintesis.html",
                "Cookie": cookie_header
            }

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
                        success, b_data, st_code, err_txt = await fetch_pdf_content(page, context, url_pdf, http_client)
                        last_status = st_code
                        last_err_msg = err_txt

                        if success and len(b_data) >= 100:
                            file_bytes = b_data
                            download_success = True
                            successful_url = url_pdf
                            print(f"[Scraper Senado] ✅ Descarga exitosa de {url_pdf}: {len(file_bytes)} bytes")
                            break

                    if not download_success or len(file_bytes) < 100:
                        if last_status == 404:
                            motivo = f"No publicado por el Senado para la fecha {fecha_iso} (HTTP 404)."
                        elif last_status == 403:
                            motivo = f"Acceso restringido temporalmente por WAF / Incapsula (HTTP 403)."
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

                        cur.execute(
                            "SELECT id FROM boletines WHERE fecha_boletin = %s AND nombre_archivo = %s",
                            (fecha_iso, safe_filename)
                        )
                        existente = cur.fetchone()

                        if existente:
                            boletin_id = str(existente["id"])
                            print(f"[Scraper Senado] Boletín ya registrado previamente (ID: {boletin_id}). Actualizando...")
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
    evadiendo Incapsula sin procesar OCR ni guardar archivos pesados en disco.
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
        async with acquire_browser_session() as (browser, context, _):
            page = await context.new_page()

            # Resolver sesión y WAF
            dom_links = await resolve_senado_session(page)

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
                        is_avail, sz, st_code = await check_pdf_availability(page, context, curl, http_client)
                        last_status = st_code
                        if is_avail:
                            disponible = True
                            tamano = sz
                            break

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
