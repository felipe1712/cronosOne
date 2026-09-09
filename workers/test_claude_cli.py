import os
import sys
from pathlib import Path
from dotenv import load_dotenv

# Cargar variables de entorno de workers
env_path = Path(__file__).resolve().parent / ".env"
load_dotenv(dotenv_path=env_path)

import anthropic

def run_test():
    api_key = os.getenv("ANTHROPIC_API_KEY", "")
    print(f"=== Diagnóstico de Modelos Anthropic Claude ===")
    print(f"Clave detectada: {api_key[:12]}... (longitud: {len(api_key)})" if api_key else "❌ NO SE ENCONTRÓ ANTHROPIC_API_KEY")
    
    if not api_key or "tu_api_key" in api_key:
        print("ERROR: Debes colocar tu API key real en /opt/cronosOne/workers/.env")
        return

    client = anthropic.Anthropic(api_key=api_key)
    
    models = [
        "claude-3-7-sonnet-20250219",
        "claude-3-5-sonnet-20241022",
        "claude-3-5-sonnet-20240620",
        "claude-3-5-sonnet-latest",
        "claude-3-5-haiku-20241022",
        "claude-3-haiku-20240307",
        "claude-3-sonnet-20240229",
    ]

    success_found = False
    for m in models:
        print(f"\nProbando: {m} ...", end=" ", flush=True)
        try:
            res = client.messages.create(
                model=m,
                max_tokens=15,
                messages=[{"role": "user", "content": "Di 'OK funcionando' en 2 palabras"}]
            )
            text = res.content[0].text.strip()
            print(f"✅ ÉXITO -> Respuesta: '{text}'")
            success_found = True
        except Exception as e:
            print(f"❌ FALLÓ: {e}")

    print("\n" + "=" * 50)
    if success_found:
        print("¡Al menos un modelo respondió correctamente!")
    else:
        print("Ningún modelo respondió. Revisa si tu API key tiene créditos en console.anthropic.com o si está restringida.")

if __name__ == "__main__":
    run_test()
