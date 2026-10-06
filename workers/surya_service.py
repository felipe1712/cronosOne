import os
import io
import tempfile
from typing import List, Dict, Any, Optional
from fastapi import FastAPI, File, UploadFile, HTTPException, Form
from pydantic import BaseModel
import uvicorn

app = FastAPI(
    title="ExposureIQ Surya OCR Service",
    description="Servicio local de OCR con Surya para periódicos, documentos y portadas",
    version="1.0.0"
)

# Cache de modelos en memoria para reutilización
_det_model = None
_det_processor = None
_rec_model = None
_rec_processor = None

def get_surya_models():
    """Carga los modelos de detección y reconocimiento de Surya de forma lazy."""
    global _det_model, _det_processor, _rec_model, _rec_processor
    if _det_model is None or _rec_model is None:
        print("[Surya Service] Cargando modelos de Surya OCR en memoria...")
        try:
            from surya.model.detection.model import load_model as load_det_model, load_processor as load_det_processor
            from surya.model.recognition.model import load_model as load_rec_model, load_processor as load_rec_processor

            _det_model = load_det_model()
            _det_processor = load_det_processor()
            _rec_model = load_rec_model()
            _rec_processor = load_rec_processor()
            print("[Surya Service] ✅ Modelos Surya OCR cargados exitosamente.")
        except Exception as e:
            print(f"[Surya Service] ❌ Error cargando modelos Surya OCR: {e}")
            raise e
    return _det_model, _det_processor, _rec_model, _rec_processor

def pdf_to_pil_images(pdf_path: str, max_dpi: int = 150) -> list:
    """Convierte las páginas del PDF a imágenes PIL de alta resolución."""
    from PIL import Image
    images = []

    # 1. Intentar con pypdfium2 (renderizado directo de alta fidelidad)
    try:
        import pypdfium2 as pdfium
        pdf = pdfium.PdfDocument(pdf_path)
        scale = max_dpi / 72.0
        for page in pdf:
            image = page.render(scale=scale).to_pil()
            images.append(image)
        if images:
            return images
    except Exception as e:
        print(f"[Surya Service] Aviso en pypdfium2 ({e}), probando fallback de imágenes...")

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
            return images
    except Exception as e:
        print(f"[Surya Service] Aviso en extracción pypdf: {e}")

    # 3. Intentar con pdf2image si está disponible
    try:
        from pdf2image import convert_from_path
        images = convert_from_path(pdf_path, dpi=max_dpi)
        if images:
            return images
    except Exception as e:
        print(f"[Surya Service] pdf2image no disponible: {e}")

    return images

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
        raise ValueError(f"No se pudieron extraer imágenes legibles del archivo: {file_path}")

    det_model, det_processor, rec_model, rec_processor = get_surya_models()
    from surya.ocr import run_ocr

    langs = ["es"]
    pages_result = []

    # Procesar página por página para control estricto de memoria RAM en servidores
    for idx, img in enumerate(images):
        page_num = idx + 1
        print(f"[Surya Service] Procesando OCR en página {page_num} de {len(images)}...")

        predictions = run_ocr(
            [img],
            [langs],
            det_model,
            det_processor,
            rec_model,
            rec_processor
        )

        lines = []
        if predictions:
            pred = predictions[0]
            for text_line in getattr(pred, "text_lines", []):
                t = getattr(text_line, "text", "").strip()
                if t:
                    lines.append(t)

        page_text = "\n".join(lines).strip()
        pages_result.append({
            "page": page_num,
            "text": page_text,
            "lines": lines
        })

    return pages_result

class OcrPathRequest(BaseModel):
    file_path: str

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
    file: Optional[UploadFile] = File(None),
    file_path: Optional[str] = Form(None)
):
    """
    Endpoint principal de OCR:
    Acepta subida multipart de archivo ('file') o ruta absoluta local ('file_path').
    """
    target_path = None
    temp_file_to_clean = None

    if file_path and os.path.exists(file_path):
        target_path = file_path
    elif file is not None:
        suffix = os.path.splitext(file.filename or "document.pdf")[1]
        with tempfile.NamedTemporaryFile(delete=False, suffix=suffix) as tmp:
            contents = await file.read()
            tmp.write(contents)
            target_path = tmp.name
            temp_file_to_clean = tmp.name
    else:
        raise HTTPException(
            status_code=400,
            detail="Debe proporcionar un archivo en el campo 'file' o una ruta en 'file_path'"
        )

    try:
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
