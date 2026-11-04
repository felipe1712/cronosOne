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
            // Si aún no existe, crear la edición inicial en modo borrador
            sqlx::query_as::<_, NewsEdition>(
                "INSERT INTO news_editions (edition_date, status, generated_at, created_at, updated_at)
                 VALUES ($1, 'draft', now(), now(), now())
                 ON CONFLICT (edition_date) DO UPDATE SET updated_at = now()
                 RETURNING id, edition_date, status, generated_at, published_at, created_at, updated_at"
            )
            .bind(date)
            .fetch_one(&pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?
        }
    };

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

pub async fn webhook_kapso(
    State((pool, _)): State<(DbPool, Arc<Config>)>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let event_type = payload.get("type").and_then(|v| v.as_str()).unwrap_or("unknown");
    let event_id = payload.get("id").and_then(|v| v.as_str()).unwrap_or("");

    // Deduplicación e Idempotencia
    if !event_id.is_empty() {
        let exists: Option<bool> = sqlx::query_scalar("SELECT true FROM whatsapp_message_events WHERE event_id = $1")
            .bind(event_id)
            .fetch_optional(&pool)
            .await
            .unwrap_or(None);

        if exists.unwrap_or(false) {
            return (StatusCode::OK, Json(json!({"status": "duplicate_ignored"})));
        }

        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        payload.to_string().hash(&mut hasher);
        let p_hash = format!("{:x}", hasher.finish());
        let msg_id = payload.get("message").and_then(|m| m.get("id")).and_then(|i| i.as_str()).unwrap_or("");

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

    match event_type {
        // A. Gestión de Bajas y Consentimiento (STOP)
        "whatsapp.contact.marketing_preference_changed" => {
            let wa_id = payload.get("contact").and_then(|c| c.get("wa_id")).and_then(|w| w.as_str()).unwrap_or("");
            let pref_status = payload.get("marketing_preference").and_then(|m| m.get("status")).and_then(|s| s.as_str()).unwrap_or("");

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
                }
            }
        }

        // B. Telemetría de Entrega y Lectura
        "whatsapp.message.sent" | "whatsapp.message.delivered" | "whatsapp.message.read" | "whatsapp.message.failed" => {
            let wamid = payload.get("message").and_then(|m| m.get("id")).and_then(|i| i.as_str()).unwrap_or("");
            if !wamid.is_empty() {
                let status_name = match event_type {
                    "whatsapp.message.sent" => "sent",
                    "whatsapp.message.delivered" => "delivered",
                    "whatsapp.message.read" => "read",
                    "whatsapp.message.failed" => "failed",
                    _ => "pending",
                };

                let err_code = payload.get("error").and_then(|e| e.get("code")).map(|c| c.to_string());
                let err_msg = payload.get("error").and_then(|e| e.get("message")).and_then(|m| m.as_str()).map(|s| s.to_string());

                let _ = sqlx::query(
                    "UPDATE whatsapp_broadcast_recipients
                     SET status = $1,
                         sent_at = CASE WHEN $1 = 'sent' AND sent_at IS NULL THEN now() ELSE sent_at END,
                         delivered_at = CASE WHEN $1 = 'delivered' AND delivered_at IS NULL THEN now() ELSE delivered_at END,
                         read_at = CASE WHEN $1 = 'read' AND read_at IS NULL THEN now() ELSE read_at END,
                         error_code = COALESCE($2, error_code),
                         error_message = COALESCE($3, error_message),
                         updated_at = now()
                     WHERE wamid = $4"
                )
                .bind(status_name)
                .bind(err_code)
                .bind(err_msg)
                .bind(wamid)
                .execute(&pool)
                .await;
            }
        }

        _ => {}
    }

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

    // Obtener template id o name configurado
    let template_id = if let Ok(env_tpl) = std::env::var("KAPSO_TEMPLATE_ID") {
        env_tpl
    } else if let Ok(env_tpl) = std::env::var("WHATSAPP_BRIEF_TEMPLATE") {
        env_tpl
    } else if !kapso_cfg.template_name.trim().is_empty() && kapso_cfg.template_name != "servicio_monitoreo" {
        kapso_cfg.template_name.clone()
    } else {
        "brief_informativo_diario".to_string()
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

    // 2. Crear Broadcast en Kapso Platform API
    info!("Iniciando creación de Broadcast '{}' en Kapso Platform...", broadcast_name);
    let create_resp = client.create_broadcast(&broadcast_name, &template_id).await?;
    let kapso_b_id = create_resp.get("data").and_then(|d| d.get("id")).and_then(|i| i.as_str())
        .or_else(|| create_resp.get("id").and_then(|i| i.as_str()))
        .ok_or_else(|| "Respuesta de Kapso no incluye id de broadcast".to_string())?
        .to_string();

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
        return Err("No hay suscriptores con consentimiento activo ('subscribed') para enviar el broadcast".to_string());
    }

    // 4. Registrar en base de datos local
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
    .bind(&kapso_b_id)
    .bind(&kapso_cfg.phone_number_id)
    .bind(&template_id)
    .bind(target_date)
    .bind(recipients_data.len() as i32)
    .fetch_one(pool)
    .await
    .map_err(|e| format!("Error guardando broadcast local: {}", e))?;

    // 5. Cargar destinatarios en lotes de 1,000 en Kapso
    for chunk in recipients_data.chunks(1000) {
        let mut kapso_recipients = Vec::new();
        for (sub_id, phone) in chunk {
            kapso_recipients.push(json!({
                "phone_number": phone
            }));

            // Registrar en whatsapp_broadcast_recipients
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

    // 6. Enviar ahora o Programar
    let final_status: String;
    if send_immediately {
        info!("Enviando broadcast {} inmediatamente...", kapso_b_id);
        client.send_broadcast(&kapso_b_id).await?;
        final_status = "sending".to_string();
    } else {
        // Programar a las 07:00 AM hora México (UTC = 13:00 en horario estándar o 12:00 en verano)
        let scheduled_utc = format!("{}T13:00:00Z", target_date);
        info!("Programando broadcast {} para {} UTC...", kapso_b_id, scheduled_utc);
        client.schedule_broadcast(&kapso_b_id, &scheduled_utc).await?;
        final_status = "scheduled".to_string();
    }

    sqlx::query("UPDATE whatsapp_broadcasts SET status = $1, updated_at = now() WHERE id = $2")
        .bind(&final_status)
        .bind(local_broadcast_id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(json!({
        "ok": true,
        "broadcast_id": kapso_b_id,
        "status": final_status,
        "total_destinatarios": recipients_data.len()
    }))
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
