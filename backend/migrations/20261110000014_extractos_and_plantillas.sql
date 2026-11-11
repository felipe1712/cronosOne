-- =============================================================================
-- ExposureIQ — Migración 014: Tabla Extractos, Administrador de Plantillas y Primeras Planas
-- =============================================================================

-- 1. Tabla Extractos: Resumen ejecutivo acotado exclusivo para envío por WhatsApp
CREATE TABLE IF NOT EXISTS extractos (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    sintesis_id     UUID REFERENCES sintesis_diarias(id) ON DELETE CASCADE,
    fecha           DATE UNIQUE NOT NULL,
    texto           TEXT NOT NULL CHECK (char_length(texto) <= 1024),
    origen          VARCHAR(20) NOT NULL DEFAULT 'corte'
                    CHECK (origen IN ('corte', 'claude', 'manual')),
    modelo_usado    VARCHAR(80),
    tokens_usados   INT,
    editado_por     UUID REFERENCES usuarios(id) ON DELETE SET NULL,
    creado_en       TIMESTAMPTZ NOT NULL DEFAULT now(),
    actualizado_en  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_extractos_fecha ON extractos (fecha DESC);

-- 2. Página pública del CMS para "Primeras Planas"
INSERT INTO news_pages (section_key, title, subtitle, color, icon, orden, active)
VALUES (
    'primeras_planas',
    'Primeras Planas',
    'Lo más relevante de las portadas nacionales del día',
    '#7c3aed',
    'newspaper',
    0,
    true
)
ON CONFLICT (section_key) DO UPDATE
SET title = EXCLUDED.title,
    subtitle = EXCLUDED.subtitle,
    color = EXCLUDED.color,
    icon = EXCLUDED.icon,
    active = true,
    updated_at = now();

-- 3. Catálogo / Administrador de Plantillas WhatsApp de Kapso
CREATE TABLE IF NOT EXISTS whatsapp_plantillas (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    nombre              VARCHAR(512) NOT NULL,
    idioma              VARCHAR(10) NOT NULL DEFAULT 'es_MX',
    activa              BOOLEAN NOT NULL DEFAULT false,
    texto_fijo_body     TEXT NOT NULL DEFAULT '',
    tope_extracto       INT NOT NULL DEFAULT 900 CHECK (tope_extracto BETWEEN 1 AND 1024),
    parametros          JSONB NOT NULL DEFAULT '[]',
    kapso_template_id   VARCHAR(100),
    creado_en           TIMESTAMPTZ NOT NULL DEFAULT now(),
    actualizado_en      TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT uq_plantilla_nombre_idioma UNIQUE (nombre, idioma)
);

CREATE UNIQUE INDEX IF NOT EXISTS uq_plantilla_activa ON whatsapp_plantillas (activa) WHERE activa = true;

-- Seed inicial de la plantilla oficial brief_informativo_cp_v2
INSERT INTO whatsapp_plantillas (
    nombre,
    idioma,
    activa,
    texto_fijo_body,
    tope_extracto,
    parametros
)
VALUES (
    'brief_informativo_cp_v2',
    'es_MX',
    true,
    'PRIMERAS PLANAS \n\n .\n\n Si no deseas seguir recibiendo estos mensajes manda la palabra "STOP"',
    900,
    '[
        {"componente": "header", "nombre": "date", "fuente": "fecha", "formato": "D de Mes"},
        {"componente": "body", "nombre": "primeras_planas", "fuente": "extracto"}
    ]'::jsonb
)
ON CONFLICT (nombre, idioma) DO UPDATE
SET activa = true,
    tope_extracto = EXCLUDED.tope_extracto,
    parametros = EXCLUDED.parametros,
    updated_at = now();

-- Actualizar valor de configuración activa si existe la tabla configuraciones_sistema
UPDATE configuraciones_sistema
SET valor = 'brief_informativo_cp_v2', actualizado_en = now()
WHERE clave = 'WHATSAPP_TEMPLATE_NAME';
