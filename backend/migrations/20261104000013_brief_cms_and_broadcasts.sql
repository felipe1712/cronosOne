-- =============================================================================
-- ExposureIQ — Base de Datos: exposureiq_db
-- Migración 013: CMS de Páginas Blog, Ediciones Diarias y Broadcasts Kapso
-- =============================================================================

-- 1. Catálogo de Páginas / Secciones Administrables (CMS)
CREATE TABLE IF NOT EXISTS news_pages (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    section_key         VARCHAR(50) UNIQUE NOT NULL,      -- 'politica', 'economia', 'seguridad'
    title               VARCHAR(150) NOT NULL,            -- 'Política y Gobierno'
    subtitle            VARCHAR(250),                     -- 'Congreso, reformas y gobernabilidad'
    color               VARCHAR(30) NOT NULL DEFAULT '#0284c7',   -- Color distintivo de la sección
    icon                VARCHAR(50) NOT NULL DEFAULT 'landmark',  -- Identificador de icono
    orden               INT NOT NULL DEFAULT 1,
    active              BOOLEAN NOT NULL DEFAULT true,    -- Para dar de baja páginas
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Seed inicial de las 3 páginas principales del brief
INSERT INTO news_pages (section_key, title, subtitle, color, icon, orden, active)
VALUES 
    ('politica', 'Política y Gobierno', 'Análisis legislativo, reformas, acuerdos parlamentarios y política nacional', '#0284c7', 'landmark', 1, true),
    ('economia', 'Economía y Finanzas', 'Indicadores de mercado, deuda soberana, inflación y actividad económica', '#16a34a', 'trending-up', 2, true),
    ('seguridad', 'Seguridad Nacional', 'Seguridad pública, incidentes carreteros, siniestralidad e inteligencia', '#dc2626', 'shield-alert', 3, true)
ON CONFLICT (section_key) DO UPDATE
SET title = EXCLUDED.title,
    subtitle = EXCLUDED.subtitle,
    color = EXCLUDED.color,
    icon = EXCLUDED.icon,
    updated_at = now();

-- 2. Ediciones Diarias Consolidadas
CREATE TABLE IF NOT EXISTS news_editions (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    edition_date        DATE UNIQUE NOT NULL,
    status              VARCHAR(30) NOT NULL DEFAULT 'draft'
                        CHECK (status IN ('draft', 'review', 'published', 'archived')),
    generated_at        TIMESTAMPTZ DEFAULT now(),
    published_at        TIMESTAMPTZ,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_news_editions_date ON news_editions (edition_date DESC);
CREATE INDEX IF NOT EXISTS idx_news_editions_status ON news_editions (status);

-- 3. Secciones Específicas por Edición
CREATE TABLE IF NOT EXISTS news_sections (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    edition_id          UUID NOT NULL REFERENCES news_editions(id) ON DELETE CASCADE,
    section_key         VARCHAR(50) NOT NULL REFERENCES news_pages(section_key) ON UPDATE CASCADE,
    title               VARCHAR(200) NOT NULL,
    summary             TEXT NOT NULL DEFAULT '',
    full_content        TEXT NOT NULL DEFAULT '',
    public_slug         VARCHAR(250) NOT NULL,
    published           BOOLEAN NOT NULL DEFAULT false,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (edition_id, section_key)
);

CREATE INDEX IF NOT EXISTS idx_news_sections_key_pub ON news_sections (section_key, published, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_news_sections_edition ON news_sections (edition_id);

-- 4. Suscriptores con trazabilidad estricta de consentimiento y Opt-in
CREATE TABLE IF NOT EXISTS subscribers (
    id                          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    phone_number                VARCHAR(30) UNIQUE NOT NULL, -- Formato E.164 (+52...)
    display_name                VARCHAR(150),
    consent_status              VARCHAR(30) NOT NULL DEFAULT 'subscribed'
                                CHECK (consent_status IN ('subscribed', 'unsubscribed', 'blocked', 'invalid', 'pending')),
    consent_source              VARCHAR(50) NOT NULL DEFAULT 'migration_lista_distribucion',
    consent_text_version        VARCHAR(50) NOT NULL DEFAULT 'whatsapp_news_v1',
    consented_at                TIMESTAMPTZ NOT NULL DEFAULT now(),
    unsubscribed_at             TIMESTAMPTZ,
    last_marketing_preference   VARCHAR(50), -- 'stopped', 'resumed'
    created_at                  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at                  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_subscribers_active ON subscribers (consent_status);

-- Migración inicial de contactos activos desde lista_distribucion a subscribers
INSERT INTO subscribers (
    phone_number,
    display_name,
    consent_status,
    consent_source,
    consent_text_version,
    consented_at,
    created_at,
    updated_at
)
SELECT 
    CASE 
        WHEN telefono LIKE '+%' THEN telefono
        WHEN telefono LIKE '52%' THEN '+' || telefono
        ELSE '+52' || regexp_replace(telefono, '\D', '', 'g')
    END AS phone_number,
    nombre AS display_name,
    'subscribed' AS consent_status,
    'migration_lista_distribucion' AS consent_source,
    'whatsapp_news_v1' AS consent_text_version,
    creado_en AS consented_at,
    creado_en AS created_at,
    now() AS updated_at
FROM lista_distribucion
WHERE activo = true AND length(regexp_replace(telefono, '\D', '', 'g')) >= 10
ON CONFLICT (phone_number) DO UPDATE
SET display_name = EXCLUDED.display_name,
    consent_status = 'subscribed',
    updated_at = now();

-- 5. Registro Maestro de Broadcasts
CREATE TABLE IF NOT EXISTS whatsapp_broadcasts (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    edition_id          UUID REFERENCES news_editions(id) ON DELETE SET NULL,
    broadcast_id        VARCHAR(100) UNIQUE NOT NULL, -- UUID asignado por Kapso Platform
    phone_number_id     VARCHAR(50) NOT NULL,
    template_id         VARCHAR(100) NOT NULL,
    edition_date        DATE NOT NULL,
    status              VARCHAR(50) NOT NULL DEFAULT 'draft', -- draft, scheduled, sending, completed, failed, stopped
    scheduled_at        TIMESTAMPTZ,
    total_recipients    INT DEFAULT 0,
    sent_count          INT DEFAULT 0,
    delivered_count     INT DEFAULT 0,
    read_count          INT DEFAULT 0,
    failed_count        INT DEFAULT 0,
    suppressed_count    INT DEFAULT 0,
    responded_count     INT DEFAULT 0,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT uq_broadcast_edition_template UNIQUE (edition_date, phone_number_id, template_id)
);

CREATE INDEX IF NOT EXISTS idx_broadcasts_date ON whatsapp_broadcasts (edition_date DESC);
CREATE INDEX IF NOT EXISTS idx_broadcasts_status ON whatsapp_broadcasts (status);

-- 6. Trazabilidad por Destinatario en Broadcast
CREATE TABLE IF NOT EXISTS whatsapp_broadcast_recipients (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    broadcast_id        UUID NOT NULL REFERENCES whatsapp_broadcasts(id) ON DELETE CASCADE,
    subscriber_id       UUID REFERENCES subscribers(id) ON DELETE SET NULL,
    phone_number        VARCHAR(30) NOT NULL,
    kapso_recipient_id  VARCHAR(100),
    status              VARCHAR(50) NOT NULL DEFAULT 'pending', -- pending, sent, delivered, read, failed, suppressed
    wamid               VARCHAR(150),
    error_code          VARCHAR(50),
    error_message       TEXT,
    sent_at             TIMESTAMPTZ,
    delivered_at        TIMESTAMPTZ,
    read_at             TIMESTAMPTZ,
    responded_at        TIMESTAMPTZ,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_recipients_wamid ON whatsapp_broadcast_recipients (wamid);
CREATE INDEX IF NOT EXISTS idx_recipients_broadcast ON whatsapp_broadcast_recipients (broadcast_id, status);

-- 7. Deduplicación e Idempotencia de Webhooks
CREATE TABLE IF NOT EXISTS whatsapp_message_events (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_id            VARCHAR(150) UNIQUE NOT NULL,
    message_id          VARCHAR(150) NOT NULL,
    event_type          VARCHAR(100) NOT NULL,
    payload_hash        VARCHAR(64) NOT NULL,
    received_at         TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_msg_events_wamid ON whatsapp_message_events (message_id);

-- 8. Backfill automático: crear ediciones a partir de sintesis_diarias existentes
DO $$
DECLARE
    rec RECORD;
    v_edition_id UUID;
BEGIN
    FOR rec IN SELECT fecha, texto, estado, creado_en FROM sintesis_diarias LOOP
        INSERT INTO news_editions (edition_date, status, generated_at, published_at, created_at, updated_at)
        VALUES (
            rec.fecha, 
            CASE WHEN rec.estado IN ('aprobado', 'enviado') THEN 'published' ELSE 'draft' END,
            rec.creado_en,
            CASE WHEN rec.estado IN ('aprobado', 'enviado') THEN rec.creado_en ELSE NULL END,
            rec.creado_en,
            now()
        )
        ON CONFLICT (edition_date) DO NOTHING
        RETURNING id INTO v_edition_id;

        IF v_edition_id IS NOT NULL THEN
            -- Crear al menos la sección de política con el contenido consolidado existente
            INSERT INTO news_sections (edition_id, section_key, title, summary, full_content, public_slug, published)
            VALUES (
                v_edition_id,
                'politica',
                'Síntesis Política y Legislativa del Día',
                SUBSTRING(rec.texto, 1, 300),
                rec.texto,
                'politica',
                CASE WHEN rec.estado IN ('aprobado', 'enviado') THEN true ELSE false END
            )
            ON CONFLICT (edition_id, section_key) DO NOTHING;
        END IF;
    END LOOP;
END $$;
