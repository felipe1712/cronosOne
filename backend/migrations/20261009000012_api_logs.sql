-- Migración 12: Tabla de auditoría y trazabilidad api_logs (Claude y WhatsApp)
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE TABLE IF NOT EXISTS api_logs (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    servicio                VARCHAR(50) NOT NULL,
    accion                  VARCHAR(100) NOT NULL,
    modelo_o_proveedor      VARCHAR(100),
    estado                  VARCHAR(50) NOT NULL,
    codigo_http             INT,
    latencia_ms             INT,
    tokens_input            INT DEFAULT 0,
    tokens_output           INT DEFAULT 0,
    tokens_total            INT DEFAULT 0,
    max_tokens_configurado  INT,
    destinatario            VARCHAR(100),
    peticion_payload        TEXT,
    respuesta_payload       TEXT,
    error_mensaje           TEXT,
    creado_en               TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_api_logs_servicio_fecha ON api_logs (servicio, creado_en DESC);
CREATE INDEX IF NOT EXISTS idx_api_logs_creado_en ON api_logs (creado_en DESC);
CREATE INDEX IF NOT EXISTS idx_api_logs_estado ON api_logs (estado);
