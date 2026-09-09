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

async def process_pdf_ocr(file_path: str) -> List[Dict[str, Any]]:
    """
    Envía el PDF al servicio interno Surya OCR en el servidor.
    Si el servicio no responde (entorno de pruebas local), utiliza fallback con pypdf.
    Retorna una lista de páginas con texto estructurado.
    """
    resolved_path = resolve_file_path(file_path)
    if not os.path.exists(resolved_path):
        raise FileNotFoundError(f"No se encontró el archivo: {file_path} (probado en {resolved_path})")
    
    file_path = resolved_path

    # 1. Intentar llamar al servicio interno Surya OCR
    try:
        async with httpx.AsyncClient(timeout=120.0) as client:
            with open(file_path, "rb") as f:
                files = {"file": (os.path.basename(file_path), f, "application/pdf")}
                response = await client.post(settings.surya_ocr_url, files=files)
                if response.status_code == 200:
                    data = response.json()
                    # Si Surya retorna lista de páginas estructuradas
                    if isinstance(data, list):
                        return data
                    elif "pages" in data:
                        return data["pages"]
    except Exception as e:
        print(f"[Surya OCR] Servicio Surya no disponible en {settings.surya_ocr_url} ({e}). Usando fallback local pypdf.")

    # 2. Fallback de extracción de texto con pypdf
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

    print(f"[Fallback OCR] Extraídas {total_pages} páginas exitosamente.")
    return pages_result
