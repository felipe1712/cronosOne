import os
from pathlib import Path
from dotenv import load_dotenv

env_path = Path(__file__).resolve().parent / ".env"
load_dotenv(dotenv_path=env_path)

import anthropic

api_key = os.getenv("ANTHROPIC_API_KEY", "")
client = anthropic.Anthropic(api_key=api_key)

print("Consultando lista oficial de modelos habilitados para tu API Key en Anthropic...")
try:
    models_page = client.models.list()
    print("Modelos encontrados en tu cuenta:")
    for m in models_page.data:
        print(f" - ID: {m.id} | Nombre: {getattr(m, 'display_name', m.id)}")
except Exception as e:
    print(f"Error listando modelos con client.models.list(): {e}")
    # Probar nombres modernos comunes (Claude 4, Claude 3.7 latest, etc.)
    for candidate in [
        "claude-3-7-sonnet-latest",
        "claude-4-sonnet",
        "claude-4-haiku",
        "claude-3-haiku-20240307",
        "claude-instant-1.2"
    ]:
        try:
            r = client.messages.create(model=candidate, max_tokens=10, messages=[{"role": "user", "content": "hi"}])
            print(f"✅ FUNCIONA: {candidate}")
        except Exception as err:
            print(f"❌ {candidate}: {err}")
