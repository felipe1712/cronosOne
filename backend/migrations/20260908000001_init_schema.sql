-- =============================================================================
-- ExposureIQ — Base de Datos: exposureiq_db
-- Migración 001: Esquema inicial (usuarios, boletines, secciones, síntesis, mensajes, OSINT)
-- =============================================================================

-- Extensión para generación de UUIDs si no está habilitada
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- 1. Usuarios del panel administrativo y operativo (admin, director, analista)
CREATE TABLE IF NOT EXISTS usuarios (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    nombre          VARCHAR(150) NOT NULL,
    correo          VARCHAR(150) UNIQUE NOT NULL,
    rol             VARCHAR(20) NOT NULL CHECK (rol IN ('admin', 'director', 'analista')),
    password_hash   TEXT NOT NULL,
    activo          BOOLEAN NOT NULL DEFAULT TRUE,
    creado_en       TIMESTAMPTZ NOT NULL DEFAULT now(),
    actualizado_en  TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 2. Registro de cada PDF diario cargado (Boletín Coparmex)
CREATE TABLE IF NOT EXISTS boletines (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    fecha_boletin   DATE NOT NULL,
    nombre_archivo  VARCHAR(255) NOT NULL,
    ruta_archivo    TEXT NOT NULL,
    subido_por      UUID REFERENCES usuarios(id) ON DELETE SET NULL,
    estado          VARCHAR(20) NOT NULL DEFAULT 'pendiente_ocr'
                    CHECK (estado IN ('pendiente_ocr', 'en_ocr', 'ocr_completo', 'error_ocr', 'sintesis_lista', 'error_sintesis')),
    total_paginas   INT,
    error_mensaje   TEXT,
    creado_en       TIMESTAMPTZ NOT NULL DEFAULT now(),
    actualizado_en  TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 3. Secciones extraídas del boletín tras el procesamiento OCR (Surya)
CREATE TABLE IF NOT EXISTS secciones (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    boletin_id      UUID NOT NULL REFERENCES boletines(id) ON DELETE CASCADE,
    orden           INT NOT NULL,
    tema            VARCHAR(150),
    pagina_inicio   INT,
    pagina_fin      INT,
    contenido       JSONB NOT NULL,   -- texto OCR estructurado con bloques, párrafos o jerarquía
    creado_en       TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 4. Síntesis ejecutiva diaria generada por el LLM (Claude API)
CREATE TABLE IF NOT EXISTS sintesis_generadas (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    boletin_id      UUID NOT NULL REFERENCES boletines(id) ON DELETE CASCADE,
    texto           TEXT NOT NULL,    -- Resumen ejecutivo optimizado para WhatsApp
    temas           TEXT[] DEFAULT '{}', -- ej. {"regulación", "seguridad", "movilidad"}
    modelo_usado    VARCHAR(50) DEFAULT 'claude-3-5-sonnet',
    tokens_usados   INT,
    creado_en       TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 5. Cola unificada de mensajes pendientes de envío para el cron de n8n / WAHA
CREATE TABLE IF NOT EXISTS mensajes_pendientes (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tipo            VARCHAR(20) NOT NULL CHECK (tipo IN ('brief', 'alerta')),
    referencia_id   UUID,             -- boletin_id o alerta_id
    texto           TEXT NOT NULL,
    destinatario    VARCHAR(50) NOT NULL,   -- Número WhatsApp del Director (E.164)
    estado          VARCHAR(20) NOT NULL DEFAULT 'pendiente'
                    CHECK (estado IN ('pendiente', 'entregado_a_n8n', 'confirmado', 'error')),
    intento_conteo  INT NOT NULL DEFAULT 0,
    error_mensaje   TEXT,
    creado_en       TIMESTAMPTZ NOT NULL DEFAULT now(),
    entregado_en    TIMESTAMPTZ,
    confirmado_en   TIMESTAMPTZ
);

-- 6. Entidades vigiladas (marcas, dominios, directivos, pólizas) para world-intel-mcp
CREATE TABLE IF NOT EXISTS entidades_vigiladas (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tipo            VARCHAR(30) NOT NULL CHECK (tipo IN ('marca', 'dominio', 'ejecutivo', 'patron_poliza')),
    valor           VARCHAR(255) NOT NULL,
    descripcion     TEXT,
    activo          BOOLEAN NOT NULL DEFAULT TRUE,
    creado_en       TIMESTAMPTZ NOT NULL DEFAULT now(),
    actualizado_en  TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 7. Alertas OSINT de redes y dark web detectadas por world-intel-mcp
CREATE TABLE IF NOT EXISTS alertas_osint (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entidad_id      UUID REFERENCES entidades_vigiladas(id) ON DELETE SET NULL,
    fuente          VARCHAR(100) NOT NULL, -- ej. "darkweb_forum", "telegram", "x_twitter"
    titulo          VARCHAR(255) NOT NULL,
    descripcion     TEXT NOT NULL,
    severidad       VARCHAR(20) NOT NULL DEFAULT 'media' CHECK (severidad IN ('baja', 'media', 'alta', 'critica')),
    evidencia       JSONB,                 -- Datos sanitizados, metadatos y enlaces seguros
    estado          VARCHAR(20) NOT NULL DEFAULT 'por_validar'
                    CHECK (estado IN ('por_validar', 'validada', 'descartada')),
    validada_por    UUID REFERENCES usuarios(id) ON DELETE SET NULL,
    notas_analista  TEXT,
    creado_en       TIMESTAMPTZ NOT NULL DEFAULT now(),
    validada_en     TIMESTAMPTZ
);

-- 8. Índices de optimización
CREATE INDEX IF NOT EXISTS idx_boletines_fecha ON boletines(fecha_boletin);
CREATE INDEX IF NOT EXISTS idx_boletines_estado ON boletines(estado);
CREATE INDEX IF NOT EXISTS idx_secciones_boletin ON secciones(boletin_id);
CREATE INDEX IF NOT EXISTS idx_secciones_tema ON secciones(tema);
CREATE INDEX IF NOT EXISTS idx_mensajes_estado ON mensajes_pendientes(estado);
CREATE INDEX IF NOT EXISTS idx_mensajes_creado_en ON mensajes_pendientes(creado_en);
CREATE INDEX IF NOT EXISTS idx_alertas_estado ON alertas_osint(estado);
CREATE INDEX IF NOT EXISTS idx_alertas_severidad ON alertas_osint(severidad);
CREATE INDEX IF NOT EXISTS idx_entidades_activo ON entidades_vigiladas(activo);
