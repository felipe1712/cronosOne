import psycopg2
from psycopg2.extras import RealDictCursor
from config import settings

def get_db_connection():
    """Retorna una conexión a PostgreSQL con cursor en formato diccionario."""
    return psycopg2.connect(settings.database_url, cursor_factory=RealDictCursor)

def ensure_database_schema():
    """Verifica y asegura la existencia de sintesis_diarias y columnas requeridas en PostgreSQL."""
    try:
        conn = get_db_connection()
        cur = conn.cursor()
        cur.execute("""
            CREATE EXTENSION IF NOT EXISTS "pgcrypto";

            CREATE TABLE IF NOT EXISTS sintesis_diarias (
                id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                fecha           DATE UNIQUE NOT NULL,
                texto           TEXT NOT NULL DEFAULT '',
                temas           TEXT[] DEFAULT '{}',
                documentos_ids  UUID[] DEFAULT '{}',
                estado          VARCHAR(30) NOT NULL DEFAULT 'borrador',
                modelo_usado    VARCHAR(50) DEFAULT 'claude-sonnet-4-5-20250929',
                tokens_usados   INT,
                aprobado_por    UUID,
                creado_en       TIMESTAMPTZ NOT NULL DEFAULT now(),
                actualizado_en  TIMESTAMPTZ NOT NULL DEFAULT now()
            );

            ALTER TABLE boletines ADD COLUMN IF NOT EXISTS incluido_en_sintesis BOOLEAN NOT NULL DEFAULT TRUE;
            ALTER TABLE boletines ADD COLUMN IF NOT EXISTS origen VARCHAR(50) NOT NULL DEFAULT 'manual';
            ALTER TABLE sintesis_diarias ADD COLUMN IF NOT EXISTS temas TEXT[] DEFAULT '{}';

            CREATE INDEX IF NOT EXISTS idx_boletines_fecha_origen ON boletines (fecha_boletin, origen);
            CREATE INDEX IF NOT EXISTS idx_sintesis_diarias_fecha ON sintesis_diarias (fecha);
        """)
        conn.commit()
        cur.close()
        conn.close()
        print("[DB Worker] ✅ Esquema sintesis_diarias y columnas adicionales verificados/creados exitosamente.")
    except Exception as e:
        print(f"[DB Worker] ⚠️ Advertencia verificando esquema en PostgreSQL: {e}")

