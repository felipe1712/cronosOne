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

def resolve_target_path(path_str: str) -> Optional[str]:
    """Resuelve la ruta del archivo tolerando nombres de archivo con o sin prefijo UUID."""
    if not path_str:
        return None
    if os.path.exists(path_str):
        return os.path.abspath(path_str)

    filename = os.path.basename(path_str)
    search_dirs = [
        "/opt/cronosOne/backend/uploads",
        "/opt/cronosOne/backend",
        "/opt/cronosOne/workers",
        os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "backend", "uploads")),
    ]

    clean_name = filename.replace(" ", "_").lower().replace(".pdf", "")
    for sdir in search_dirs:
        if not os.path.exists(sdir):
            continue
        exact = os.path.join(sdir, filename)
        if os.path.exists(exact):
            return exact
        try:
            for f in os.listdir(sdir):
                f_lower = f.lower()
                if clean_name in f_lower or f_lower.endswith(clean_name + ".pdf"):
                    return os.path.join(sdir, f)
                parts = [p for p in clean_name.split("_") if len(p) > 3]
                if len(parts) >= 2 and all(p in f_lower for p in parts[:3]):
                    return os.path.join(sdir, f)
        except Exception:
            pass

    return None

def get_surya_models():
    """Carga los modelos de detección y reconocimiento de Surya de forma lazy con soporte multi-versión."""
    global _det_model, _det_processor, _rec_model, _rec_processor
    if _det_model is None or _rec_model is None:
        print("[Surya Service] Verificando modelos de Surya OCR en memoria...")

        det_proc = None
        det_mod = None

        try:
            from surya.model.detection.processor import load_processor as load_det_proc
            det_proc = load_det_proc()
        except Exception:
            pass

        try:
            from surya.model.detection.model import load_model as load_det_mod
            det_mod = load_det_mod()
        except Exception:
            pass

        if det_mod is None:
            try:
                from surya.model.detection.segformer import load_model as load_det_mod, load_processor as load_det_proc
                det_mod = load_det_mod()
                det_proc = load_det_proc()
            except Exception:
                pass

        if isinstance(det_mod, tuple):
            det_proc, det_mod = det_mod[0], det_mod[1]

        rec_proc = None
        rec_mod = None

        try:
            from surya.model.recognition.processor import load_processor as load_rec_proc
            rec_proc = load_rec_proc()
        except Exception:
            pass

        try:
            from surya.model.recognition.model import load_model as load_rec_mod
            rec_mod = load_rec_mod()
        except Exception:
            pass

        if isinstance(rec_mod, tuple):
            rec_proc, rec_mod = rec_mod[0], rec_mod[1]

        if det_mod is None or rec_mod is None:
            print("[Surya Service] Modelos en memoria no inicializados directamente. Se utilizará el motor oficial CLI surya_ocr.")
            return None, None, None, None

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

        with tempfile.TemporaryDirectory() as out_dir:
            # En versiones actuales de surya_ocr la opción oficial es --output_dir
            cmd = [surya_bin, file_path, "--output_dir", out_dir]
            print(f"[Surya CLI] Ejecutando: {' '.join(cmd)}")
            res = subprocess.run(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, timeout=600)

            # Si falla porque una versión requería --results_dir, reintentar con --results_dir
            if res.returncode != 0 and "--output_dir" in res.stderr:
                cmd = [surya_bin, file_path, "--results_dir", out_dir]
                print(f"[Surya CLI] Reintentando con --results_dir: {' '.join(cmd)}")
                res = subprocess.run(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, timeout=600)

            results_file = None
            for root, _, files in os.walk(out_dir):
                if "results.json" in files:
                    results_file = os.path.join(root, "results.json")
                    break

            if not results_file or not os.path.exists(results_file):
                print(f"[Surya CLI] Código {res.returncode}. No se halló results.json. Stderr: {res.stderr[:300]}")
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
    resolved = resolve_target_path(file_path)
    if resolved:
        file_path = resolved

    # Prioridad 1: Intentar con la CLI oficial de Surya (ejecución robusta y nativa)
    print(f"[Surya Pipeline] Intentando OCR con CLI oficial de Surya para: {file_path}")
    cli_pages = run_surya_cli(file_path)
    if cli_pages and any(len(p.get("text", "")) > 40 for p in cli_pages):
        print(f"[Surya Pipeline] ✅ CLI surya_ocr completó exitosamente ({len(cli_pages)} páginas).")
        return cli_pages

    # Prioridad 2: Renderizar páginas a imágenes PIL
    from PIL import Image
    images = []
    lower_path = file_path.lower()
    if lower_path.endswith(".pdf"):
        images = pdf_to_pil_images(file_path)
    elif lower_path.endswith((".png", ".jpg", ".jpeg", ".webp", ".bmp", ".tiff")):
        images = [Image.open(file_path).convert("RGB")]

    # Intentar con modelos en memoria si están disponibles
    det_model, det_processor, rec_model, rec_processor = get_surya_models()
    if det_model is not None and rec_model is not None and images:
        try:
            from surya.ocr import run_ocr
            langs = ["es"]
            pages_result = []
            for idx, img in enumerate(images):
                page_num = idx + 1
                try:
                    preds = run_ocr([img], [langs], det_model, det_processor, rec_model, rec_processor)
                except TypeError:
                    preds = run_ocr(images=[img], langs=[langs], det_model=det_model, det_processor=det_processor, rec_model=rec_model, rec_processor=rec_processor)
                lines = extract_lines_from_prediction(preds[0]) if preds else []
                pages_result.append({
                    "page": page_num,
                    "text": "\n".join(lines).strip(),
                    "lines": lines
                })
            if any(len(p.get("text", "")) > 40 for p in pages_result):
                return pages_result
        except Exception as py_err:
            print(f"[Surya Pipeline] Aviso: Falló OCR en memoria: {py_err}")

    # Prioridad 3: Fallback local con Tesseract nativo sobre las imágenes renderizadas
    if images:
        tess_pages = run_tesseract_on_images(images)
        if tess_pages and any(len(p.get("text", "")) > 40 for p in tess_pages):
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
    if file_path:
        target_path = resolve_target_path(file_path)

    # 2. JSON body
    if not target_path:
        try:
            body = await request.json()
            if isinstance(body, dict) and "file_path" in body:
                target_path = resolve_target_path(body["file_path"])
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

