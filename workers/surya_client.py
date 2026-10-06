import os
import httpx
from typing import Dict, Any, List
from config import settings
from pypdf import PdfReader

def resolve_file_path(file_path: str) -> str:
    """
    Resuelve la ruta del archivo asegurando compatibilidad entre servicios
    cuando el backend y los workers corren en diferentes directorios de trabajo.
    """
    if os.path.exists(file_path):
        return os.path.abspath(file_path)

    clean_path = file_path.lstrip("./").lstrip(".\\")
    filename = os.path.basename(file_path)

    candidates = [
        os.path.join("/opt/cronosOne/backend", clean_path),
        os.path.join("/opt/cronosOne/backend/uploads", filename),
        os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "backend", clean_path)),
        os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "backend", "uploads", filename)),
        os.path.abspath(os.path.join(os.path.dirname(__file__), "..", clean_path)),
        os.path.abspath(os.path.join(os.path.dirname(__file__), clean_path)),
        os.path.abspath(os.path.join(os.getcwd(), clean_path)),
    ]

    for candidate in candidates:
        if os.path.exists(candidate):
            print(f"[File Resolver] Archivo resuelto en: {candidate}")
            return candidate

    return file_path

import shutil
import subprocess
import tempfile
import base64
import asyncio

def get_largest_image_from_page(page) -> tuple[bytes, str] | None:
    """Extrae la imagen principal de la página del PDF (la portada del periódico)."""
    if not hasattr(page, "images"):
        return None
    try:
        images_list = list(page.images)
        if not images_list:
            return None
        largest_img = None
        max_bytes = 0
        for img in images_list:
            data = getattr(img, "data", None)
            if data and len(data) > max_bytes:
                max_bytes = len(data)
                largest_img = (data, getattr(img, "name", "page.jpg"))
        # Descartar logos pequeños (debe pesar más de 12 KB)
        if largest_img and max_bytes > 12000:
            ext = os.path.splitext(largest_img[1])[1].lower().lstrip(".")
            media_type = "image/jpeg" if ext in ["jpg", "jpeg"] else f"image/{ext}" if ext in ["png", "webp"] else "image/jpeg"
            return largest_img[0], media_type
    except Exception as e:
        print(f"[Image Parser] Error obteniendo imagen de portada: {e}")
    return None

async def transcribe_image_with_claude(image_bytes: bytes, media_type: str, page_num: int) -> str:
    """Utiliza Claude Vision API para transcribir portadas escaneadas con máxima precisión."""
    if not settings.anthropic_api_key:
        print("[Claude Vision OCR] ANTHROPIC_API_KEY no configurada. Omitiendo OCR visual.")
        return ""
    try:
        import anthropic
        client = anthropic.Anthropic(api_key=settings.anthropic_api_key)
        b64 = base64.b64encode(image_bytes).decode("utf-8")

        prompt = (
            f"Analiza con visión artificial la imagen de esta portada de periódico nacional (Página {page_num}).\n"
            "Realiza un OCR y extracción de contenido completo en español estructurado:\n"
            "1. Nombre del Periódico (ej. Reforma, El Universal, Milenio, El Financiero, La Jornada, etc.) y Fecha.\n"
            "2. Titular principal (nota de ocho / portada) con su desarrollo y subtítulo.\n"
            "3. Todos los titulares secundarios, balazos, notas destacadas y columnas de opinión (especialmente política, economía, seguridad y negocios).\n"
            "4. Cifras clave, nombres propios y declaraciones relevantes.\n\n"
            "Devuelve el texto organizado y limpio, sin saludos ni comentarios introductorios."
        )

        def _call_claude():
            model = settings.claude_model or "claude-3-5-sonnet-20241022"
            resp = client.messages.create(
                model=model,
                max_tokens=2048,
                messages=[{
                    "role": "user",
                    "content": [
                        {
                            "type": "image",
                            "source": {
                                "type": "base64",
                                "media_type": media_type,
                                "data": b64
                            }
                        },
                        {
                            "type": "text",
                            "text": prompt
                        }
                    ]
                }]
            )
            parts = [b.text for b in resp.content if hasattr(b, "text")]
            return "\n".join(parts).strip()

        return await asyncio.to_thread(_call_claude)
    except Exception as e:
        print(f"[Claude Vision OCR] Error en página {page_num}: {e}")
        return ""

def try_local_tesseract_ocr(file_path: str) -> List[Dict[str, Any]]:
    """
    Fallback usando herramientas nativas del sistema (tesseract + pdftoppm)
    para PDFs escaneados donde cada página es una imagen rasterizada.
    """
    pdftoppm = shutil.which("pdftoppm")
    tesseract = shutil.which("tesseract")
    if not pdftoppm or not tesseract:
        return []

    try:
        print("[Tesseract OCR] pdftoppm y tesseract detectados en el sistema. Ejecutando OCR local...")
        pages = []
        with tempfile.TemporaryDirectory() as tmpdir:
            cmd_ppm = [pdftoppm, "-png", "-r", "150", file_path, os.path.join(tmpdir, "page")]
            subprocess.run(cmd_ppm, check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

            img_files = sorted([f for f in os.listdir(tmpdir) if f.startswith("page") and f.endswith(".png")])
            for idx, img_name in enumerate(img_files):
                img_path = os.path.join(tmpdir, img_name)
                out_base = os.path.join(tmpdir, f"ocr_{idx}")
                cmd_tess = [tesseract, img_path, out_base, "-l", "spa+eng"]
                res = subprocess.run(cmd_tess, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
                if res.returncode != 0:
                    subprocess.run([tesseract, img_path, out_base], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

                txt_path = f"{out_base}.txt"
                text = ""
                if os.path.exists(txt_path):
                    with open(txt_path, "r", encoding="utf-8", errors="ignore") as f:
                        text = f.read().strip()

                pages.append({
                    "page": idx + 1,
                    "text": text,
                    "lines": [line.strip() for line in text.splitlines() if line.strip()]
                })

        print(f"[Tesseract OCR] Procesadas {len(pages)} páginas exitosamente con OCR local.")
        return pages
    except Exception as e:
        print(f"[Tesseract OCR] Falló ejecución local de tesseract ({e}). Continuando...")
        return []

async def process_pdf_ocr(file_path: str) -> List[Dict[str, Any]]:
    """
    Envía el PDF al servicio interno Surya OCR en el servidor.
    Si el servicio no responde, utiliza fallback con tesseract o Claude Vision AI.
    Retorna una lista de páginas con texto estructurado.
    """
    resolved_path = resolve_file_path(file_path)
    if not os.path.exists(resolved_path):
        raise FileNotFoundError(f"No se encontró el archivo: {file_path} (probado en {resolved_path})")

    file_path = resolved_path

    # 1. Intentar llamar al servicio Surya OCR (probar URL configurada y candidatos habituales)
    candidate_urls = [settings.surya_ocr_url]
    if "5000" not in settings.surya_ocr_url:
        candidate_urls.append("http://127.0.0.1:5000/ocr")
        candidate_urls.append("http://127.0.0.1:5000/api/ocr")
    elif "/ocr" not in settings.surya_ocr_url:
        candidate_urls.append(f"{settings.surya_ocr_url.rstrip('/')}/ocr")

    for ocr_url in candidate_urls:
        try:
            async with httpx.AsyncClient(timeout=180.0) as client:
                with open(file_path, "rb") as f:
                    files = {"file": (os.path.basename(file_path), f, "application/pdf")}
                    print(f"[Surya OCR] Consultando servicio OCR en {ocr_url}...")
                    response = await client.post(ocr_url, files=files)
                    if response.status_code == 200:
                        data = response.json()
                        if isinstance(data, list) and data:
                            print(f"[Surya OCR] ✅ Respuesta exitosa desde {ocr_url} ({len(data)} páginas)")
                            return data
                        elif isinstance(data, dict):
                            for key in ["pages", "results", "ocr", "data", "secciones"]:
                                if key in data and isinstance(data[key], list) and data[key]:
                                    print(f"[Surya OCR] ✅ Respuesta exitosa desde {ocr_url} con clave '{key}' ({len(data[key])} páginas)")
                                    return data[key]
                            if "text" in data and isinstance(data["text"], str) and data["text"].strip():
                                print(f"[Surya OCR] ✅ Respuesta de texto plano desde {ocr_url}")
                                return [{"page": 1, "text": data["text"], "lines": data["text"].splitlines()}]
                    else:
                        print(f"[Surya OCR] Endpoint {ocr_url} devolvió HTTP {response.status_code}: {response.text[:150]}")
        except Exception as e:
            print(f"[Surya OCR] No se pudo conectar con {ocr_url}: {e}")

    # 2. Fallback con Tesseract nativo si está disponible en el servidor (ideal para imágenes de periódicos)
    tess_pages = try_local_tesseract_ocr(file_path)
    if tess_pages:
        has_real_content = any(len(p.get("text", "")) > 50 for p in tess_pages)
        if has_real_content:
            return tess_pages

    # 3. Fallback de extracción de texto digital con pypdf
    pages_result = []
    reader = PdfReader(file_path)
    total_pages = len(reader.pages)

    for idx, page in enumerate(reader.pages):
        page_num = idx + 1
        text = page.extract_text() or ""
        pages_result.append({
            "page": page_num,
            "text": text.strip(),
            "lines": [line.strip() for line in text.splitlines() if line.strip()]
        })

    texts = [p["text"] for p in pages_result if p["text"]]
    unique_texts = set(t.strip() for t in texts)
    is_image_only = (
        len(pages_result) > 1 and len(unique_texts) <= 2 and all(len(t) < 300 for t in unique_texts)
    ) or (total_pages > 0 and not any(len(p.get("text", "")) > 150 for p in pages_result))

    # 4. Fallback de Visión Artificial (Claude Vision) para PDFs escaneados / portadas de periódicos
    if is_image_only and settings.anthropic_api_key:
        print(f"[Fallback OCR] ⚠️ AVISO: El PDF '{os.path.basename(file_path)}' contiene imágenes de portadas sin texto digital real.")
        print(f"[Claude Vision OCR] 🤖 Activando extracción visual con Claude Vision API para las {total_pages} páginas...")

        sem = asyncio.Semaphore(3)

        async def _process_single_page(idx: int, page_obj):
            img_info = get_largest_image_from_page(page_obj)
            if not img_info:
                return idx, ""
            raw_bytes, m_type = img_info
            async with sem:
                print(f"[Claude Vision OCR] Transcribiendo visualmente portada {idx + 1} de {total_pages}...")
                txt = await transcribe_image_with_claude(raw_bytes, m_type, idx + 1)
                return idx, txt

        tasks = [_process_single_page(i, page) for i, page in enumerate(reader.pages)]
        results = await asyncio.gather(*tasks)

        vision_count = 0
        for idx, vision_text in results:
            if vision_text and len(vision_text) > 80:
                pages_result[idx]["text"] = vision_text
                pages_result[idx]["lines"] = [l.strip() for l in vision_text.splitlines() if l.strip()]
                vision_count += 1

        if vision_count > 0:
            print(f"[Claude Vision OCR] ✅ {vision_count}/{total_pages} portadas transcritas con éxito usando visión artificial.")
            return pages_result
        else:
            print("[Claude Vision OCR] No se pudieron extraer imágenes de las páginas o falló la API.")

    print(f"[Fallback OCR] Extraídas {total_pages} páginas exitosamente con pypdf.")
    return pages_result
