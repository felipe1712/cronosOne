import os
import httpx
import shutil
import subprocess
import tempfile
import asyncio
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

def try_local_tesseract_ocr(file_path: str) -> List[Dict[str, Any]]:
    """
    Fallback 100% local usando tesseract y pdftoppm del sistema operativo
    si el servicio de Surya no estuviera respondiendo.
    """
    pdftoppm = shutil.which("pdftoppm")
    tesseract = shutil.which("tesseract")
    if not pdftoppm or not tesseract:
        return []

    try:
        print("[Tesseract OCR Local] Herramientas del sistema detectadas. Ejecutando OCR local...")
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

        print(f"[Tesseract OCR Local] Procesadas {len(pages)} páginas exitosamente.")
        return pages
    except Exception as e:
        print(f"[Tesseract OCR Local] Error en ejecución ({e}).")
        return []

def try_inprocess_surya_ocr(file_path: str) -> List[Dict[str, Any]]:
    """
    Ejecuta Surya OCR directamente dentro del proceso del worker si las bibliotecas
    están instaladas localmente en el entorno virtual.
    """
    try:
        print("[Surya In-Process] Intentando ejecución directa de Surya OCR en memoria...")
        from surya_service import run_surya_ocr_pipeline
        pages = run_surya_ocr_pipeline(file_path)
        if pages and any(len(p.get("text", "")) > 30 for p in pages):
            print(f"[Surya In-Process] ✅ {len(pages)} páginas procesadas exitosamente.")
            return pages
    except ImportError as e:
        print(f"[Surya In-Process] Módulo surya-ocr no disponible en el venv: {e}")
    except Exception as e:
        print(f"[Surya In-Process] Error ejecutando pipeline local: {e}")
    return []

async def process_pdf_ocr(file_path: str) -> List[Dict[str, Any]]:
    """
    Envía el PDF al servicio interno local Surya OCR (puerto 5000).
    Procesamiento 100% privado y local en el servidor, sin terceros.
    """
    resolved_path = resolve_file_path(file_path)
    if not os.path.exists(resolved_path):
        raise FileNotFoundError(f"No se encontró el archivo: {file_path} (probado en {resolved_path})")

    file_path = resolved_path

    # 1. Intentar llamar al microservicio local Surya OCR en localhost:5000
    candidate_urls = [settings.surya_ocr_url]
    if "5000" not in settings.surya_ocr_url:
        candidate_urls.append("http://127.0.0.1:5000/ocr")
    elif "/ocr" not in settings.surya_ocr_url:
        candidate_urls.append(f"{settings.surya_ocr_url.rstrip('/')}/ocr")

    for ocr_url in candidate_urls:
        try:
            async with httpx.AsyncClient(timeout=300.0) as client:
                # Intento 1.A: Pasar ruta absoluta local en JSON (sin transferencia de bytes por socket)
                try:
                    resp_path = await client.post(ocr_url, json={"file_path": file_path})
                    if resp_path.status_code == 200:
                        data = resp_path.json()
                        pages = data.get("pages") if isinstance(data, dict) else data
                        if isinstance(pages, list) and pages and any(len(p.get("text", "")) > 30 for p in pages):
                            print(f"[Surya OCR] ✅ Procesamiento exitoso vía ruta JSON en {ocr_url} ({len(pages)} páginas)")
                            return pages
                except Exception as ex_json:
                    print(f"[Surya OCR] Aviso en intento ruta JSON: {ex_json}")

                # Intento 1.B: Subida multipart del archivo
                with open(file_path, "rb") as f:
                    files = {"file": (os.path.basename(file_path), f, "application/pdf")}
                    print(f"[Surya OCR] Enviando archivo multipart al microservicio local en {ocr_url}...")
                    response = await client.post(ocr_url, files=files)
                    if response.status_code == 200:
                        data = response.json()
                        pages = data.get("pages") if isinstance(data, dict) else data
                        if isinstance(pages, list) and pages and any(len(p.get("text", "")) > 30 for p in pages):
                            print(f"[Surya OCR] ✅ Procesamiento exitoso multipart en {ocr_url} ({len(pages)} páginas)")
                            return pages
                    else:
                        print(f"[Surya OCR] {ocr_url} devolvió HTTP {response.status_code}: {response.text[:200]}")
        except Exception as e:
            print(f"[Surya OCR] Microservicio no disponible en {ocr_url}: {e}")

    # 2. Intentar ejecución directa in-process de Surya si el microservicio HTTP no está activo
    inproc_pages = try_inprocess_surya_ocr(file_path)
    if inproc_pages:
        return inproc_pages

    # 3. Fallback con Tesseract local si está instalado en el servidor Linux
    tess_pages = try_local_tesseract_ocr(file_path)
    if tess_pages and any(len(p.get("text", "")) > 50 for p in tess_pages):
        return tess_pages

    # 4. Fallback digital con pypdf (para documentos vectoriales con texto real incrustado)
    print(f"[Fallback OCR] Extrayendo texto digital con pypdf para: {os.path.basename(file_path)}")
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
        print(f"[Fallback OCR] ⚠️ ATENCIÓN: El PDF '{os.path.basename(file_path)}' contiene imágenes de portadas y el microservicio local de Surya OCR no está activo en el puerto 5000.")
        print(f"[Fallback OCR] ⚠️ Inicie el servicio 'exposureiq-surya' para que Surya OCR procese las portadas de forma 100% local.")

    return pages_result
