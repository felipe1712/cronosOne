-- =============================================================================
-- ExposureIQ — Migración 005: Tabla de Configuración Dinámica del Sistema
-- Permite configurar y alternar modelos de IA (Claude) desde la interfaz web.
-- =============================================================================

CREATE TABLE IF NOT EXISTS configuraciones_sistema (
    clave           VARCHAR(100) PRIMARY KEY,
    valor           TEXT NOT NULL,
    descripcion     TEXT,
    categoria       VARCHAR(50) DEFAULT 'ia',
    actualizado_en  TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Configuración por defecto de Claude
INSERT INTO configuraciones_sistema (clave, valor, descripcion, categoria)
VALUES 
    ('CLAUDE_MODEL', 'claude-3-5-sonnet-20241022', 'Modelo de Anthropic Claude seleccionado para la síntesis de boletines', 'ia'),
    ('CLAUDE_MAX_TOKENS', '1000', 'Límite máximo de tokens por síntesis', 'ia')
ON CONFLICT (clave) DO NOTHING;
