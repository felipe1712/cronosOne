-- =============================================================================
-- ExposureIQ — Base de Datos: exposureiq_db
-- Migración 003: Tabla de estado de sesión de WhatsApp (administrado vía n8n)
-- =============================================================================

CREATE TABLE IF NOT EXISTS whatsapp_sesion (
    id              VARCHAR(50) PRIMARY KEY DEFAULT 'default',
    estado          VARCHAR(50) NOT NULL DEFAULT 'DISCONNECTED', -- 'CONNECTED', 'SCAN_QR_CODE', 'STOPPED', etc.
    qr_code         TEXT,
    detalles        JSONB,
    actualizado_en  TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Fila inicial por defecto
INSERT INTO whatsapp_sesion (id, estado, detalles)
VALUES ('default', 'DISCONNECTED', '{"info": "Esperando webhook de sesión desde n8n"}'::jsonb)
ON CONFLICT (id) DO NOTHING;
