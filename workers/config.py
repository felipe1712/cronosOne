import os
from pathlib import Path
from pydantic_settings import BaseSettings
from dotenv import load_dotenv

env_path = Path(__file__).resolve().parent / ".env"
if env_path.exists():
    load_dotenv(dotenv_path=env_path, override=True)

def _get_env_non_empty(*keys, default=""):
    for k in keys:
        v = os.getenv(k, "").strip()
        if v:
            return v
    return default

class Settings(BaseSettings):
    port: int = int(os.getenv("PORT", "8091"))
    host: str = os.getenv("HOST", "0.0.0.0")
    database_url: str = os.getenv(
        "DATABASE_URL", 
        "postgres://exposureiq_user:ExposureIQ2026_Secure!@127.0.0.1:5432/exposureiq_db"
    )
    anthropic_api_key: str = os.getenv("ANTHROPIC_API_KEY", "")
    claude_model: str = os.getenv("CLAUDE_MODEL", "claude-sonnet-4-5-20250929")
    surya_ocr_url: str = os.getenv("SURYA_OCR_URL", "http://127.0.0.1:5000/ocr")
    director_whatsapp: str = os.getenv("DIRECTOR_WHATSAPP_PHONE", "")
    world_intel_mcp_url: str = os.getenv("WORLD_INTEL_MCP_URL", "http://127.0.0.1:8095")
    senado_sintesis_url: str = os.getenv("SENADO_SINTESIS_URL", "https://comunicacionsocial.senado.gob.mx/sintesiss")
    lightpanda_cdp_url: str = os.getenv("LIGHTPANDA_CDP_URL", "")
    
    # Configuración de Proxy Residencial (Geonode, etc.)
    scraper_proxy_url: str = _get_env_non_empty(
        "SCRAPER_PROXY_URL", "GEONODE_PROXY_URL", "PROXY_URL", "HTTP_PROXY", "HTTPS_PROXY"
    )
    scraper_proxy_host: str = _get_env_non_empty("SCRAPER_PROXY_HOST", "GEONODE_HOST")
    scraper_proxy_port: str = _get_env_non_empty("SCRAPER_PROXY_PORT", "GEONODE_PORT", default="9000")
    scraper_proxy_username: str = _get_env_non_empty("SCRAPER_PROXY_USERNAME", "GEONODE_USER", "SCRAPER_PROXY_USER")
    scraper_proxy_password: str = _get_env_non_empty("SCRAPER_PROXY_PASSWORD", "GEONODE_PASSWORD", "SCRAPER_PROXY_PASS")
    
    auto_ingest_dir: str = os.getenv("AUTO_INGEST_DIR", "")

settings = Settings()
