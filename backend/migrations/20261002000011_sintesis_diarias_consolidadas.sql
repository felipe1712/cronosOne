-- =============================================================================
-- ExposureIQ — Base de Datos: exposureiq_db
-- Migración 011: Síntesis Diarias Consolidadas y Universo de Documentos
-- =============================================================================

-- 1. Tabla de Síntesis Diarias Consolidadas (una única síntesis por fecha)
CREATE TABLE IF NOT EXISTS sintesis_diarias (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    fecha           DATE UNIQUE NOT NULL,
    texto           TEXT NOT NULL DEFAULT '',
    temas           TEXT[] DEFAULT '{}',
    documentos_ids  UUID[] DEFAULT '{}',
    estado          VARCHAR(30) NOT NULL DEFAULT 'borrador'
                    CHECK (estado IN ('borrador', 'procesando', 'sintesis_lista', 'aprobado', 'enviado', 'error')),
    modelo_usado    VARCHAR(50) DEFAULT 'claude-sonnet-4-5-20250929',
    tokens_usados   INT,
    aprobado_por    UUID REFERENCES usuarios(id) ON DELETE SET NULL,
    creado_en       TIMESTAMPTZ NOT NULL DEFAULT now(),
    actualizado_en  TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 2. Añadir columnas a boletines para control de inclusión y origen
ALTER TABLE boletines ADD COLUMN IF NOT EXISTS incluido_en_sintesis BOOLEAN NOT NULL DEFAULT TRUE;
ALTER TABLE boletines ADD COLUMN IF NOT EXISTS origen VARCHAR(50) NOT NULL DEFAULT 'manual';

-- 3. Backfill de origen para documentos existentes del Senado
UPDATE boletines 
SET origen = 'senado' 
WHERE nombre_archivo ILIKE '%senado%' AND origen = 'manual';

-- 4. Índice para búsquedas rápidas por fecha y origen
CREATE INDEX IF NOT EXISTS idx_boletines_fecha_origen ON boletines (fecha_boletin, origen);
CREATE INDEX IF NOT EXISTS idx_sintesis_diarias_fecha ON sintesis_diarias (fecha);
