-- =============================================================================
-- ExposureIQ — Migración 008: Limpieza de número telefónico de ejemplo residual
-- =============================================================================

-- 1. Si la configuración del sistema aún tiene el número de ejemplo '5215512345678', limpiarla a vacío
UPDATE configuraciones_sistema
SET valor = '', actualizado_en = now()
WHERE clave = 'DIRECTOR_WHATSAPP_PHONE' AND valor = '5215512345678';

-- 2. Si la lista de distribución tiene una fila con el número dummy '5215512345678', eliminarla
DELETE FROM lista_distribucion
WHERE telefono = '5215512345678';
