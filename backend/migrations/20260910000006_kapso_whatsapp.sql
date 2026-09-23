-- =============================================================================
-- ExposureIQ — Migración 006: Integración de WhatsApp Cloud API vía Kapso
-- =============================================================================

-- 1. Asegurar tabla de configuraciones y registrar parámetros de Kapso
CREATE TABLE IF NOT EXISTS configuraciones_sistema (
    clave           VARCHAR(100) PRIMARY KEY,
    valor           TEXT NOT NULL,
    descripcion     TEXT,
    categoria       VARCHAR(50) DEFAULT 'ia',
    actualizado_en  TIMESTAMPTZ NOT NULL DEFAULT now()
);

INSERT INTO configuraciones_sistema (clave, valor, descripcion, categoria)
VALUES 
    ('WHATSAPP_PROVIDER', 'kapso', 'Proveedor activo de WhatsApp: kapso o waha', 'whatsapp'),
    ('KAPSO_API_KEY', '', 'Clave de API del proyecto en Kapso (X-API-Key)', 'whatsapp'),
    ('KAPSO_PHONE_NUMBER_ID', '', 'Identificador de número telefónico de WhatsApp en Kapso / Meta', 'whatsapp'),
    ('DIRECTOR_WHATSAPP_PHONE', '5215512345678', 'Número de WhatsApp de destino del Director en formato E.164', 'whatsapp')
ON CONFLICT (clave) DO NOTHING;

-- 2. Extender tabla de mensajes_pendientes con metadatos de Kapso y estado de Meta
ALTER TABLE mensajes_pendientes 
    ADD COLUMN IF NOT EXISTS proveedor VARCHAR(30) DEFAULT 'kapso',
    ADD COLUMN IF NOT EXISTS kapso_message_id TEXT,
    ADD COLUMN IF NOT EXISTS meta_status VARCHAR(30) DEFAULT 'pendiente';
