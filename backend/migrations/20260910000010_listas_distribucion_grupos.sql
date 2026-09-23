-- =============================================================================
-- ExposureIQ — Migración 010: Grupos / Listas de Distribución y Relación N a M
-- =============================================================================

-- 1. Asegurar tabla base de destinatarios
CREATE TABLE IF NOT EXISTS lista_distribucion (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    nombre          VARCHAR(100) NOT NULL,
    telefono        VARCHAR(50) NOT NULL,
    cargo           VARCHAR(100),
    activo          BOOLEAN NOT NULL DEFAULT true,
    notas           TEXT,
    creado_en       TIMESTAMPTZ NOT NULL DEFAULT now(),
    actualizado_en  TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 2. Tabla de listas / grupos temáticos de distribución
CREATE TABLE IF NOT EXISTS grupos_distribucion (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    nombre          VARCHAR(100) NOT NULL UNIQUE,
    descripcion     TEXT,
    color           VARCHAR(30) NOT NULL DEFAULT '#0284c7',
    activo          BOOLEAN NOT NULL DEFAULT true,
    creado_en       TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 3. Tabla intermedia (N a M) entre destinatarios y grupos
CREATE TABLE IF NOT EXISTS destinatarios_grupos (
    destinatario_id UUID NOT NULL REFERENCES lista_distribucion(id) ON DELETE CASCADE,
    grupo_id        UUID NOT NULL REFERENCES grupos_distribucion(id) ON DELETE CASCADE,
    creado_en       TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (destinatario_id, grupo_id)
);

CREATE INDEX IF NOT EXISTS idx_dest_grupos_dest ON destinatarios_grupos(destinatario_id);
CREATE INDEX IF NOT EXISTS idx_dest_grupos_grupo ON destinatarios_grupos(grupo_id);

-- 4. Semilla inicial de grupos sugeridos si la tabla está vacía
INSERT INTO grupos_distribucion (nombre, descripcion, color)
VALUES 
    ('Comité Directivo', 'Recepción de briefings matutinos ejecutivos y decisiones clave', '#0284c7'),
    ('Operaciones & Siniestros', 'Alertas tempranas de seguridad, incidentes carreteros y siniestros', '#16a34a'),
    ('Legal & Regulatorio', 'Reformas jurídicas, CNSF, SHCP, jurisprudencia y laboral', '#9333ea')
ON CONFLICT (nombre) DO NOTHING;
