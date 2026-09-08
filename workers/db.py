import psycopg2
from psycopg2.extras import RealDictCursor
from config import settings

def get_db_connection():
    """Retorna una conexión a PostgreSQL con cursor en formato diccionario."""
    return psycopg2.connect(settings.database_url, cursor_factory=RealDictCursor)
