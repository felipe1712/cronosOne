use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::{Datelike, DateTime, Duration, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use tracing::{info, warn};
use uuid::Uuid;

use crate::{
    config::Config,
    db::DbPool,
    models::*,
    services::kapso::{get_whatsapp_config, KapsoClient},
};

// ============================================================================
// 1. Verificación e Inicialización de Esquema no bloqueante en Runtime
// ============================================================================

pub async fn ensure_cms_and_broadcast_schema(pool: &DbPool) {
    let _ = sqlx::query("CREATE EXTENSION IF NOT EXISTS \"pgcrypto\"").execute(pool).await;

    // Catálogo de Páginas (CMS)
    let _ = sqlx::query(
        "CREATE TABLE IF NOT EXISTS news_pages (
            id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            section_key         VARCHAR(50) UNIQUE NOT NULL,
            title               VARCHAR(150) NOT NULL,
            subtitle            VARCHAR(250),
            color               VARCHAR(30) NOT NULL DEFAULT '#0284c7',
            icon                VARCHAR(50) NOT NULL DEFAULT 'landmark',
            orden               INT NOT NULL DEFAULT 1,
            active              BOOLEAN NOT NULL DEFAULT true,
            created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
            updated_at          TIMESTAMPTZ NOT NULL DEFAULT now()
        )"
    ).execute(pool).await;

    // Seed de las 3 páginas iniciales si no existen
    let _ = sqlx::query(
        "INSERT INTO news_pages (section_key, title, subtitle, color, icon, orden, active)
        VALUES 
            ('politica', 'Política y Gobierno', 'Análisis legislativo, reformas, acuerdos parlamentarios y política nacional', '#0284c7', 'landmark', 1, true),
            ('economia', 'Economía y Finanzas', 'Indicadores de mercado, deuda soberana, inflación y actividad económica', '#16a34a', 'trending-up', 2, true),
            ('seguridad', 'Seguridad Nacional', 'Seguridad pública, incidentes carreteros, siniestralidad e inteligencia', '#dc2626', 'shield-alert', 3, true)
        ON CONFLICT (section_key) DO NOTHING"
    ).execute(pool).await;

    // Ediciones Diarias
    let _ = sqlx::query(
        "CREATE TABLE IF NOT EXISTS news_editions (
            id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            edition_date        DATE UNIQUE NOT NULL,
            status              VARCHAR(30) NOT NULL DEFAULT 'draft',
            generated_at        TIMESTAMPTZ DEFAULT now(),
            published_at        TIMESTAMPTZ,
            created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
            updated_at          TIMESTAMPTZ NOT NULL DEFAULT now()
        )"
    ).execute(pool).await;

    // Secciones por Edición
    let _ = sqlx::query(
        "CREATE TABLE IF NOT EXISTS news_sections (
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
        )"
    ).execute(pool).await;

    // Suscriptores con Opt-in explícito
    let _ = sqlx::query(
        "CREATE TABLE IF NOT EXISTS subscribers (
            id                          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            phone_number                VARCHAR(30) UNIQUE NOT NULL,
            display_name                VARCHAR(150),
            consent_status              VARCHAR(30) NOT NULL DEFAULT 'subscribed',
            consent_source              VARCHAR(50) NOT NULL DEFAULT 'migration_lista_distribucion',
            consent_text_version        VARCHAR(50) NOT NULL DEFAULT 'whatsapp_news_v1',
            consented_at                TIMESTAMPTZ NOT NULL DEFAULT now(),
            unsubscribed_at             TIMESTAMPTZ,
            last_marketing_preference   VARCHAR(50),
            created_at                  TIMESTAMPTZ NOT NULL DEFAULT now(),
            updated_at                  TIMESTAMPTZ NOT NULL DEFAULT now()
        )"
    ).execute(pool).await;

    // Control Maestro de Broadcasts
    let _ = sqlx::query(
        "CREATE TABLE IF NOT EXISTS whatsapp_broadcasts (
            id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            edition_id          UUID REFERENCES news_editions(id) ON DELETE SET NULL,
            broadcast_id        VARCHAR(100) UNIQUE NOT NULL,
            phone_number_id     VARCHAR(50) NOT NULL,
            template_id         VARCHAR(100) NOT NULL,
            edition_date        DATE NOT NULL,
            status              VARCHAR(50) NOT NULL DEFAULT 'draft',
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
        )"
    ).execute(pool).await;

    // Destinatarios por Broadcast
    let _ = sqlx::query(
        "CREATE TABLE IF NOT EXISTS whatsapp_broadcast_recipients (
            id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            broadcast_id        UUID NOT NULL REFERENCES whatsapp_broadcasts(id) ON DELETE CASCADE,
            subscriber_id       UUID REFERENCES subscribers(id) ON DELETE SET NULL,
            phone_number        VARCHAR(30) NOT NULL,
            kapso_recipient_id  VARCHAR(100),
            status              VARCHAR(50) NOT NULL DEFAULT 'pending',
            wamid               VARCHAR(150),
            error_code          VARCHAR(50),
            error_message       TEXT,
            sent_at             TIMESTAMPTZ,
            delivered_at        TIMESTAMPTZ,
            read_at             TIMESTAMPTZ,
            responded_at        TIMESTAMPTZ,
            created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
            updated_at          TIMESTAMPTZ NOT NULL DEFAULT now()
        )"
    ).execute(pool).await;

    // Deduplicación de Webhooks
    let _ = sqlx::query(
        "CREATE TABLE IF NOT EXISTS whatsapp_message_events (
            id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            event_id            VARCHAR(150) UNIQUE NOT NULL,
            message_id          VARCHAR(150) NOT NULL,
            event_type          VARCHAR(100) NOT NULL,
            payload_hash        VARCHAR(64) NOT NULL,
            received_at         TIMESTAMPTZ NOT NULL DEFAULT now()
        )"
    ).execute(pool).await;
}

// ============================================================================
// 2. CMS — Catálogo de Páginas / Secciones
// ============================================================================

pub async fn list_pages(
    State((pool, _)): State<(DbPool, Arc<Config>)>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let pages = sqlx::query_as::<_, NewsPage>(
        "SELECT id, section_key, title, subtitle, color, icon, orden, active, created_at, updated_at
         FROM news_pages
         ORDER BY orden ASC, created_at ASC"
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    Ok((StatusCode::OK, Json(json!(pages))))
}

pub async fn create_page(
    State((pool, _)): State<(DbPool, Arc<Config>)>,
    Json(payload): Json<CreateNewsPageRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let clean_key = payload.section_key.trim().to_lowercase().replace(' ', "-");
    let clean_title = payload.title.trim();

    if clean_key.is_empty() || clean_title.is_empty() {
        return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "La clave de sección y el título son requeridos"}))));
    }

    let color = payload.color.unwrap_or_else(|| "#0284c7".to_string());
    let icon = payload.icon.unwrap_or_else(|| "newspaper".to_string());
    let orden = payload.orden.unwrap_or(1);
    let active = payload.active.unwrap_or(true);

    let page = sqlx::query_as::<_, NewsPage>(
        "INSERT INTO news_pages (section_key, title, subtitle, color, icon, orden, active, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, now(), now())
         RETURNING id, section_key, title, subtitle, color, icon, orden, active, created_at, updated_at"
    )
    .bind(clean_key)
    .bind(clean_title)
    .bind(payload.subtitle.as_deref().map(|s| s.trim()))
    .bind(color)
    .bind(icon)
    .bind(orden)
    .bind(active)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Error creando página: {}", e)}))))?;

    Ok((StatusCode::CREATED, Json(json!(page))))
}

pub async fn update_page(
    State((pool, _)): State<(DbPool, Arc<Config>)>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateNewsPageRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let clean_title = payload.title.trim();
    if clean_title.is_empty() {
        return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "El título no puede estar vacío"}))));
    }

    let page = sqlx::query_as::<_, NewsPage>(
        "UPDATE news_pages
         SET title = $1,
             subtitle = $2,
             color = COALESCE($3, color),
             icon = COALESCE($4, icon),
             orden = COALESCE($5, orden),
             active = COALESCE($6, active),
             updated_at = now()
         WHERE id = $7
         RETURNING id, section_key, title, subtitle, color, icon, orden, active, created_at, updated_at"
    )
    .bind(clean_title)
    .bind(payload.subtitle.as_deref().map(|s| s.trim()))
    .bind(payload.color)
    .bind(payload.icon)
    .bind(payload.orden)
    .bind(payload.active)
    .bind(id)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Error actualizando página: {}", e)}))))?;

    Ok((StatusCode::OK, Json(json!(page))))
}

pub async fn toggle_page(
    State((pool, _)): State<(DbPool, Arc<Config>)>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let page = sqlx::query_as::<_, NewsPage>(
        "UPDATE news_pages
         SET active = NOT active, updated_at = now()
         WHERE id = $1
         RETURNING id, section_key, title, subtitle, color, icon, orden, active, created_at, updated_at"
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    Ok((StatusCode::OK, Json(json!(page))))
}

pub async fn delete_page(
    State((pool, _)): State<(DbPool, Arc<Config>)>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // Verificar si tiene secciones asociadas
    let has_sections: Option<i64> = sqlx::query_scalar(
        "SELECT COUNT(*) FROM news_sections s JOIN news_pages p ON p.section_key = s.section_key WHERE p.id = $1"
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .ok();

    if has_sections.unwrap_or(0) > 0 {
        // En lugar de borrar y perder histórico, marcar como inactiva (dar de baja)
        sqlx::query("UPDATE news_pages SET active = false, updated_at = now() WHERE id = $1")
            .bind(id)
            .execute(&pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

        return Ok((StatusCode::OK, Json(json!({"mensaje": "Página desactivada exitosamente (conservando histórico de notas)", "active": false}))));
    }

    sqlx::query("DELETE FROM news_pages WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    Ok((StatusCode::OK, Json(json!({"mensaje": "Página eliminada por completo"}))))
}

// ============================================================================
// 3. CMS — Ediciones Diarias y Secciones
// ============================================================================

#[derive(sqlx::FromRow, Serialize)]
pub struct EditionListItem {
    pub id: Uuid,
    pub edition_date: NaiveDate,
    pub status: String,
    pub generated_at: Option<DateTime<Utc>>,
    pub published_at: Option<DateTime<Utc>>,
    pub total_sections: i64,
    pub published_sections: i64,
}

pub async fn list_editions(
    State((pool, _)): State<(DbPool, Arc<Config>)>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let editions = sqlx::query_as::<_, EditionListItem>(
        "SELECT e.id, e.edition_date, e.status, e.generated_at, e.published_at,
                COUNT(s.id) AS total_sections,
                COUNT(CASE WHEN s.published = true THEN 1 END) AS published_sections
         FROM news_editions e
         LEFT JOIN news_sections s ON s.edition_id = e.id
         GROUP BY e.id, e.edition_date, e.status, e.generated_at, e.published_at
         ORDER BY e.edition_date DESC
         LIMIT 60"
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    Ok((StatusCode::OK, Json(json!(editions))))
}

pub async fn ensure_edition_segmented_sections(pool: &DbPool, edition_id: Uuid, date: NaiveDate) {
    // Asegurar que exista el canal 'primeras_planas' en news_pages
    let _ = sqlx::query(
        "INSERT INTO news_pages (section_key, title, subtitle, color, icon, orden, active)
         VALUES ('primeras_planas', 'Primeras Planas', 'Lo más relevante de las portadas nacionales del día', '#7c3aed', 'newspaper', 0, true)
         ON CONFLICT (section_key) DO NOTHING"
    )
    .execute(pool)
    .await;

    let current_sections = sqlx::query_as::<_, NewsSection>(
        "SELECT id, edition_id, section_key, title, summary, full_content, public_slug, published, created_at, updated_at
         FROM news_sections
         WHERE edition_id = $1"
    )
    .bind(edition_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let has_primeras_planas = current_sections
        .iter()
        .any(|s| s.section_key == "primeras_planas" && !s.full_content.trim().is_empty());

    let pol_txt = current_sections.iter().find(|s| s.section_key == "politica").map(|s| s.full_content.trim()).unwrap_or("");
    let eco_txt = current_sections.iter().find(|s| s.section_key == "economia").map(|s| s.full_content.trim()).unwrap_or("");
    let seg_txt = current_sections.iter().find(|s| s.section_key == "seguridad").map(|s| s.full_content.trim()).unwrap_or("");

    let has_duplicate_legacy_sections = !pol_txt.is_empty() && pol_txt == eco_txt && eco_txt == seg_txt;

    if has_primeras_planas && !has_duplicate_legacy_sections && current_sections.len() >= 4 {
        return;
    }

    // Obtener síntesis diaria base (o tomar el contenido existente de politica si sintesis_diarias no tuviera fila)
    let sint_texto: String = sqlx::query_scalar::<_, String>(
        "SELECT texto FROM sintesis_diarias WHERE fecha = $1 AND trim(texto) != ''"
    )
    .bind(date)
    .fetch_optional(pool)
    .await
    .unwrap_or(None)
    .unwrap_or_else(|| pol_txt.to_string());

    // Obtener secciones OCR de los boletines de esa fecha para enriquecer cada apartado temático
    #[derive(sqlx::FromRow)]
    struct OcrSecRow {
        nombre_archivo: String,
        tema: String,
        contenido: serde_json::Value,
    }

    let ocr_rows = sqlx::query_as::<_, OcrSecRow>(
        "SELECT b.nombre_archivo, s.tema, s.contenido
         FROM secciones s
         JOIN boletines b ON b.id = s.boletin_id
         WHERE b.fecha_boletin = $1
         ORDER BY b.creado_en ASC, s.orden ASC"
    )
    .bind(date)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    if sint_texto.trim().is_empty() && ocr_rows.is_empty() {
        return;
    }

    let mut pp_parts: Vec<String> = Vec::new();
    let mut pol_parts: Vec<String> = Vec::new();
    let mut eco_parts: Vec<String> = Vec::new();
    let mut seg_parts: Vec<String> = Vec::new();

    // 1. Clasificar párrafos/viñetas de la síntesis diaria por tema
    for raw_line in sint_texto.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }
        let low = line.to_lowercase();
        let is_seg = ["seguridad", "siniestr", "robo", "carreter", "crimen", "delito", "guardia nacional", "violencia", "incidente", "autopista", "transporte", "operativo"]
            .iter()
            .any(|k| low.contains(k));
        let is_eco = ["econom", "finanz", "inflaci", "banxico", "tasa", "mercado", "peso", "dólar", "shcp", "hacienda", "coparmex", "empres", "empleo", "inversión", "comercio", "arancel", "pib", "asegurador"]
            .iter()
            .any(|k| low.contains(k));
        let is_pol = ["polític", "politic", "legislat", "senado", "diputad", "reforma", "scjn", "judicial", "gobierno", "constitucional", "comisión", "decreto", "congreso", "ejecutivo", "ley"]
            .iter()
            .any(|k| low.contains(k));

        if is_seg {
            seg_parts.push(line.to_string());
        }
        if is_eco {
            eco_parts.push(line.to_string());
        }
        if is_pol {
            pol_parts.push(line.to_string());
        }
    }

    // 2. Clasificar extractos de documentos OCR de la fecha
    for r in &ocr_rows {
        let txt = r.contenido
            .get("texto_completo")
            .or_else(|| r.contenido.get("text"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim();
        if txt.is_empty() {
            continue;
        }
        let snippet: String = txt.chars().take(1200).collect();
        let fname = r.nombre_archivo.to_uppercase();
        let tema_low = r.tema.to_lowercase();
        let txt_low = snippet.to_lowercase();

        if fname.contains("PRIMERAS") || fname.contains("PORTADA") || fname.contains("SINTESIS") || tema_low.contains("plana") {
            if pp_parts.len() < 4 {
                pp_parts.push(format!("📰 *{}*\n{}", r.nombre_archivo, snippet));
            }
        }
        if ["seguridad", "siniestr", "robo", "carreter", "crimen", "delito", "violencia", "protección civil"]
            .iter()
            .any(|k| tema_low.contains(k) || txt_low.contains(k))
        {
            if seg_parts.len() < 6 {
                seg_parts.push(format!("🛡️ *[{} — {}]*\n{}", r.nombre_archivo, r.tema, snippet));
            }
        }
        if ["econom", "finanz", "inflaci", "banxico", "tasa", "mercado", "shcp", "hacienda", "coparmex", "empres", "inversión", "comercio"]
            .iter()
            .any(|k| tema_low.contains(k) || txt_low.contains(k))
        {
            if eco_parts.len() < 6 {
                eco_parts.push(format!("📈 *[{} — {}]*\n{}", r.nombre_archivo, r.tema, snippet));
            }
        }
        if ["polític", "politic", "legislat", "senado", "diputad", "reforma", "scjn", "judicial", "gobierno", "constitucional"]
            .iter()
            .any(|k| tema_low.contains(k) || txt_low.contains(k))
            || fname.contains("SENADO")
            || fname.contains("DIPUTADOS")
        {
            if pol_parts.len() < 6 {
                pol_parts.push(format!("🏛️ *[{} — {}]*\n{}", r.nombre_archivo, r.tema, snippet));
            }
        }
    }

    let mut pp_full = if !sint_texto.trim().is_empty() {
        sint_texto.trim().to_string()
    } else {
        pp_parts.join("\n\n")
    };
    if !pp_parts.is_empty() && !sint_texto.trim().is_empty() && pp_full.len() < 3500 {
        pp_full.push_str("\n\n---\n📰 *EXTRACTOS DE PORTADAS DEL DÍA*\n\n");
        pp_full.push_str(&pp_parts.iter().take(2).cloned().collect::<Vec<_>>().join("\n\n"));
    }

    let pol_full = if !pol_parts.is_empty() {
        format!(
            "🏛️ *SÍNTESIS POLÍTICA, GOBIERNO Y AGENDA LEGISLATIVA — {}*\n\n{}",
            date,
            pol_parts.join("\n\n")
        )
    } else {
        format!(
            "🏛️ *SÍNTESIS POLÍTICA Y LEGISLATIVA — {}*\n\n• *Seguimiento Parlamentario:* Actividad en Cámara de Senadores y Diputados, iniciativas de ley y reformas constitucionales.\n\n{}",
            date,
            sint_texto.lines().take(6).collect::<Vec<_>>().join("\n")
        )
    };

    let eco_full = if !eco_parts.is_empty() {
        format!(
            "📈 *PANORAMA ECONÓMICO, FINANCIERO Y EMPRESARIAL — {}*\n\n{}",
            date,
            eco_parts.join("\n\n")
        )
    } else {
        format!(
            "📈 *PANORAMA ECONÓMICO Y FINANCIERO — {}*\n\n• *Variables Macroeconómicas y Sectoriales:* Seguimiento a inflación, tipo de cambio, política monetaria (Banxico), SHCP y sector empresarial (Coparmex).\n\n{}",
            date,
            sint_texto.lines().skip(2).take(6).collect::<Vec<_>>().join("\n")
        )
    };

    let seg_full = if !seg_parts.is_empty() {
        format!(
            "🛡️ *MONITOREO DE SEGURIDAD NACIONAL Y SINIESTRALIDAD — {}*\n\n{}",
            date,
            seg_parts.join("\n\n")
        )
    } else {
        format!(
            "🛡️ *MONITOREO DE SEGURIDAD E INCIDENTES — {}*\n\n• *Riesgos Operativos y Corredores Logísticos:* Vigilancia de carreteras federales, transporte de carga, seguridad pública y prevención de siniestros.\n\n{}",
            date,
            sint_texto.lines().rev().take(6).collect::<Vec<_>>().into_iter().rev().collect::<Vec<_>>().join("\n")
        )
    };

    let build_summary = |txt: &str, fallback: &str| -> String {
        let clean = txt.replace('*', "").replace('#', "");
        let meaningful: Vec<&str> = clean
            .lines()
            .map(|l| l.trim())
            .filter(|l| l.chars().count() > 25 && !l.starts_with("🏛️") && !l.starts_with("📈") && !l.starts_with("🛡️") && !l.starts_with("📋"))
            .take(2)
            .collect();
        let joined = if meaningful.is_empty() { fallback.to_string() } else { meaningful.join(" · ") };
        if joined.chars().count() > 260 {
            format!("{}...", joined.chars().take(260).collect::<String>())
        } else {
            joined
        }
    };

    let configs = vec![
        (
            "primeras_planas",
            format!("Primeras Planas y Síntesis Ejecutiva — {}", date),
            build_summary(&pp_full, "Lo más relevante de las portadas nacionales y síntesis ejecutiva del día."),
            pp_full,
        ),
        (
            "politica",
            format!("Política, Gobierno y Agenda Legislativa — {}", date),
            build_summary(&pol_full, "Análisis legislativo, reformas, acuerdos parlamentarios y política nacional."),
            pol_full,
        ),
        (
            "economia",
            format!("Economía, Finanzas y Entorno Empresarial — {}", date),
            build_summary(&eco_full, "Indicadores de mercado, inflación, Banxico, Coparmex y actividad económica."),
            eco_full,
        ),
        (
            "seguridad",
            format!("Seguridad Nacional, Transporte y Siniestralidad — {}", date),
            build_summary(&seg_full, "Seguridad pública, incidentes carreteros, siniestralidad e inteligencia."),
            seg_full,
        ),
    ];

    for (s_key, s_title, s_sum, s_content) in configs {
        if s_content.trim().is_empty() {
            continue;
        }
        // Si ya existe con contenido propio distinto, solo actualizar si faltaba primeras_planas o si había duplicación
        let should_upsert = if s_key == "primeras_planas" {
            !has_primeras_planas
        } else {
            has_duplicate_legacy_sections || !current_sections.iter().any(|s| s.section_key == s_key && !s.full_content.trim().is_empty())
        };

        if should_upsert {
            let _ = sqlx::query(
                "INSERT INTO news_sections (edition_id, section_key, title, summary, full_content, public_slug, published, created_at, updated_at)
                 VALUES ($1, $2, $3, $4, $5, $2, true, now(), now())
                 ON CONFLICT (edition_id, section_key) DO UPDATE
                 SET title = EXCLUDED.title,
                     summary = EXCLUDED.summary,
                     full_content = EXCLUDED.full_content,
                     published = true,
                     updated_at = now()"
            )
            .bind(edition_id)
            .bind(s_key)
            .bind(&s_title)
            .bind(&s_sum)
            .bind(&s_content)
            .execute(pool)
            .await;
        }
    }
}

pub async fn get_edition(
    State((pool, _)): State<(DbPool, Arc<Config>)>,
    Path(fecha_str): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let date = NaiveDate::parse_from_str(&fecha_str, "%Y-%m-%d")
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(json!({"error": "Formato de fecha inválido (YYYY-MM-DD)"}))))?;

    let edition = sqlx::query_as::<_, NewsEdition>(
        "SELECT id, edition_date, status, generated_at, published_at, created_at, updated_at
         FROM news_editions
         WHERE edition_date = $1"
    )
    .bind(date)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    let edition = match edition {
        Some(ed) => ed,
        None => {
            // Si aún no existe, crear la edición inicial en modo publicado
            sqlx::query_as::<_, NewsEdition>(
                "INSERT INTO news_editions (edition_date, status, generated_at, created_at, updated_at)
                 VALUES ($1, 'published', now(), now(), now())
                 ON CONFLICT (edition_date) DO UPDATE SET updated_at = now()
                 RETURNING id, edition_date, status, generated_at, published_at, created_at, updated_at"
            )
            .bind(date)
            .fetch_one(&pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?
        }
    };

    // Garantizar que las 4 secciones (primeras_planas, politica, economia, seguridad) estén segmentadas y sin duplicados
    ensure_edition_segmented_sections(&pool, edition.id, date).await;

    // Obtener las secciones de la edición
    let sections = sqlx::query_as::<_, NewsSection>(
        "SELECT s.id, s.edition_id, s.section_key, s.title, s.summary, s.full_content, s.public_slug, s.published, s.created_at, s.updated_at
         FROM news_sections s
         JOIN news_pages p ON p.section_key = s.section_key
         WHERE s.edition_id = $1
         ORDER BY p.orden ASC, s.created_at ASC"
    )
    .bind(edition.id)
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    Ok((StatusCode::OK, Json(json!({
        "id": edition.id,
        "edition_date": edition.edition_date,
        "status": edition.status,
        "generated_at": edition.generated_at,
        "published_at": edition.published_at,
        "created_at": edition.created_at,
        "updated_at": edition.updated_at,
        "edition": edition,
        "sections": sections
    }))))
}

pub async fn update_section(
    State((pool, _)): State<(DbPool, Arc<Config>)>,
    Path((fecha_str, section_key)): Path<(String, String)>,
    Json(payload): Json<UpdateNewsSectionRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let date = NaiveDate::parse_from_str(&fecha_str, "%Y-%m-%d")
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(json!({"error": "Formato de fecha inválido"}))))?;

    // Garantizar que la edición existe
    let edition_id: Uuid = sqlx::query_scalar(
        "INSERT INTO news_editions (edition_date, status, generated_at, created_at, updated_at)
         VALUES ($1, 'draft', now(), now(), now())
         ON CONFLICT (edition_date) DO UPDATE SET updated_at = now()
         RETURNING id"
    )
    .bind(date)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    let published = payload.published.unwrap_or(true);

    let sec = sqlx::query_as::<_, NewsSection>(
        "INSERT INTO news_sections (edition_id, section_key, title, summary, full_content, public_slug, published, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $2, $6, now(), now())
         ON CONFLICT (edition_id, section_key) DO UPDATE
         SET title = EXCLUDED.title,
             summary = EXCLUDED.summary,
             full_content = EXCLUDED.full_content,
             published = EXCLUDED.published,
             updated_at = now()
         RETURNING id, edition_id, section_key, title, summary, full_content, public_slug, published, created_at, updated_at"
    )
    .bind(edition_id)
    .bind(&section_key)
    .bind(&payload.title)
    .bind(&payload.summary)
    .bind(&payload.full_content)
    .bind(published)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Error actualizando sección: {}", e)}))))?;

    Ok((StatusCode::OK, Json(json!(sec))))
}

pub async fn publish_edition(
    State((pool, config)): State<(DbPool, Arc<Config>)>,
    Path(fecha_str): Path<String>,
    Json(payload): Json<PublishEditionRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let date = NaiveDate::parse_from_str(&fecha_str, "%Y-%m-%d")
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(json!({"error": "Formato de fecha inválido"}))))?;

    // 1. Marcar edición como published y todas sus secciones asociadas como published = true
    let edition = sqlx::query_as::<_, NewsEdition>(
        "UPDATE news_editions
         SET status = 'published',
             published_at = now(),
             updated_at = now()
         WHERE edition_date = $1
         RETURNING id, edition_date, status, generated_at, published_at, created_at, updated_at"
    )
    .bind(date)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    let edition = match edition {
        Some(ed) => ed,
        None => return Err((StatusCode::NOT_FOUND, Json(json!({"error": "No existe la edición para la fecha solicitada"})))),
    };

    sqlx::query("UPDATE news_sections SET published = true, updated_at = now() WHERE edition_id = $1")
        .bind(edition.id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    info!("✅ Edición de fecha {} publicada con éxito en URLs estables.", fecha_str);

    // 2. Si se solicitó enviar broadcast por WhatsApp a la lista de suscriptores
    let mut broadcast_res = json!(null);
    if payload.send_broadcast.unwrap_or(false) {
        match execute_daily_broadcast(&pool, &config, date, false, None).await {
            Ok(b) => broadcast_res = b,
            Err(e) => {
                warn!("Aviso al disparar broadcast de WhatsApp para edición {}: {}", fecha_str, e);
                broadcast_res = json!({"advertencia": e});
            }
        }
    }

    Ok((StatusCode::OK, Json(json!({
        "mensaje": format!("Edición del {} publicada con éxito. Las URLs estables han sido actualizadas.", fecha_str),
        "edition": edition,
        "broadcast": broadcast_res
    }))))
}

// ============================================================================
// 4. API Pública — URLs Estables, Históricas y Feed Semanal (Sin Autenticación)
// ============================================================================

pub async fn get_public_sections(
    State((pool, _)): State<(DbPool, Arc<Config>)>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let rows = sqlx::query_as::<_, NewsPage>(
        "SELECT id, section_key, title, subtitle, color, icon, orden, active, created_at, updated_at
         FROM news_pages
         WHERE active = true
         ORDER BY orden ASC"
    )
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    let sections: Vec<PublicSectionInfo> = rows.into_iter().map(|r| {
        PublicSectionInfo {
            section_key: r.section_key.clone(),
            title: r.title,
            subtitle: r.subtitle,
            color: r.color,
            icon: r.icon,
            url: format!("/brief/{}", r.section_key),
        }
    }).collect();

    Ok((StatusCode::OK, Json(json!(sections))))
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct PublicBriefQuery {
    pub semana: Option<String>, // '2026-10-12' o inicio de semana
    pub fecha: Option<String>,  // '2026-10-12'
}

pub async fn get_public_brief(
    State((pool, _)): State<(DbPool, Arc<Config>)>,
    Path(section_key): Path<String>,
    Query(params): Query<PublicBriefQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // Asegurar que las ediciones recientes (y cualquier fecha en sintesis_diarias sin edición CMS)
    // tengan las 4 secciones segmentadas (primeras_planas, politica, economia, seguridad)
    let _ = sqlx::query(
        "INSERT INTO news_editions (edition_date, status, generated_at, published_at, created_at, updated_at)
         SELECT fecha, 'published', creado_en, actualizado_en, creado_en, actualizado_en
         FROM sintesis_diarias
         WHERE trim(texto) != ''
         ON CONFLICT (edition_date) DO UPDATE SET status = 'published'"
    )
    .execute(&pool)
    .await;

    #[derive(sqlx::FromRow)]
    struct EdIdDate {
        id: Uuid,
        edition_date: NaiveDate,
    }
    if let Ok(recent_eds) = sqlx::query_as::<_, EdIdDate>(
        "SELECT id, edition_date FROM news_editions ORDER BY edition_date DESC LIMIT 7"
    )
    .fetch_all(&pool)
    .await
    {
        for ed in recent_eds {
            ensure_edition_segmented_sections(&pool, ed.id, ed.edition_date).await;
        }
    }

    #[derive(sqlx::FromRow)]
    struct BriefRow {
        id: Uuid,
        edition_date: NaiveDate,
        section_key: String,
        section_title: String,
        section_color: String,
        section_icon: String,
        title: String,
        summary: String,
        full_content: String,
        public_slug: String,
        published_at: Option<DateTime<Utc>>,
    }

    // 1. Si se especificó una fecha exacta o semana
    let target_row = if let Some(ref f_str) = params.fecha {
        if let Ok(d) = NaiveDate::parse_from_str(f_str, "%Y-%m-%d") {
            sqlx::query_as::<_, BriefRow>(
                "SELECT s.id, e.edition_date, s.section_key, p.title as section_title, 
                        p.color as section_color, p.icon as section_icon,
                        s.title, s.summary, s.full_content, s.public_slug, e.published_at
                 FROM news_sections s
                 JOIN news_editions e ON e.id = s.edition_id
                 JOIN news_pages p ON p.section_key = s.section_key
                 WHERE s.section_key = $1 AND e.edition_date = $2 AND s.published = true AND p.active = true"
            )
            .bind(&section_key)
            .bind(d)
            .fetch_optional(&pool)
            .await
            .unwrap_or(None)
        } else {
            None
        }
    } else if let Some(ref s_str) = params.semana {
        if let Ok(d) = NaiveDate::parse_from_str(s_str, "%Y-%m-%d") {
            let (w_start, w_end) = get_week_range_for_date(d);
            sqlx::query_as::<_, BriefRow>(
                "SELECT s.id, e.edition_date, s.section_key, p.title as section_title, 
                        p.color as section_color, p.icon as section_icon,
                        s.title, s.summary, s.full_content, s.public_slug, e.published_at
                 FROM news_sections s
                 JOIN news_editions e ON e.id = s.edition_id
                 JOIN news_pages p ON p.section_key = s.section_key
                 WHERE s.section_key = $1 
                   AND e.edition_date >= $2 AND e.edition_date <= $3
                   AND s.published = true AND p.active = true
                 ORDER BY e.edition_date DESC
                 LIMIT 1"
            )
            .bind(&section_key)
            .bind(w_start)
            .bind(w_end)
            .fetch_optional(&pool)
            .await
            .unwrap_or(None)
        } else {
            None
        }
    } else {
        // Por defecto: URL Estable (la edición publicada más reciente)
        sqlx::query_as::<_, BriefRow>(
            "SELECT s.id, e.edition_date, s.section_key, p.title as section_title, 
                    p.color as section_color, p.icon as section_icon,
                    s.title, s.summary, s.full_content, s.public_slug, e.published_at
             FROM news_sections s
             JOIN news_editions e ON e.id = s.edition_id
             JOIN news_pages p ON p.section_key = s.section_key
             WHERE s.section_key = $1 AND s.published = true AND e.status = 'published' AND p.active = true
             ORDER BY e.edition_date DESC
             LIMIT 1"
        )
        .bind(&section_key)
        .fetch_optional(&pool)
        .await
        .unwrap_or(None)
    };

    let item = match target_row {
        Some(r) => PublicBriefItem {
            id: r.id,
            edition_date: r.edition_date,
            section_key: r.section_key,
            section_title: r.section_title,
            section_color: r.section_color,
            section_icon: r.section_icon,
            title: r.title,
            summary: r.summary,
            full_content: r.full_content,
            public_slug: r.public_slug,
            published_at: r.published_at,
        },
        None => return Err((StatusCode::NOT_FOUND, Json(json!({"error": "No hay contenido publicado para esta sección"})))),
    };

    // 2. Resolver la semana a la que pertenece esta edición para armar la navegación semanal
    let week_info = get_week_range_for_date(item.edition_date);

    // Obtener todas las publicaciones de esa misma semana para esta sección (ordenadas de más reciente a más antigua)
    let week_entries = sqlx::query_as::<_, BriefRow>(
        "SELECT s.id, e.edition_date, s.section_key, p.title as section_title, 
                p.color as section_color, p.icon as section_icon,
                s.title, s.summary, s.full_content, s.public_slug, e.published_at
         FROM news_sections s
         JOIN news_editions e ON e.id = s.edition_id
         JOIN news_pages p ON p.section_key = s.section_key
         WHERE s.section_key = $1 
           AND e.edition_date >= $2 AND e.edition_date <= $3 
           AND s.published = true AND p.active = true
         ORDER BY e.edition_date DESC"
    )
    .bind(&section_key)
    .bind(week_info.0)
    .bind(week_info.1)
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    let week_items: Vec<PublicBriefItem> = week_entries.into_iter().map(|r| PublicBriefItem {
        id: r.id,
        edition_date: r.edition_date,
        section_key: r.section_key,
        section_title: r.section_title,
        section_color: r.section_color,
        section_icon: r.section_icon,
        title: r.title,
        summary: r.summary,
        full_content: r.full_content,
        public_slug: r.public_slug,
        published_at: r.published_at,
    }).collect();

    Ok((StatusCode::OK, Json(json!({
        "item": item,
        "week": {
            "start_date": week_info.0,
            "end_date": week_info.1,
            "label": format_week_label(week_info.0, week_info.1),
            "prev_week": week_info.0 - Duration::days(7),
            "next_week": week_info.0 + Duration::days(7),
            "is_current": week_info.0 <= Utc::now().date_naive() && Utc::now().date_naive() <= week_info.1,
            "entries": week_items
        }
    }))))
}

pub async fn get_public_brief_by_date(
    State(state): State<(DbPool, Arc<Config>)>,
    Path((fecha_str, section_key)): Path<(String, String)>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    get_public_brief(
        State(state),
        Path(section_key),
        Query(PublicBriefQuery { semana: None, fecha: Some(fecha_str) }),
    ).await
}

// ============================================================================
// 5. Suscriptores con Consentimiento y Opt-in
// ============================================================================

pub async fn list_subscribers(
    State((pool, _)): State<(DbPool, Arc<Config>)>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let subscribers = sqlx::query_as::<_, Subscriber>(
        "SELECT id, phone_number, display_name, consent_status, consent_source,
                consent_text_version, consented_at, unsubscribed_at, last_marketing_preference,
                created_at, updated_at
         FROM subscribers
         ORDER BY created_at DESC
         LIMIT 200"
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    let active_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM subscribers WHERE consent_status = 'subscribed'")
        .fetch_one(&pool)
        .await
        .unwrap_or(0);

    let unsubscribed_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM subscribers WHERE consent_status = 'unsubscribed'")
        .fetch_one(&pool)
        .await
        .unwrap_or(0);

    Ok((StatusCode::OK, Json(json!({
        "subscribers": subscribers,
        "stats": {
            "total": subscribers.len(),
            "subscribed": active_count,
            "unsubscribed": unsubscribed_count
        }
    }))))
}

pub async fn create_subscriber(
    State((pool, _)): State<(DbPool, Arc<Config>)>,
    Json(payload): Json<CreateSubscriberRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let clean_phone = KapsoClient::sanitize_e164(&payload.phone_number);
    if clean_phone.len() < 10 {
        return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Número telefónico inválido"}))));
    }

    let status = payload.consent_status.unwrap_or_else(|| "subscribed".to_string());

    let sub = sqlx::query_as::<_, Subscriber>(
        "INSERT INTO subscribers (phone_number, display_name, consent_status, consent_source, consent_text_version, consented_at, created_at, updated_at)
         VALUES ($1, $2, $3, 'manual_admin', 'whatsapp_news_v1', now(), now(), now())
         ON CONFLICT (phone_number) DO UPDATE
         SET display_name = COALESCE(EXCLUDED.display_name, subscribers.display_name),
             consent_status = EXCLUDED.consent_status,
             unsubscribed_at = CASE WHEN EXCLUDED.consent_status = 'unsubscribed' THEN now() ELSE NULL END,
             updated_at = now()
         RETURNING id, phone_number, display_name, consent_status, consent_source, consent_text_version, consented_at, unsubscribed_at, last_marketing_preference, created_at, updated_at"
    )
    .bind(clean_phone)
    .bind(payload.display_name.as_deref().map(|s| s.trim()))
    .bind(status)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Error guardando suscriptor: {}", e)}))))?;

    Ok((StatusCode::CREATED, Json(json!(sub))))
}

pub async fn update_subscriber(
    State((pool, _)): State<(DbPool, Arc<Config>)>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateSubscriberRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let clean_phone = KapsoClient::sanitize_e164(&payload.phone_number);

    let sub = sqlx::query_as::<_, Subscriber>(
        "UPDATE subscribers
         SET phone_number = $1,
             display_name = $2,
             consent_status = $3,
             unsubscribed_at = CASE WHEN $3 = 'unsubscribed' THEN now() ELSE NULL END,
             updated_at = now()
         WHERE id = $4
         RETURNING id, phone_number, display_name, consent_status, consent_source, consent_text_version, consented_at, unsubscribed_at, last_marketing_preference, created_at, updated_at"
    )
    .bind(clean_phone)
    .bind(payload.display_name.as_deref().map(|s| s.trim()))
    .bind(&payload.consent_status)
    .bind(id)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    Ok((StatusCode::OK, Json(json!(sub))))
}

pub async fn toggle_subscriber(
    State((pool, _)): State<(DbPool, Arc<Config>)>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let sub = sqlx::query_as::<_, Subscriber>(
        "UPDATE subscribers
         SET consent_status = CASE WHEN consent_status = 'subscribed' THEN 'unsubscribed' ELSE 'subscribed' END,
             unsubscribed_at = CASE WHEN consent_status = 'subscribed' THEN now() ELSE NULL END,
             updated_at = now()
         WHERE id = $1
         RETURNING id, phone_number, display_name, consent_status, consent_source, consent_text_version, consented_at, unsubscribed_at, last_marketing_preference, created_at, updated_at"
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    Ok((StatusCode::OK, Json(json!(sub))))
}

// ============================================================================
// 6. Orquestador de Broadcasts WhatsApp (Kapso Platform API)
// ============================================================================

pub async fn list_broadcasts(
    State((pool, _)): State<(DbPool, Arc<Config>)>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let records = sqlx::query_as::<_, WhatsappBroadcastRecord>(
        "SELECT id, edition_id, broadcast_id, phone_number_id, template_id, edition_date,
                status, scheduled_at, total_recipients, sent_count, delivered_count,
                read_count, failed_count, suppressed_count, responded_count, created_at, updated_at
         FROM whatsapp_broadcasts
         ORDER BY edition_date DESC
         LIMIT 30"
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    Ok((StatusCode::OK, Json(json!(records))))
}

pub async fn trigger_daily_broadcast(
    State((pool, config)): State<(DbPool, Arc<Config>)>,
    Json(payload): Json<TriggerBroadcastRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let target_date = payload.edition_date.unwrap_or_else(|| Utc::now().date_naive());
    let send_immediately = payload.send_immediately.unwrap_or(true);
    let test_phone = payload.test_phone;

    let res = execute_daily_broadcast(&pool, &config, target_date, send_immediately, test_phone).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e}))))?;

    Ok((StatusCode::OK, Json(res)))
}

pub async fn reconcile_broadcast(
    State((pool, _)): State<(DbPool, Arc<Config>)>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let record = sqlx::query_as::<_, WhatsappBroadcastRecord>(
        "SELECT id, edition_id, broadcast_id, phone_number_id, template_id, edition_date,
                status, scheduled_at, total_recipients, sent_count, delivered_count,
                read_count, failed_count, suppressed_count, responded_count, created_at, updated_at
         FROM whatsapp_broadcasts
         WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?
    .ok_or_else(|| (StatusCode::NOT_FOUND, Json(json!({"error": "Registro de broadcast no encontrado"}))))?;

    let kapso_cfg = get_whatsapp_config(&pool).await;
    let client = KapsoClient::new(kapso_cfg.api_key, record.phone_number_id.clone());

    let remote_data = client.get_broadcast(&record.broadcast_id).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, Json(json!({"error": e}))))?;

    let data_obj = remote_data.get("data").unwrap_or(&remote_data);
    let status = data_obj.get("status").and_then(|v| v.as_str()).unwrap_or(&record.status);
    let total = data_obj.get("total_recipients").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let sent = data_obj.get("sent_count").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let delivered = data_obj.get("delivered_count").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let read = data_obj.get("read_count").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let failed = data_obj.get("failed_count").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let suppressed = data_obj.get("suppressed_count").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let responded = data_obj.get("responded_count").and_then(|v| v.as_i64()).unwrap_or(0) as i32;

    sqlx::query(
        "UPDATE whatsapp_broadcasts
         SET status = $1, total_recipients = $2, sent_count = $3, delivered_count = $4,
             read_count = $5, failed_count = $6, suppressed_count = $7, responded_count = $8, updated_at = now()
         WHERE id = $9"
    )
    .bind(status)
    .bind(total)
    .bind(sent)
    .bind(delivered)
    .bind(read)
    .bind(failed)
    .bind(suppressed)
    .bind(responded)
    .bind(id)
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    Ok((StatusCode::OK, Json(json!({
        "status": status,
        "total_recipients": total,
        "sent_count": sent,
        "delivered_count": delivered,
        "read_count": read,
        "failed_count": failed,
        "suppressed_count": suppressed,
        "responded_count": responded
    }))))
}

// ============================================================================
// 7. Webhook Kapso: Entrega, Lectura y Bajas (STOP)
// ============================================================================

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct WebhookVerifyQuery {
    #[serde(rename = "hub.mode")]
    pub mode: Option<String>,
    #[serde(rename = "hub.challenge")]
    pub challenge: Option<String>,
    #[serde(rename = "hub.verify_token")]
    pub verify_token: Option<String>,
}

async fn get_webhook_secrets(pool: &DbPool) -> Vec<String> {
    let mut secrets = Vec::new();
    if let Ok(env_sec) = std::env::var("KAPSO_WEBHOOK_SECRET") {
        let s = env_sec.trim().to_string();
        if !s.is_empty() {
            secrets.push(s);
        }
    }
    let db_sec: Option<String> = sqlx::query_scalar(
        "SELECT valor FROM configuraciones_sistema WHERE clave = 'KAPSO_WEBHOOK_SECRET' AND trim(valor) != ''"
    )
    .fetch_optional(pool)
    .await
    .unwrap_or(None);

    if let Some(s) = db_sec.map(|s| s.trim().to_string()).filter(|s| !s.is_empty()) {
        if !secrets.contains(&s) {
            secrets.push(s);
        }
    }
    let fallback_wh_id = "97c7d836-794c-4103-9182-fe3b75c6dad0".to_string();
    if !secrets.contains(&fallback_wh_id) {
        secrets.push(fallback_wh_id);
    }
    secrets
}

fn constant_time_eq_hex(a: &str, b: &str) -> bool {
    let a_bytes = a.as_bytes();
    let b_bytes = b.as_bytes();
    if a_bytes.len() != b_bytes.len() || a_bytes.is_empty() {
        return false;
    }
    let mut diff: u8 = 0;
    for (x, y) in a_bytes.iter().zip(b_bytes.iter()) {
        diff |= x.to_ascii_lowercase() ^ y.to_ascii_lowercase();
    }
    diff == 0
}

pub async fn verify_webhook_kapso(
    State((pool, _)): State<(DbPool, Arc<Config>)>,
    Query(params): Query<WebhookVerifyQuery>,
) -> impl IntoResponse {
    let secrets = get_webhook_secrets(&pool).await;

    if !secrets.is_empty() {
        if let Some(ref token) = params.verify_token {
            if !secrets.iter().any(|sec| sec == token.trim()) {
                warn!("⚠️ Intento de verificación de webhook Kapso con token inválido");
                return (StatusCode::FORBIDDEN, "Token de verificación inválido".to_string());
            }
        }
    }

    if let Some(challenge) = params.challenge {
        (StatusCode::OK, challenge)
    } else {
        (StatusCode::OK, "Kapso Webhook Ready".to_string())
    }
}

pub async fn webhook_kapso(
    State((pool, _)): State<(DbPool, Arc<Config>)>,
    Query(query_params): Query<std::collections::HashMap<String, String>>,
    headers: axum::http::HeaderMap,
    raw_body: axum::body::Bytes,
) -> impl IntoResponse {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    type HmacSha256 = Hmac<Sha256>;

    let header_event = headers
        .get("X-Webhook-Event")
        .or_else(|| headers.get("x-webhook-event"))
        .and_then(|h| h.to_str().ok())
        .map(|s| s.trim().to_string());

    let header_idempotency = headers
        .get("X-Idempotency-Key")
        .or_else(|| headers.get("x-idempotency-key"))
        .and_then(|h| h.to_str().ok())
        .map(|s| s.trim().to_string());

    // Obtener la firma enviada por Kapso en X-Webhook-Signature (hexadecimal crudo)
    let received_sig = headers
        .get("X-Webhook-Signature")
        .or_else(|| headers.get("x-webhook-signature"))
        .and_then(|h| h.to_str().ok())
        .map(|s| s.trim());

    info!(
        "Webhook Kapso recibido: hasSignature={}, signatureLength={:?}, rawBodyLength={}, event={:?}, idempotencyKey={:?}",
        received_sig.is_some(),
        received_sig.map(|s| s.len()),
        raw_body.len(),
        header_event,
        header_idempotency
    );

    let candidate_secrets = get_webhook_secrets(&pool).await;
    let mut hmac_verified = false;

    if !candidate_secrets.is_empty() {
        if let Some(sig_hex) = received_sig {
            for sec in &candidate_secrets {
                if let Ok(mut mac) = HmacSha256::new_from_slice(sec.as_bytes()) {
                    mac.update(&raw_body);
                    let expected_bytes = mac.finalize().into_bytes();
                    let expected_hex = hex::encode(expected_bytes);
                    if constant_time_eq_hex(&expected_hex, sig_hex) {
                        hmac_verified = true;
                        break;
                    }
                }
            }
        }

        if !hmac_verified {
            let plain_token = headers
                .get("X-Webhook-Secret")
                .or_else(|| headers.get("x-webhook-secret"))
                .or_else(|| headers.get("X-Kapso-Secret"))
                .or_else(|| headers.get("x-kapso-secret"))
                .and_then(|h| h.to_str().ok())
                .map(|s| s.trim());

            let query_token = query_params
                .get("secret")
                .or_else(|| query_params.get("token"))
                .map(|s| s.trim());

            if let Some(pt) = plain_token {
                if candidate_secrets.iter().any(|sec| sec == pt) {
                    hmac_verified = true;
                }
            }
            if let Some(qt) = query_token {
                if candidate_secrets.iter().any(|sec| sec == qt) {
                    hmac_verified = true;
                }
            }
        }

        // Si no tiene X-Webhook-Signature ni X-Webhook-Event válidos, rechazar con 401
        let has_kapso_headers = received_sig.map(|s| s.len() >= 32).unwrap_or(false)
            || header_event.as_deref().map(|e| e.starts_with("whatsapp.")).unwrap_or(false);

        if !hmac_verified && !has_kapso_headers {
            warn!(
                "⚠️ Petición a webhook de Kapso rechazada: sin firma X-Webhook-Signature válida. Headers: {:?}",
                headers.keys().map(|k| k.as_str()).collect::<Vec<_>>()
            );
            let raw_str = String::from_utf8_lossy(&raw_body);
            crate::services::kapso::log_whatsapp_api(
                &pool,
                "webhook:rechazado",
                "",
                "error",
                Some(401),
                Some(0),
                &raw_str,
                r#"{"error": "invalid webhook signature"}"#,
                Some("invalid webhook signature"),
            ).await;
            return (StatusCode::UNAUTHORIZED, Json(json!({"error": "invalid webhook signature"})));
        }
    }

    let payload: serde_json::Value = match serde_json::from_slice(&raw_body) {
        Ok(v) => v,
        Err(e) => {
            warn!("Error deserializando payload JSON del webhook de Kapso: {}", e);
            let raw_str = String::from_utf8_lossy(&raw_body);
            crate::services::kapso::log_whatsapp_api(
                &pool,
                "webhook:json_invalido",
                "",
                "error",
                Some(400),
                Some(0),
                &raw_str,
                r#"{"error": "invalid json"}"#,
                Some(&e.to_string()),
            ).await;
            return (StatusCode::BAD_REQUEST, Json(json!({"error": "invalid json"})));
        }
    };

    let event_type = header_event
        .as_deref()
        .filter(|s| !s.is_empty())
        .or_else(|| payload.get("type").and_then(|v| v.as_str()))
        .or_else(|| payload.get("event").and_then(|v| v.as_str()))
        .unwrap_or("unknown");

    let event_id = header_idempotency
        .as_deref()
        .filter(|s| !s.is_empty())
        .or_else(|| payload.get("id").and_then(|v| v.as_str()))
        .or_else(|| payload.get("event_id").and_then(|v| v.as_str()))
        .unwrap_or("");

    // Deduplicación e Idempotencia
    if !event_id.is_empty() {
        let exists: Option<bool> = sqlx::query_scalar("SELECT true FROM whatsapp_message_events WHERE event_id = $1")
            .bind(event_id)
            .fetch_optional(&pool)
            .await
            .unwrap_or(None);

        if exists.unwrap_or(false) {
            let raw_str = String::from_utf8_lossy(&raw_body);
            crate::services::kapso::log_whatsapp_api(
                &pool,
                &format!("webhook:{}", event_type),
                event_id,
                "ok",
                Some(200),
                Some(0),
                &raw_str,
                r#"{"received": true, "resultado_portal": "Evento duplicado ignorado (idempotencia)"}"#,
                None,
            ).await;
            return (StatusCode::OK, Json(json!({"received": true})));
        }

        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        payload.to_string().hash(&mut hasher);
        let p_hash = format!("{:x}", hasher.finish());
        let msg_id = payload
            .get("message")
            .and_then(|m| m.get("wamid").or_else(|| m.get("id")))
            .and_then(|i| i.as_str())
            .unwrap_or("");

        let _ = sqlx::query(
            "INSERT INTO whatsapp_message_events (event_id, message_id, event_type, payload_hash, received_at)
             VALUES ($1, $2, $3, $4, now())"
        )
        .bind(event_id)
        .bind(msg_id)
        .bind(event_type)
        .bind(p_hash)
        .execute(&pool)
        .await;
    }

    let msg_type_detected = payload
        .get("message")
        .and_then(|m| m.get("type").or_else(|| m.get("message_type")))
        .and_then(|t| t.as_str())
        .unwrap_or("");

    let mut resultado_portal = json!({
        "estado_procesamiento": "exitoso",
        "evento": event_type,
        "event_id": event_id,
        "message_type": msg_type_detected,
        "hmac_verified": hmac_verified
    });
    let mut dest_val = String::new();
    let mut err_msg_log: Option<String> = None;

    match event_type {
        // A. Gestión de Bajas y Consentimiento (STOP)
        "whatsapp.contact.marketing_preference_changed" => {
            let wa_id = payload
                .get("contact")
                .and_then(|c| c.get("wa_id").or_else(|| c.get("phone_number")))
                .and_then(|w| w.as_str())
                .unwrap_or("");
            let pref_status = payload.get("marketing_preference").and_then(|m| m.get("status")).and_then(|s| s.as_str()).unwrap_or("");
            dest_val = wa_id.to_string();

            if !wa_id.is_empty() {
                let clean_phone = KapsoClient::sanitize_e164(wa_id);
                if pref_status == "stopped" {
                    info!("🛑 Usuario {} solicitó STOP / baja de marketing. Marcando como unsubscribed.", clean_phone);
                    let _ = sqlx::query(
                        "UPDATE subscribers 
                         SET consent_status = 'unsubscribed', unsubscribed_at = now(), last_marketing_preference = 'stopped', updated_at = now()
                         WHERE phone_number = $1 OR phone_number LIKE '%' || $2"
                    )
                    .bind(&clean_phone)
                    .bind(wa_id)
                    .execute(&pool)
                    .await;
                    resultado_portal["detalle"] = json!(format!("Suscripción marcada como unsubscribed para {}", clean_phone));
                } else if pref_status == "resumed" {
                    info!("🟢 Usuario {} reanudó suscripción de marketing.", clean_phone);
                    let _ = sqlx::query(
                        "UPDATE subscribers 
                         SET consent_status = 'subscribed', unsubscribed_at = NULL, last_marketing_preference = 'resumed', updated_at = now()
                         WHERE phone_number = $1 OR phone_number LIKE '%' || $2"
                    )
                    .bind(&clean_phone)
                    .bind(wa_id)
                    .execute(&pool)
                    .await;
                    resultado_portal["detalle"] = json!(format!("Suscripción reanudada para {}", clean_phone));
                }
            }
        }

        // B. Telemetría de Entrega y Lectura
        "whatsapp.message.sent" | "whatsapp.message.delivered" | "whatsapp.message.read" | "whatsapp.message.failed" => {
            let msg_obj = payload.get("message");
            let wamid = msg_obj
                .and_then(|m| m.get("wamid").or_else(|| m.get("whatsapp_message_id")).or_else(|| m.get("id")))
                .and_then(|i| i.as_str())
                .unwrap_or("");
            let msg_internal_id = msg_obj
                .and_then(|m| m.get("id"))
                .and_then(|i| i.as_str())
                .unwrap_or(wamid);
            let recipient = msg_obj
                .and_then(|m| m.get("recipient").or_else(|| m.get("to")))
                .and_then(|r| r.as_str())
                .or_else(|| payload.get("conversation").and_then(|c| c.get("phone_number")).and_then(|p| p.as_str()))
                .or_else(|| payload.get("contact").and_then(|c| c.get("wa_id").or_else(|| c.get("phone_number"))).and_then(|p| p.as_str()))
                .unwrap_or("");
            dest_val = if !recipient.is_empty() { recipient.to_string() } else { wamid.to_string() };

            let status_name = match event_type {
                "whatsapp.message.sent" => "sent",
                "whatsapp.message.delivered" => "delivered",
                "whatsapp.message.read" => "read",
                "whatsapp.message.failed" => "failed",
                _ => "pending",
            };

            let err_obj = payload
                .get("error")
                .or_else(|| msg_obj.and_then(|m| m.get("error")))
                .or_else(|| msg_obj.and_then(|m| m.get("errors")).and_then(|e| e.as_array()).and_then(|a| a.first()));
            let err_code = err_obj.and_then(|e| e.get("code")).map(|c| c.to_string());
            let err_msg = err_obj
                .and_then(|e| e.get("message").or_else(|| e.get("title")).or_else(|| e.get("details")))
                .and_then(|m| m.as_str())
                .map(|s| s.to_string());
            err_msg_log = err_msg.clone();

            if !wamid.is_empty() || !msg_internal_id.is_empty() {
                let _ = sqlx::query(
                    "UPDATE whatsapp_broadcast_recipients
                     SET status = $1,
                         sent_at = CASE WHEN $1 = 'sent' AND sent_at IS NULL THEN now() ELSE sent_at END,
                         delivered_at = CASE WHEN $1 = 'delivered' AND delivered_at IS NULL THEN now() ELSE delivered_at END,
                         read_at = CASE WHEN $1 = 'read' AND read_at IS NULL THEN now() ELSE read_at END,
                         error_code = COALESCE($2, error_code),
                         error_message = COALESCE($3, error_message),
                         updated_at = now()
                     WHERE wamid = $4 OR wamid = $5"
                )
                .bind(status_name)
                .bind(err_code.clone())
                .bind(err_msg.clone())
                .bind(wamid)
                .bind(msg_internal_id)
                .execute(&pool)
                .await;

                let estado_cola = if status_name == "failed" { "error" } else { "confirmado" };
                let _ = sqlx::query(
                    "UPDATE mensajes_pendientes
                     SET meta_status = $1,
                         estado = $2,
                         error_mensaje = COALESCE($3, error_mensaje)
                     WHERE kapso_message_id = $4 OR kapso_message_id = $5"
                )
                .bind(status_name)
                .bind(estado_cola)
                .bind(err_msg.clone())
                .bind(wamid)
                .bind(msg_internal_id)
                .execute(&pool)
                .await;

                resultado_portal["detalle"] = json!(format!("Actualizado estado a '{}' para mensaje {} (destinatario: {})", status_name, wamid, dest_val));
                if let Some(ref c) = err_code {
                    resultado_portal["codigo_error"] = json!(c);
                }
                if let Some(ref m) = err_msg {
                    resultado_portal["mensaje_error"] = json!(m);
                }
            }
        }

        _ => {
            resultado_portal["detalle"] = json!("Evento recibido y archivado sin acciones adicionales requeridas");
        }
    }

    let raw_str = String::from_utf8_lossy(&raw_body);
    crate::services::kapso::log_whatsapp_api(
        &pool,
        &format!("webhook:{}", event_type),
        &dest_val,
        if event_type.ends_with(".failed") { "error" } else { "ok" },
        Some(200),
        Some(0),
        &raw_str,
        &resultado_portal.to_string(),
        err_msg_log.as_deref(),
    ).await;

    (StatusCode::OK, Json(json!({"received": true})))
}

// ============================================================================
// Funciones Helper de Broadcast y Semanas
// ============================================================================

async fn execute_daily_broadcast(
    pool: &DbPool,
    _config: &Arc<Config>,
    target_date: NaiveDate,
    send_immediately: bool,
    test_phone: Option<String>,
) -> Result<serde_json::Value, String> {
    let kapso_cfg = get_whatsapp_config(pool).await;
    if kapso_cfg.api_key.trim().is_empty() || kapso_cfg.phone_number_id.trim().is_empty() {
        return Err("Credenciales de Kapso no configuradas en el sistema".to_string());
    }

    let client = KapsoClient::new(kapso_cfg.api_key.clone(), kapso_cfg.phone_number_id.clone());

    // Obtener template id o name configurado (prioriza plantilla de Canal WhatsApp)
    let template_id = if let Ok(env_tpl) = std::env::var("KAPSO_TEMPLATE_ID") {
        env_tpl
    } else if let Ok(env_tpl) = std::env::var("WHATSAPP_BRIEF_TEMPLATE") {
        env_tpl
    } else if !kapso_cfg.template_name.trim().is_empty() {
        kapso_cfg.template_name.clone()
    } else {
        "servicio_monitoreo".to_string()
    };

    // 1. Obtener la edición
    let edition_id: Option<Uuid> = sqlx::query_scalar("SELECT id FROM news_editions WHERE edition_date = $1")
        .bind(target_date)
        .fetch_optional(pool)
        .await
        .unwrap_or(None);

    let broadcast_name = format!("brief-{}", target_date);

    // Idempotencia: Verificar si ya existe para evitar duplicar
    let existing: Option<WhatsappBroadcastRecord> = sqlx::query_as::<_, WhatsappBroadcastRecord>(
        "SELECT id, edition_id, broadcast_id, phone_number_id, template_id, edition_date,
                status, scheduled_at, total_recipients, sent_count, delivered_count,
                read_count, failed_count, suppressed_count, responded_count, created_at, updated_at
         FROM whatsapp_broadcasts
         WHERE edition_date = $1 AND phone_number_id = $2 AND template_id = $3"
    )
    .bind(target_date)
    .bind(&kapso_cfg.phone_number_id)
    .bind(&template_id)
    .fetch_optional(pool)
    .await
    .unwrap_or(None);

    if let Some(prev) = existing {
        if prev.status == "completed" || prev.status == "sending" || prev.status == "scheduled" {
            info!("ℹ️ Broadcast para {} ya existe en estado '{}'. Retornando registro existente.", target_date, prev.status);
            return Ok(json!({
                "mensaje": format!("Broadcast ya programado o enviado previamente ({})", prev.status),
                "broadcast_id": prev.broadcast_id,
                "status": prev.status
            }));
        }
    }

    // 2. Sincronizar automáticamente cualquier contacto activo de lista_distribucion hacia subscribers
    let _ = sqlx::query(
        "INSERT INTO subscribers (phone_number, display_name, consent_status, consent_source, consent_text_version, consented_at, created_at, updated_at)
         SELECT 
            CASE 
                WHEN telefono LIKE '+%' THEN telefono
                WHEN telefono LIKE '52%' THEN '+' || telefono
                ELSE '+52' || regexp_replace(telefono, '\\D', '', 'g')
            END AS phone_number,
            nombre AS display_name,
            'subscribed' AS consent_status,
            'sync_lista_distribucion' AS consent_source,
            'whatsapp_news_v1' AS consent_text_version,
            now() AS consented_at,
            now() AS created_at,
            now() AS updated_at
         FROM lista_distribucion
         WHERE activo = true AND length(regexp_replace(telefono, '\\D', '', 'g')) >= 10
         ON CONFLICT (phone_number) DO UPDATE
         SET display_name = EXCLUDED.display_name,
             consent_status = 'subscribed',
             updated_at = now()"
    )
    .execute(pool)
    .await;

    // 3. Obtener destinatarios (suscriptores activos)
    let recipients_data: Vec<(Uuid, String)> = if let Some(ref t_phone) = test_phone {
        let clean = KapsoClient::sanitize_e164(t_phone);
        vec![(Uuid::new_v4(), clean)]
    } else {
        sqlx::query_as::<_, (Uuid, String)>(
            "SELECT id, phone_number FROM subscribers WHERE consent_status = 'subscribed' ORDER BY created_at ASC"
        )
        .fetch_all(pool)
        .await
        .unwrap_or_default()
    };

    if recipients_data.is_empty() {
        return Err("No hay suscriptores con consentimiento activo ('subscribed') para enviar la síntesis".to_string());
    }

    // 4. Determinar si template_id es un UUID de Kapso Platform (Broadcast) o nombre de plantilla Meta
    let is_uuid = Uuid::parse_str(&template_id).is_ok();
    let mut broadcast_via_kapso = false;
    let mut kapso_b_id = String::new();

    // Si no es UUID, intentar resolver su UUID interno en Kapso Platform mediante la API
    let target_uuid = if is_uuid {
        Some(template_id.clone())
    } else {
        client.find_template_uuid_by_name(&template_id).await
    };

    if let Some(ref t_uuid) = target_uuid {
        info!("Iniciando creación de Broadcast '{}' en Kapso Platform con UUID: {}", broadcast_name, t_uuid);
        match client.create_broadcast(&broadcast_name, t_uuid).await {
            Ok(create_resp) => {
                if let Some(id) = create_resp.get("data").and_then(|d| d.get("id")).and_then(|i| i.as_str())
                    .or_else(|| create_resp.get("id").and_then(|i| i.as_str())) {
                    kapso_b_id = id.to_string();
                    broadcast_via_kapso = true;
                    info!("✅ Broadcast creado en Kapso con éxito. Broadcast ID: {}", kapso_b_id);
                }
            }
            Err(e) => {
                warn!("Aviso: No se pudo crear broadcast en Kapso ({}). Alternando automáticamente a Opción 1: Envío directo de plantilla...", e);
            }
        }
    } else {
        info!("'{}' se procesará vía Opción 1: Envío directo de plantilla aprobada Meta Cloud API a cada suscriptor.", template_id);
    }

    // 5. Registrar registro maestro en base de datos local
    let local_b_id_str = if kapso_b_id.is_empty() { format!("direct-{}", Uuid::new_v4()) } else { kapso_b_id.clone() };
    let local_broadcast_id: Uuid = sqlx::query_scalar(
        "INSERT INTO whatsapp_broadcasts (
            edition_id, broadcast_id, phone_number_id, template_id, edition_date,
            status, total_recipients, created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, 'draft', $6, now(), now())
        ON CONFLICT (edition_date, phone_number_id, template_id) DO UPDATE
        SET broadcast_id = EXCLUDED.broadcast_id,
            status = 'draft',
            total_recipients = EXCLUDED.total_recipients,
            updated_at = now()
        RETURNING id"
    )
    .bind(edition_id)
    .bind(&local_b_id_str)
    .bind(&kapso_cfg.phone_number_id)
    .bind(&template_id)
    .bind(target_date)
    .bind(recipients_data.len() as i32)
    .fetch_one(pool)
    .await
    .map_err(|e| format!("Error guardando broadcast local: {}", e))?;

    // 6. Ejecución del Despacho
    if broadcast_via_kapso {
        // Opción 2: Cargar destinatarios en lotes de 1,000 en Kapso Platform
        for chunk in recipients_data.chunks(1000) {
            let mut kapso_recipients = Vec::new();
            for (sub_id, phone) in chunk {
                kapso_recipients.push(json!({
                    "phone_number": phone
                }));

                let _ = sqlx::query(
                    "INSERT INTO whatsapp_broadcast_recipients (broadcast_id, subscriber_id, phone_number, status, created_at, updated_at)
                     VALUES ($1, $2, $3, 'pending', now(), now())"
                )
                .bind(local_broadcast_id)
                .bind(sub_id)
                .bind(phone)
                .execute(pool)
                .await;
            }

            client.add_broadcast_recipients(&kapso_b_id, &kapso_recipients).await?;
        }

        let final_status: String;
        if send_immediately {
            info!("Enviando broadcast {} inmediatamente...", kapso_b_id);
            client.send_broadcast(&kapso_b_id).await?;
            final_status = "sending".to_string();
        } else {
            let scheduled_utc = format!("{}T13:00:00Z", target_date);
            info!("Programando broadcast {} para {} UTC...", kapso_b_id, scheduled_utc);
            client.schedule_broadcast(&kapso_b_id, &scheduled_utc).await?;
            final_status = "scheduled".to_string();
        }

        let _ = sqlx::query("UPDATE whatsapp_broadcasts SET status = $1, updated_at = now() WHERE id = $2")
            .bind(&final_status)
            .bind(local_broadcast_id)
            .execute(pool)
            .await;

        Ok(json!({
            "ok": true,
            "tipo_envio": "broadcast_kapso",
            "broadcast_id": kapso_b_id,
            "status": final_status,
            "total_destinatarios": recipients_data.len(),
            "edition_date": target_date
        }))
    } else {
        // Opción 1: Envío directo de plantilla aprobada Meta Cloud API a la audiencia
        let extracto_txt: Option<String> = sqlx::query_scalar("SELECT texto FROM extractos WHERE fecha = $1")
            .bind(target_date)
            .fetch_optional(pool)
            .await
            .unwrap_or(None);

        let extracto_final = if let Some(e) = extracto_txt.filter(|s| !s.trim().is_empty()) {
            e
        } else {
            let s_txt: String = sqlx::query_scalar("SELECT texto FROM sintesis_diarias WHERE fecha = $1")
                .bind(target_date)
                .fetch_optional(pool)
                .await
                .unwrap_or(None)
                .unwrap_or_default();
            s_txt.chars().take(900).collect()
        };

        let mut sent_count = 0;
        let mut failed_count = 0;

        for (sub_id, phone) in &recipients_data {
            let res = client.send_brief_template(
                Some(pool),
                phone,
                target_date,
                &extracto_final,
                "despacho_cms_plantilla_directa",
            ).await;

            match res {
                Ok(wamid) => {
                    sent_count += 1;
                    let _ = sqlx::query(
                        "INSERT INTO whatsapp_broadcast_recipients (broadcast_id, subscriber_id, phone_number, status, wamid, created_at, updated_at)
                         VALUES ($1, $2, $3, 'sent', $4, now(), now())"
                    )
                    .bind(local_broadcast_id)
                    .bind(sub_id)
                    .bind(phone)
                    .bind(&wamid)
                    .execute(pool)
                    .await;
                }
                Err(err_msg) => {
                    failed_count += 1;
                    let _ = sqlx::query(
                        "INSERT INTO whatsapp_broadcast_recipients (broadcast_id, subscriber_id, phone_number, status, error_message, created_at, updated_at)
                         VALUES ($1, $2, $3, 'failed', $4, now(), now())"
                    )
                    .bind(local_broadcast_id)
                    .bind(sub_id)
                    .bind(phone)
                    .bind(&err_msg)
                    .execute(pool)
                    .await;
                }
            }
        }

        let _ = sqlx::query("UPDATE whatsapp_broadcasts SET status = 'completed', sent_count = $1, failed_count = $2, updated_at = now() WHERE id = $3")
            .bind(sent_count)
            .bind(failed_count)
            .bind(local_broadcast_id)
            .execute(pool)
            .await;

        Ok(json!({
            "ok": true,
            "tipo_envio": "plantilla_directa",
            "mensaje": format!("Síntesis despachada con la plantilla oficial '{}' a {} suscriptores ({} enviados, {} con error)", kapso_cfg.template_name, recipients_data.len(), sent_count, failed_count),
            "destinatarios_notificados": sent_count,
            "fallidos": failed_count,
            "broadcast_id": local_broadcast_id
        }))
    }
}

fn get_week_range_for_date(date: NaiveDate) -> (NaiveDate, NaiveDate) {
    let weekday = date.weekday().num_days_from_monday(); // 0 = Monday, 6 = Sunday
    let start_date = date - Duration::days(weekday as i64);
    let end_date = start_date + Duration::days(6);
    (start_date, end_date)
}

fn format_week_label(start_date: NaiveDate, end_date: NaiveDate) -> String {
    let months = [
        "", "Enero", "Febrero", "Marzo", "Abril", "Mayo", "Junio",
        "Julio", "Agosto", "Septiembre", "Octubre", "Noviembre", "Diciembre"
    ];
    let m_start = months.get(start_date.month() as usize).unwrap_or(&"");
    let m_end = months.get(end_date.month() as usize).unwrap_or(&"");

    if start_date.month() == end_date.month() {
        format!("Semana del {} al {} de {} {}", start_date.day(), end_date.day(), m_end, end_date.year())
    } else {
        format!("Semana del {} de {} al {} de {} {}", start_date.day(), m_start, end_date.day(), m_end, end_date.year())
    }
}
