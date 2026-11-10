"""
ExposureIQ — Script de Diagnóstico y Verificación de Proxy (Geonode)
Ejecutar desde terminal:
  python workers/test_proxy.py
O dentro del contenedor:
  docker compose exec workers python test_proxy.py
"""

import os
import sys
from pathlib import Path
from dotenv import load_dotenv

# Cargar explícitamente workers/.env con override=True
env_file = Path(__file__).resolve().parent / ".env"
if env_file.exists():
    load_dotenv(dotenv_path=env_file, override=True)

import httpx
from config import settings
from senado_scraper import get_resolved_proxy_config

def test_proxy_connection():
    print("=================================================================")
    print("   ExposureIQ — Diagnóstico de Conexión de Proxy Residencial     ")
    print("=================================================================")
    print(f"📁 Archivo .env inspeccionado: {env_file} (Existe: {env_file.exists()})")

    proxy_cfg = get_resolved_proxy_config()
    if not proxy_cfg:
        print("\n❌ ERROR: No se detectó ninguna configuración de proxy.")
        print("Variables buscadas: SCRAPER_PROXY_URL, GEONODE_PROXY_URL, HTTP_PROXY o SCRAPER_PROXY_HOST")
        if env_file.exists():
            with open(env_file, "r", encoding="utf-8", errors="ignore") as f:
                lines = [l.strip() for l in f if "PROXY" in l.upper()]
                print(f"Líneas con 'PROXY' encontradas en {env_file.name}: {lines if lines else 'Ninguna'}")
        print("\nFormato correcto en workers/.env:")
        print("SCRAPER_PROXY_URL=http://tu_usuario:tu_password@92.204.171.83:9000")
        sys.exit(1)

    server = proxy_cfg["server"]
    user = proxy_cfg.get("username", "(sin usuario)")
    print(f"\n1. Configuración detectada:")
    print(f"   • Servidor: {server}")
    print(f"   • Usuario : {user}")

    # Construir URL formateada para httpx
    if "username" in proxy_cfg and "password" in proxy_cfg:
        user_pass = f"{proxy_cfg['username']}:{proxy_cfg['password']}@"
    else:
        user_pass = ""
    clean_server = server.replace("http://", "").replace("https://", "")
    actual_proxy_url = f"http://{user_pass}{clean_server}"

    print(f"\n2. Verificando IP pública y país de salida a través del proxy...")
    try:
        with httpx.Client(proxy=actual_proxy_url, timeout=20.0) as client:
            resp = client.get("https://ipinfo.io/json")
            if resp.status_code == 200:
                data = resp.json()
                ip = data.get("ip", "Desconocida")
                country = data.get("country", "??")
                city = data.get("city", "Desconocida")
                org = data.get("org", "Desconocido")
                
                print(f"   ✅ Conexión con Geonode exitosa!")
                print(f"   • IP de salida: {ip}")
                print(f"   • País        : {country} {'(🇲🇽 México - Correcto)' if country == 'MX' else '(⚠️ Ojo: No es MX, revisa el filtro -country-mx en el usuario)'}")
                print(f"   • Ciudad      : {city}")
                print(f"   • Proveedor/ISP: {org}")
            else:
                print(f"   ⚠️ Respuesta inesperada de ipinfo: HTTP {resp.status_code}")
    except Exception as e:
        print(f"   ❌ Fallo al conectar con el proxy: {e}")
        print("   Revisa que el host, puerto, usuario y contraseña de Geonode sean correctos.")
        sys.exit(1)

    print(f"\n3. Probando conectividad hacia el portal del Senado...")
    try:
        headers = {
            "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36"
        }
        with httpx.Client(proxy=actual_proxy_url, headers=headers, timeout=25.0, follow_redirects=True) as client:
            target = "https://comunicacionsocial.senado.gob.mx/sintesis/sintesis.html"
            resp = client.get(target)
            print(f"   • URL destino: {target}")
            print(f"   • Código HTTP: {resp.status_code}")
            
            if resp.status_code == 200:
                print(f"   ✅ El portal del Senado responde correctamente a través del proxy!")
            elif resp.status_code == 403:
                print(f"   ℹ️ El Senado devolvió HTTP 403 (reto WAF activo). Camoufox resolverá la telemetría en el navegador.")
            else:
                print(f"   • Estado: Código {resp.status_code}")
    except Exception as e:
        print(f"   ❌ Error conectando al Senado vía proxy: {e}")

    print("\n=================================================================")
    print("   Diagnóstico concluido.")
    print("=================================================================\n")

if __name__ == "__main__":
    test_proxy_connection()
