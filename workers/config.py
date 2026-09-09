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
    claude_model: str = os.getenv("CLAUDE_MODEL", "claude-3-5-sonnet-20241022")
    surya_ocr_url: str = os.getenv("SURYA_OCR_URL", "http://127.0.0.1:8000")
    director_whatsapp: str = os.getenv("DIRECTOR_WHATSAPP_PHONE", "5215512345678")
    world_intel_mcp_url: str = os.getenv("WORLD_INTEL_MCP_URL", "http://127.0.0.1:8095")

settings = Settings()
