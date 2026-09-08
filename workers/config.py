import os
from pydantic_settings import BaseSettings
from dotenv import load_dotenv

load_dotenv()

class Settings(BaseSettings):
    port: int = int(os.getenv("PORT", "8001"))
    host: str = os.getenv("HOST", "0.0.0.0")
    database_url: str = os.getenv(
        "DATABASE_URL", 
        "postgres://postgres:postgres@localhost:5432/exposureiq_db"
    )
    anthropic_api_key: str = os.getenv("ANTHROPIC_API_KEY", "")
    claude_model: str = os.getenv("CLAUDE_MODEL", "claude-3-5-sonnet-20241022")
    surya_ocr_url: str = os.getenv("SURYA_OCR_URL", "http://localhost:5000/ocr")
    director_whatsapp: str = os.getenv("DIRECTOR_WHATSAPP_PHONE", "5215512345678")
    world_intel_mcp_url: str = os.getenv("WORLD_INTEL_MCP_URL", "http://localhost:8088")

settings = Settings()
