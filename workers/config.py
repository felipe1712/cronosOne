import os
from pathlib import Path
from pydantic_settings import BaseSettings
from dotenv import load_dotenv

env_path = Path(__file__).resolve().parent / ".env"
load_dotenv(dotenv_path=env_path)

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
    
    # Configuración de Proxy Residencial (ej. Geonode, BrightData, IPRoyal)
    scraper_proxy_url: str = os.getenv("SCRAPER_PROXY_URL", "")
    scraper_proxy_host: str = os.getenv("SCRAPER_PROXY_HOST", "")
    scraper_proxy_port: str = os.getenv("SCRAPER_PROXY_PORT", "")
    scraper_proxy_username: str = os.getenv("SCRAPER_PROXY_USERNAME", "")
    scraper_proxy_password: str = os.getenv("SCRAPER_PROXY_PASSWORD", "")
    
    auto_ingest_dir: str = os.getenv("AUTO_INGEST_DIR", "")

settings = Settings()
