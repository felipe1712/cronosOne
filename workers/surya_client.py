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
    Si el servicio no responde, utiliza fallback con tesseract o pypdf.
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
    if len(pages_result) > 1 and len(unique_texts) <= 2 and all(len(t) < 300 for t in unique_texts):
        print(f"[Fallback OCR] ⚠️ AVISO: El PDF '{os.path.basename(file_path)}' contiene imágenes de páginas sin texto vectorial embebido.")
        print(f"[Fallback OCR] ⚠️ El texto digital extraído es solo el encabezado repetitivo: {list(unique_texts)[:1]}")
        print(f"[Fallback OCR] ⚠️ Para procesar las portadas de periódicos, asegúrate de que el servicio Surya OCR esté en http://127.0.0.1:5000/ocr o instala tesseract-ocr en el sistema.")

    print(f"[Fallback OCR] Extraídas {total_pages} páginas exitosamente con pypdf.")
    return pages_result
