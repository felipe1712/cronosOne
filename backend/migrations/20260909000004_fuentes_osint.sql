-- =============================================================================
-- ExposureIQ & SentinelIQ — Base de Datos: exposureiq_db
-- Migración 004: Administrador de Fuentes OSINT (world-intel-mcp)
-- =============================================================================

CREATE TABLE IF NOT EXISTS fuentes_osint (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    clave           VARCHAR(50) UNIQUE NOT NULL,
    nombre          VARCHAR(100) NOT NULL,
    descripcion     TEXT NOT NULL,
    dominio         VARCHAR(50) NOT NULL DEFAULT 'ciberseguridad', -- 'ciberseguridad', 'noticias_geopolitica', 'catastrofes_clima', 'financiero_regulatorio'
    tipo            VARCHAR(50) NOT NULL DEFAULT 'mcp_service',   -- 'mcp_service', 'scraper', 'rss', 'api'
    activo          BOOLEAN NOT NULL DEFAULT TRUE,
    ultimo_escaneo  TIMESTAMPTZ,
    total_hallazgos INT NOT NULL DEFAULT 0,
    creado_en       TIMESTAMPTZ NOT NULL DEFAULT now(),
    actualizado_en  TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Si la tabla ya existía, asegurar la columna dominio
ALTER TABLE fuentes_osint ADD COLUMN IF NOT EXISTS dominio VARCHAR(50) NOT NULL DEFAULT 'ciberseguridad';

-- 12 Fuentes de inteligencia de world-intel-mcp organizadas por dominio
INSERT INTO fuentes_osint (clave, nombre, descripcion, dominio, tipo, activo)
VALUES 
    -- 1. CIBERSEGURIDAD & AMENAZAS
    (
        'cyber_cisa', 
        'CISA Advisories & KEV Catalog', 
        'Catálogo oficial de vulnerabilidades explotadas activamente y directivas de emergencia de CISA/DHS.', 
        'ciberseguridad', 
        'mcp_service', 
        true
    ),
    (
        'cyber_ransomware', 
        'Ransomware Victims & Leaks', 
        'Rastreo de sitios de extorsión de bandas cibercriminales (LockBit, BlackCat, RansomHub) y filtraciones corporativas.', 
        'ciberseguridad', 
        'mcp_service', 
        true
    ),
    (
        'cyber_cve_recent', 
        'Vulnerabilidades Críticas (CVEs)', 
        'Monitoreo del National Vulnerability Database (NVD) para exploits públicos de severidad Alta y Crítica.', 
        'ciberseguridad', 
        'api', 
        true
    ),
    (
        'cyber_threat_actors', 
        'Threat Actors & Dark Web Dumps', 
        'Inteligencia sobre actores de amenazas, foros clandestinos (Breached, Exploit) y repositorios públicos con fugas de datos.', 
        'ciberseguridad', 
        'mcp_service', 
        true
    ),

    -- 2. NOTICIAS GLOBALES & GEOPOLÍTICA
    (
        'news_global_rss', 
        '47 Feeds Globales de Noticias', 
        'Monitoreo continuo de agencias internacionales y prensa financiera (Reuters, AP, Al Jazeera, Bloomberg, medios nacionales).', 
        'noticias_geopolitica', 
        'rss', 
        true
    ),
    (
        'news_sanctions', 
        'Sanciones Internacionales (OFAC / ONU)', 
        'Listas de observación de personas bloqueadas (SDN), entidades sancionadas y riesgos de lavado de dinero.', 
        'noticias_geopolitica', 
        'mcp_service', 
        true
    ),
    (
        'geopolitics_conflict', 
        'Conflictos & Inestabilidad Regional', 
        'Eventos de disturbios civiles, bloqueos carreteros y alertas de seguridad pública de alto impacto.', 
        'noticias_geopolitica', 
        'mcp_service', 
        true
    ),

    -- 3. CATASTROFES & CLIMA (Impacto en Seguros y Operaciones)
    (
        'disaster_earthquakes', 
        'Sismos en Tiempo Real (USGS)', 
        'Monitoreo sísmico en vivo del Servicio Geológico de EE.UU. con epicentro, magnitud y radio de afectación.', 
        'catastrofes_clima', 
        'api', 
        true
    ),
    (
        'disaster_wildfires', 
        'Incendios Forestales (NASA FIRMS)', 
        'Detección satelital térmica de incendios activos en zonas industriales, agrícolas y urbanas.', 
        'catastrofes_clima', 
        'mcp_service', 
        true
    ),
    (
        'disaster_weather', 
        'Alertas Meteorológicas Extremas', 
        'Rastreo de ciclones, huracanes, inundaciones y fenómenos climáticos con potencial de siniestro catastrófico.', 
        'catastrofes_clima', 
        'rss', 
        true
    ),

    -- 4. FINANCIERO & REGULATORIO
    (
        'finance_sec_filings', 
        'Reportes Regulatorios (SEC 8-K / 10-K)', 
        'Filings ante autoridades regulatorias sobre ciberataques materiales, demandas y contingencias corporativas.', 
        'financiero_regulatorio', 
        'mcp_service', 
        true
    ),
    (
        'finance_macro_signals', 
        'Señales Macroeconómicas & Divisas', 
        'Indicadores macroeconómicos clave, volatilidad cambiaria (USD/MXN) e índices de presión inflacionaria.', 
        'financiero_regulatorio', 
        'api', 
        true
    )
ON CONFLICT (clave) DO UPDATE 
SET nombre = EXCLUDED.nombre,
    descripcion = EXCLUDED.descripcion,
    dominio = EXCLUDED.dominio,
    tipo = EXCLUDED.tipo;

CREATE INDEX IF NOT EXISTS idx_fuentes_osint_activo ON fuentes_osint(activo);
CREATE INDEX IF NOT EXISTS idx_fuentes_osint_dominio ON fuentes_osint(dominio);
