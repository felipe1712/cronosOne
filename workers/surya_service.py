import os
import io
import json
import shutil
import subprocess
import tempfile
from typing import List, Dict, Any, Optional
from fastapi import FastAPI, File, UploadFile, HTTPException, Form, Request
from pydantic import BaseModel
import uvicorn

app = FastAPI(
    title="ExposureIQ Surya OCR Service",
    description="Servicio local de OCR con Surya para periódicos, documentos y portadas",
    version="1.1.0"
)

# Cache de modelos en memoria para reutilización
_det_model = None
_det_processor = None
_rec_model = None
_rec_processor = None

def get_surya_models():
    """Carga los modelos de detección y reconocimiento de Surya de forma lazy con soporte multi-versión."""
    global _det_model, _det_processor, _rec_model, _rec_processor
    if _det_model is None or _rec_model is None:
        print("[Surya Service] Cargando modelos de Surya OCR en memoria...")

        # 1. Detección (Model & Processor)
        det_proc = None
        det_mod = None

        try:
            from surya.model.detection.processor import load_processor as load_det_proc
            det_proc = load_det_proc()
        except Exception:
            try:
                from surya.model.detection.model import load_processor as load_det_proc
                det_proc = load_det_proc()
            except Exception:
                pass

        try:
            from surya.model.detection.model import load_model as load_det_mod
            det_mod = load_det_mod()
        except Exception:
            try:
                from surya.model.detection import load_model as load_det_mod
                det_mod = load_det_mod()
            except Exception:
                from surya.detection import load_model as load_det_mod
                det_mod = load_det_mod()

        if isinstance(det_mod, tuple):
            det_proc, det_mod = det_mod[0], det_mod[1]

        # 2. Reconocimiento (Model & Processor)
        rec_proc = None
        rec_mod = None

        try:
            from surya.model.recognition.processor import load_processor as load_rec_proc
            rec_proc = load_rec_proc()
        except Exception:
            try:
                from surya.model.recognition.model import load_processor as load_rec_proc
                rec_proc = load_rec_proc()
            except Exception:
                pass

        try:
            from surya.model.recognition.model import load_model as load_rec_mod
            rec_mod = load_rec_mod()
        except Exception:
            try:
                from surya.model.recognition import load_model as load_rec_mod
                rec_mod = load_rec_mod()
            except Exception:
                from surya.recognition import load_model as load_rec_mod
                rec_mod = load_rec_mod()

        if isinstance(rec_mod, tuple):
            rec_proc, rec_mod = rec_mod[0], rec_mod[1]

        _det_processor = det_proc
        _det_model = det_mod
        _rec_processor = rec_proc
        _rec_model = rec_mod
        print("[Surya Service] ✅ Modelos Surya OCR cargados exitosamente en memoria.")

    return _det_model, _det_processor, _rec_model, _rec_processor

def pdf_to_pil_images(pdf_path: str, max_dpi: int = 150) -> list:
    """Convierte las páginas del PDF a imágenes PIL de alta resolución."""
    from PIL import Image
    images = []

    # 1. Intentar con pypdfium2 (renderizado directo de alta fidelidad para periódicos)
    try:
        import pypdfium2 as pdfium
        pdf = pdfium.PdfDocument(pdf_path)
        scale = max_dpi / 72.0
        for page in pdf:
            image = page.render(scale=scale).to_pil()
            images.append(image)
        if images:
            print(f"[Surya Service] pypdfium2 renderizó {len(images)} páginas a ~{max_dpi} DPI.")
            return images
    except Exception as e:
        print(f"[Surya Service] Aviso en pypdfium2 ({e}), probando extracción de imágenes...")

    # 2. Intentar extraer imágenes rasterizadas de pypdf
    try:
        from pypdf import PdfReader
        reader = PdfReader(pdf_path)
        for page in reader.pages:
            if hasattr(page, "images") and page.images:
                largest = max(page.images, key=lambda img: len(img.data))
                img = Image.open(io.BytesIO(largest.data)).convert("RGB")
                images.append(img)
        if images:
            print(f"[Surya Service] pypdf extrajo {len(images)} imágenes rasterizadas.")
            return images
    except Exception as e:
        print(f"[Surya Service] Aviso en extracción pypdf: {e}")

    # 3. Intentar con pdf2image si está instalado poppler
    try:
        from pdf2image import convert_from_path
        images = convert_from_path(pdf_path, dpi=max_dpi)
        if images:
            print(f"[Surya Service] pdf2image convirtió {len(images)} páginas.")
            return images
    except Exception as e:
        print(f"[Surya Service] pdf2image no disponible: {e}")

    return images

def extract_lines_from_prediction(pred) -> List[str]:
    """Extrae líneas de texto de forma segura sin importar si pred es objeto, dict o lista."""
    lines = []
    if pred is None:
        return lines

    raw_lines = []
    if isinstance(pred, dict):
        raw_lines = pred.get("text_lines") or pred.get("lines") or []
    elif hasattr(pred, "text_lines"):
        raw_lines = getattr(pred, "text_lines")
    elif hasattr(pred, "lines"):
        raw_lines = getattr(pred, "lines")
    elif isinstance(pred, list):
        raw_lines = pred

    for item in raw_lines:
        line_text = ""
        if isinstance(item, dict):
            line_text = item.get("text", "")
        elif hasattr(item, "text"):
            line_text = getattr(item, "text", "")
        elif isinstance(item, str):
            line_text = item
        else:
            line_text = str(item)

        cleaned = str(line_text).strip()
        if cleaned:
            lines.append(cleaned)

    return lines

def run_surya_cli(file_path: str) -> List[Dict[str, Any]]:
    """Ejecuta surya_ocr vía comando CLI del entorno virtual como fallback robusto."""
    surya_bin = shutil.which("surya_ocr")
    if not surya_bin:
        candidates = [
            os.path.join(os.path.dirname(__file__), "venv", "bin", "surya_ocr"),
            "/opt/cronosOne/workers/venv/bin/surya_ocr",
            "/usr/local/bin/surya_ocr",
        ]
        for c in candidates:
            if os.path.exists(c):
                surya_bin = c
                break

    if not surya_bin:
        return []

    try:
        with tempfile.TemporaryDirectory() as out_dir:
            cmd = [surya_bin, file_path, "--langs", "es", "--results_dir", out_dir]
            print(f"[Surya CLI] Ejecutando: {' '.join(cmd)}")
            res = subprocess.run(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, timeout=600)

            results_file = None
            for root, _, files in os.walk(out_dir):
                if "results.json" in files:
                    results_file = os.path.join(root, "results.json")
                    break

            if not results_file or not os.path.exists(results_file):
                print(f"[Surya CLI] Código {res.returncode}. No se halló results.json. Stderr: {res.stderr[:200]}")
                return []

            with open(results_file, "r", encoding="utf-8", errors="ignore") as f:
                data = json.load(f)

            pages_entries = []
            if isinstance(data, dict):
                for _, val in data.items():
                    if isinstance(val, list):
                        pages_entries = val
                        break
            elif isinstance(data, list):
                pages_entries = data

            pages_result = []
            for idx, p_entry in enumerate(pages_entries):
                lines = extract_lines_from_prediction(p_entry)
                pages_result.append({
                    "page": idx + 1,
                    "text": "\n".join(lines).strip(),
                    "lines": lines
                })

            if any(len(p["text"]) > 40 for p in pages_result):
                print(f"[Surya CLI] ✅ Extraídas {len(pages_result)} páginas con éxito.")
                return pages_result
    except Exception as e:
        print(f"[Surya CLI] Excepción en ejecución: {e}")

    return []

def run_tesseract_on_images(images: list) -> List[Dict[str, Any]]:
    """Fallback local utilizando tesseract nativo sobre las páginas renderizadas."""
    tesseract = shutil.which("tesseract")
    if not tesseract:
        return []

    try:
        pages = []
        with tempfile.TemporaryDirectory() as tmpdir:
            for idx, img in enumerate(images):
                page_num = idx + 1
                img_path = os.path.join(tmpdir, f"page_{idx}.png")
                img.save(img_path, format="PNG")
                out_base = os.path.join(tmpdir, f"ocr_{idx}")

                cmd = [tesseract, img_path, out_base, "-l", "spa+eng"]
                res = subprocess.run(cmd, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
                if res.returncode != 0:
                    subprocess.run([tesseract, img_path, out_base], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

                txt_path = f"{out_base}.txt"
                text = ""
                if os.path.exists(txt_path):
                    with open(txt_path, "r", encoding="utf-8", errors="ignore") as f:
                        text = f.read().strip()

                lines = [l.strip() for l in text.splitlines() if l.strip()]
                pages.append({
                    "page": page_num,
                    "text": text,
                    "lines": lines
                })
        if pages and any(len(p["text"]) > 50 for p in pages):
            print(f"[Surya Service] ✅ Fallback local Tesseract completó {len(pages)} páginas.")
            return pages
    except Exception as e:
        print(f"[Surya Service] Error en fallback Tesseract: {e}")

    return []

def run_surya_ocr_pipeline(file_path: str) -> List[Dict[str, Any]]:
    """Ejecuta el OCR de Surya sobre el PDF o imagen local."""
    from PIL import Image

    images = []
    lower_path = file_path.lower()

    if lower_path.endswith(".pdf"):
        images = pdf_to_pil_images(file_path)
    elif lower_path.endswith((".png", ".jpg", ".jpeg", ".webp", ".bmp", ".tiff")):
        img = Image.open(file_path).convert("RGB")
        images = [img]

    if not images:
        cli_result = run_surya_cli(file_path)
        if cli_result:
            return cli_result
        raise ValueError(f"No se pudieron extraer imágenes legibles del archivo: {file_path}")

    # Intentar ejecutar con los modelos Surya cargados en memoria
    try:
        det_model, det_processor, rec_model, rec_processor = get_surya_models()
        from surya.ocr import run_ocr

        langs = ["es"]
        pages_result = []

        # Procesar página por página para control estricto de memoria RAM en servidores
        for idx, img in enumerate(images):
            page_num = idx + 1
            print(f"[Surya Service] Procesando OCR en página {page_num} de {len(images)}...")

            try:
                predictions = run_ocr(
                    [img],
                    [langs],
                    det_model,
                    det_processor,
                    rec_model,
                    rec_processor
                )
            except TypeError:
                predictions = run_ocr(
                    images=[img],
                    langs=[langs],
                    det_model=det_model,
                    det_processor=det_processor,
                    rec_model=rec_model,
                    rec_processor=rec_processor
                )

            lines = []
            if predictions:
                lines = extract_lines_from_prediction(predictions[0])

            page_text = "\n".join(lines).strip()
            print(f"[Surya Service] Página {page_num}: {len(lines)} líneas detectadas ({len(page_text)} caracteres).")

            pages_result.append({
                "page": page_num,
                "text": page_text,
                "lines": lines
            })

        if any(len(p["text"]) > 40 for p in pages_result):
            return pages_result

    except Exception as py_err:
        print(f"[Surya Service] Aviso: Falló ejecución Python en memoria ({py_err}). Probando fallback CLI...")

    # Fallback 1: Ejecutar vía CLI de Surya
    cli_pages = run_surya_cli(file_path)
    if cli_pages and any(len(p["text"]) > 40 for p in cli_pages):
        return cli_pages

    # Fallback 2: Tesseract nativo sobre las imágenes renderizadas
    tess_pages = run_tesseract_on_images(images)
    if tess_pages and any(len(p["text"]) > 40 for p in tess_pages):
        return tess_pages

    raise RuntimeError(f"No se pudo completar el OCR para {file_path} con ningún motor local.")

@app.get("/")
def root():
    return {
        "status": "online",
        "service": "ExposureIQ Surya OCR Engine",
        "port": 5000,
        "endpoint": "/ocr"
    }

@app.get("/health")
def health():
    try:
        import torch
        device = "cuda" if torch.cuda.is_available() else "cpu"
    except Exception:
        device = "unknown"
    return {
        "status": "ok",
        "device": device,
        "service": "Surya OCR"
    }

@app.post("/ocr")
async def ocr_endpoint(
    request: Request,
    file: Optional[UploadFile] = File(None),
    file_path: Optional[str] = Form(None)
):
    """
    Endpoint principal de OCR local:
    Acepta ruta local en JSON {'file_path': '...'}, formulario Form(file_path), o subida de archivo File.
    """
    target_path = None
    temp_file_to_clean = None

    # 1. Form field
    if file_path and os.path.exists(file_path):
        target_path = file_path

    # 2. JSON body
    if not target_path:
        try:
            body = await request.json()
            if isinstance(body, dict) and "file_path" in body:
                candidate = body["file_path"]
                if candidate and os.path.exists(candidate):
                    target_path = candidate
        except Exception:
            pass

    # 3. Multipart file upload
    if not target_path and file is not None:
        suffix = os.path.splitext(file.filename or "document.pdf")[1]
        with tempfile.NamedTemporaryFile(delete=False, suffix=suffix) as tmp:
            contents = await file.read()
            tmp.write(contents)
            target_path = tmp.name
            temp_file_to_clean = tmp.name

    if not target_path or not os.path.exists(target_path):
        raise HTTPException(
            status_code=400,
            detail="Debe proporcionar un archivo en 'file' o una ruta válida existente en 'file_path'."
        )

    try:
        print(f"[Surya Service] Recibida solicitud OCR para: {target_path}")
        pages = run_surya_ocr_pipeline(target_path)
        return {
            "status": "success",
            "model": "surya-ocr",
            "pages_count": len(pages),
            "pages": pages
        }
    except Exception as e:
        import traceback
        trace = traceback.format_exc()
        print(f"[Surya Service] Error procesando OCR: {e}\n{trace}")
        raise HTTPException(
            status_code=500,
            detail=f"Error ejecutando Surya OCR: {str(e)}"
        )
    finally:
        if temp_file_to_clean and os.path.exists(temp_file_to_clean):
            try:
                os.remove(temp_file_to_clean)
            except Exception:
                pass

if __name__ == "__main__":
    uvicorn.run("surya_service:app", host="127.0.0.1", port=5000, reload=False)

