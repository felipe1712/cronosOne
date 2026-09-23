-- =============================================================================
-- ExposureIQ — Migración 009: Permitir estado 'aprobado' y 'enviado' en boletines
-- =============================================================================

ALTER TABLE boletines DROP CONSTRAINT IF EXISTS boletines_estado_check;
ALTER TABLE boletines ADD CONSTRAINT boletines_estado_check 
    CHECK (estado IN ('pendiente_ocr', 'en_ocr', 'ocr_completo', 'error_ocr', 'sintesis_lista', 'error_sintesis', 'aprobado', 'enviado'));
