#!/usr/bin/env python3
"""
Script de Diagnóstico y Depuración de Surya OCR en Servidor
Ejecutar con:
  cd /opt/cronosOne/workers
  source venv/bin/activate
  python debug_ocr.py [ruta_al_pdf_opcional]
"""

import os
import sys
import time
import traceback
import glob

def print_step(title):
    print("\n" + "=" * 70)
    print(f"🔍 {title}")
    print("=" * 70)

def main():
    print("🚀 INICIANDO DIAGNÓSTICO DEL PIPELINE DE OCR LOCAL...")
    
    # -------------------------------------------------------------
    # PASO 1: Localizar PDF de prueba
    # -------------------------------------------------------------
    print_step("PASO 1: Localización del archivo PDF de prueba")
    pdf_path = None
    if len(sys.argv) > 1 and os.path.exists(sys.argv[1]):
        pdf_path = sys.argv[1]
    else:
        candidates = [
            "/opt/cronosOne/backend/uploads/*primeras*.pdf",
            "/opt/cronosOne/backend/uploads/*PRIMERAS*.pdf",
            "/opt/cronosOne/backend/uploads/*.pdf",
            "../backend/uploads/*.pdf",
            "./*.pdf",
        ]
        for pattern in candidates:
            matches = glob.glob(pattern)
            if matches:
                # Tomar el más reciente
                matches.sort(key=lambda x: os.path.getmtime(x), reverse=True)
                pdf_path = matches[0]
                break

    if not pdf_path or not os.path.exists(pdf_path):
        print("❌ No se encontró ningún archivo PDF en backend/uploads para probar.")
        print("   Por favor pasa la ruta como argumento: python debug_ocr.py /ruta/al/archivo.pdf")
        return

    print(f"📄 Archivo seleccionado: {pdf_path}")
    print(f"   Tamaño: {os.path.getsize(pdf_path) / (1024 * 1024):.2f} MB")

    # -------------------------------------------------------------
    # PASO 2: Verificar Entorno Python y Bibliotecas
    # -------------------------------------------------------------
    print_step("PASO 2: Verificación de paquetes y entorno")
    print(f"🐍 Python binario: {sys.executable}")
    print(f"📁 Directorio de trabajo: {os.getcwd()}")

    for pkg in ["torch", "torchvision", "pypdfium2", "PIL", "pypdf", "surya", "fastapi", "httpx"]:
        try:
            mod = __import__(pkg)
            ver = getattr(mod, "__version__", "instalado")
            print(f"   ✅ {pkg}: {ver}")
        except ImportError as e:
            print(f"   ❌ {pkg}: NO INSTALADO ({e})")

    try:
        import torch
        print(f"   ⚙️ Dispositivo PyTorch: {'CUDA (' + torch.cuda.get_device_name(0) + ')' if torch.cuda.is_available() else 'CPU'}")
    except Exception as e:
        print(f"   ⚠️ Error comprobando torch: {e}")

    # -------------------------------------------------------------
    # PASO 3: Renderizado de páginas con pypdfium2
    # -------------------------------------------------------------
    print_step("PASO 3: Renderizado de PDF a imágenes PIL")
    try:
        import pypdfium2 as pdfium
        pdf = pdfium.PdfDocument(pdf_path)
        page_count = len(pdf)
        print(f"   ✅ PDF abierto exitosamente con pypdfium2. Total de páginas: {page_count}")
        
        # Renderizar primera página a 150 DPI
        scale = 150 / 72.0
        start_t = time.time()
        page0 = pdf[0]
        pil_image = page0.render(scale=scale).to_pil()
        dur = time.time() - start_t
        print(f"   ✅ Página 1 renderizada en {dur:.2f}s.")
        print(f"   Dimensiones de la imagen: {pil_image.size} (Ancho x Alto) en modo {pil_image.mode}")
        test_img_path = "/tmp/test_ocr_page1.png"
        pil_image.save(test_img_path)
        print(f"   💾 Imagen guardada para verificación visual en: {test_img_path}")
    except Exception as e:
        print(f"   ❌ Error renderizando con pypdfium2: {e}")
        traceback.print_exc()
        pil_image = None

    # -------------------------------------------------------------
    # PASO 4: Carga de modelos de Surya OCR
    # -------------------------------------------------------------
    print_step("PASO 4: Carga de modelos de Surya OCR")
    det_model, det_processor, rec_model, rec_processor = None, None, None, None
    try:
        from surya_service import get_surya_models
        start_t = time.time()
        print("   ⏳ Cargando modelos Surya OCR en memoria...")
        det_model, det_processor, rec_model, rec_processor = get_surya_models()
        dur = time.time() - start_t
        print(f"   ✅ Modelos Surya cargados en memoria exitosamente en {dur:.2f}s.")
        print(f"   det_model: {type(det_model)}")
        print(f"   rec_model: {type(rec_model)}")
    except Exception as e:
        print(f"   ❌ ERROR al cargar modelos Surya: {e}")
        traceback.print_exc()

    # -------------------------------------------------------------
    # PASO 5: Ejecución directa de Surya OCR sobre la página 1
    # -------------------------------------------------------------
    print_step("PASO 5: Ejecución de OCR Surya sobre la página 1")
    if pil_image and det_model and rec_model:
        try:
            from surya.ocr import run_ocr
            from surya_service import extract_lines_from_prediction
            
            print("   ⏳ Ejecutando run_ocr sobre la imagen de la página 1...")
            start_t = time.time()
            try:
                preds = run_ocr(
                    [pil_image],
                    [["es"]],
                    det_model,
                    det_processor,
                    rec_model,
                    rec_processor
                )
            except TypeError:
                preds = run_ocr(
                    images=[pil_image],
                    langs=[["es"]],
                    det_model=det_model,
                    det_processor=det_processor,
                    rec_model=rec_model,
                    rec_processor=rec_processor
                )
            dur = time.time() - start_t
            print(f"   ✅ run_ocr completado en {dur:.2f}s!")
            
            lines = extract_lines_from_prediction(preds[0]) if preds else []
            print(f"   📝 Total de líneas de texto extraídas: {len(lines)}")
            print("   --- MUESTRA DEL TEXTO EXTRAÍDO (Primeras 15 líneas) ---")
            for l in lines[:15]:
                print(f"   | {l}")
            print("   --------------------------------------------------------")
            
            if len(lines) > 5:
                print("   🎉 SURYA OCR ESTÁ FUNCIONANDO PERFECTAMENTE EN MEMORIA!")
            else:
                print("   ⚠️ Surya no detectó suficientes líneas de texto en la imagen.")
        except Exception as e:
            print(f"   ❌ ERROR ejecutando run_ocr: {e}")
            traceback.print_exc()
    else:
        print("   ⏭️ Omitido porque falló el renderizado o la carga de modelos.")

    # -------------------------------------------------------------
    # PASO 6: Probar llamada HTTP al microservicio en localhost:5000
    # -------------------------------------------------------------
    print_step("PASO 6: Verificación del Microservicio HTTP (http://127.0.0.1:5000)")
    try:
        import httpx
        
        # Test /health
        print("   Consultando GET http://127.0.0.1:5000/health ...")
        resp = httpx.get("http://127.0.0.1:5000/health", timeout=5.0)
        print(f"   Status: {resp.status_code}")
        print(f"   Body: {resp.text}")

        # Test POST /ocr con JSON
        print(f"   Consultando POST http://127.0.0.1:5000/ocr con {pdf_path}...")
        start_t = time.time()
        resp_ocr = httpx.post(
            "http://127.0.0.1:5000/ocr",
            json={"file_path": os.path.abspath(pdf_path)},
            timeout=180.0
        )
        dur = time.time() - start_t
        print(f"   Status POST /ocr: {resp_ocr.status_code} (tardó {dur:.2f}s)")
        if resp_ocr.status_code == 200:
            data = resp_ocr.json()
            pages = data.get("pages", [])
            print(f"   ✅ Servicio HTTP respondió con {len(pages)} páginas procesadas.")
            if pages:
                print(f"   Muestra página 1 ({len(pages[0].get('lines', []))} líneas):")
                print(pages[0].get("text", "")[:300])
        else:
            print(f"   ❌ Fallo en endpoint HTTP: {resp_ocr.text[:500]}")
    except Exception as e:
        print(f"   ❌ Error conectando con http://127.0.0.1:5000: {e}")

    # -------------------------------------------------------------
    # PASO 7: Comprobar Tesseract nativo
    # -------------------------------------------------------------
    print_step("PASO 7: Comprobación de Tesseract en el sistema operativo")
    import shutil
    tess_path = shutil.which("tesseract")
    if tess_path:
        print(f"   ✅ Tesseract instalado en: {tess_path}")
    else:
        print("   ℹ️ Tesseract no está instalado en el sistema.")
        print("   (Para instalarlo como fallback de respaldo: apt-get install -y tesseract-ocr tesseract-ocr-spa poppler-utils)")

    print("\n" + "=" * 70)
    print("🏁 DIAGNÓSTICO FINALIZADO")
    print("=" * 70)

if __name__ == "__main__":
    main()
