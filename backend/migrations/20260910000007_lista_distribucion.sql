-- =============================================================================
-- ExposureIQ — Migración 007: Lista de Distribución de Destinatarios WhatsApp
-- =============================================================================

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

-- Si la tabla está vacía, sembramos con el teléfono del director configurado
INSERT INTO lista_distribucion (nombre, telefono, cargo, activo, notas)
SELECT 
    'Director de Operaciones', 
    valor, 
    'Dirección Ejecutiva / Operaciones', 
    true,
    'Contacto principal inicial del sistema'
FROM configuraciones_sistema
WHERE clave = 'DIRECTOR_WHATSAPP_PHONE'
  AND valor IS NOT NULL 
  AND valor != ''
  AND valor != '5215512345678'
  AND NOT EXISTS (SELECT 1 FROM lista_distribucion LIMIT 1);
