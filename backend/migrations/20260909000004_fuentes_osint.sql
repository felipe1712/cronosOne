-- =============================================================================
-- ExposureIQ — Base de Datos: exposureiq_db
-- Migración 004: Administrador de Fuentes OSINT (world-intel-mcp)
-- =============================================================================

CREATE TABLE IF NOT EXISTS fuentes_osint (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    clave           VARCHAR(50) UNIQUE NOT NULL,
    nombre          VARCHAR(100) NOT NULL,
    descripcion     TEXT NOT NULL,
    tipo            VARCHAR(50) NOT NULL DEFAULT 'mcp_service', -- 'mcp_service', 'scraper', 'rss', 'api'
    activo          BOOLEAN NOT NULL DEFAULT TRUE,
    ultimo_escaneo  TIMESTAMPTZ,
    total_hallazgos INT NOT NULL DEFAULT 0,
    creado_en       TIMESTAMPTZ NOT NULL DEFAULT now(),
    actualizado_en  TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Fuentes de inteligencia preconfiguradas para world-intel-mcp
INSERT INTO fuentes_osint (clave, nombre, descripcion, tipo, activo)
VALUES 
    (
        'darkweb_breaches', 
        'Foros & Marketplaces Dark Web', 
        'Monitoreo en redes Onion de foros de ciberdelincuencia (Breached, Exploit, XSS) buscando bases de datos y credenciales filtradas.', 
        'mcp_service', 
        true
    ),
    (
        'paste_sites', 
        'Sitios de Código y Fugas (Pastes)', 
        'Escaneo en Pastebin, Rentry y repositorios públicos (GitHub Gists) detectando filtraciones de pólizas o tokens.', 
        'scraper', 
        true
    ),
    (
        'telegram_intel', 
        'Canales de Hacktivismo Telegram', 
        'Interceptación de canales de hacktivistas y grupos cibercriminales dirigidos a infraestructuras críticas e instituciones mexicanas.', 
        'api', 
        true
    ),
    (
        'regulatory_gazette', 
        'DOF & Circulares CNSF', 
        'Vigilancia regulatoria diaria sobre sanciones, nuevas reglas operativas y avisos de la Comisión Nacional de Seguros y Fianzas.', 
        'rss', 
        true
    ),
    (
        'financial_news', 
        'Medios Económicos & Siniestralidad', 
        'Portales de noticias financieras, notas de siniestros de alto impacto y riesgo reputacional en el sector asegurador mexicano.', 
        'rss', 
        true
    ),
    (
        'x_twitter_feed', 
        'Monitoreo en X / Redes Sociales', 
        'Detección de incidentes graves en tiempo real, quejas virales de asegurados y reportes de siniestros catastróficos en X/Twitter.', 
        'api', 
        true
    )
ON CONFLICT (clave) DO NOTHING;

CREATE INDEX IF NOT EXISTS idx_fuentes_osint_activo ON fuentes_osint(activo);
