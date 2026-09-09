use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use serde_json::json;
use std::sync::Arc;
use tracing::{error, info};

use crate::{
    config::Config,
    db::DbPool,
    models::{WebhookWhatsappSessionPayload, WhatsappSesion},
};

/// Obtiene el estado actual de la sesión de WhatsApp desde la base de datos (alimentado por n8n)
pub async fn get_waha_status(
    State((pool, config)): State<(DbPool, Arc<Config>)>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let sesion = sqlx::query_as::<_, WhatsappSesion>(
        "SELECT id, estado, qr_code, detalles, actualizado_en 
         FROM whatsapp_sesion 
         WHERE id = $1",
    )
    .bind(&config.waha_session)
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Error consultando sesión de WhatsApp: {}", e)})),
        )
    })?;

    match sesion {
        Some(s) => Ok((
            StatusCode::OK,
            Json(json!({
                "name": s.id,
                "status": s.estado,
                "detalles": s.detalles,
                "actualizado_en": s.actualizado_en,
                "gestionado_por": "n8n"
            })),
        )),
        None => Ok((
            StatusCode::OK,
            Json(json!({
                "name": config.waha_session,
                "status": "DISCONNECTED",
                "message": "Sin registro de sesión en base de datos. Esperando webhook de n8n."
            })),
        )),
    }
}

/// Obtiene el código QR actual de WhatsApp almacenado por el webhook de n8n
pub async fn get_waha_qr(
    State((pool, config)): State<(DbPool, Arc<Config>)>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let sesion = sqlx::query_as::<_, WhatsappSesion>(
        "SELECT id, estado, qr_code, detalles, actualizado_en 
         FROM whatsapp_sesion 
         WHERE id = $1",
    )
    .bind(&config.waha_session)
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Error consultando código QR: {}", e)})),
        )
    })?;

    match sesion {
        Some(s) if s.qr_code.is_some() => Ok((
            StatusCode::OK,
            Json(json!({
                "qr": s.qr_code,
                "status": s.estado,
                "actualizado_en": s.actualizado_en
            })),
        )),
        Some(s) => Ok((
            StatusCode::OK,
            Json(json!({
                "qr": null,
                "status": s.estado,
                "message": if s.estado == "CONNECTED" || s.estado == "WORKING" {
                    "La sesión de WhatsApp ya está conectada y activa."
                } else {
                    "No hay código QR pendiente en este momento."
                }
            })),
        )),
        None => Ok((
            StatusCode::OK,
            Json(json!({
                "qr": null,
                "message": "Esperando inicialización desde el flujo de n8n."
            })),
        )),
    }
}

/// Solicita reinicio de sesión notificando a n8n
pub async fn restart_waha_session(
    State((pool, config)): State<(DbPool, Arc<Config>)>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let now = Utc::now();

    // Actualizar estado local a RESTART_REQUESTED
    let _ = sqlx::query(
        "UPDATE whatsapp_sesion 
         SET estado = 'RESTART_REQUESTED', actualizado_en = $1 
         WHERE id = $2",
    )
    .bind(now)
    .bind(&config.waha_session)
    .execute(&pool)
    .await;

    // Si hay un webhook de reinicio configurado hacia n8n, dispararlo
    if let Some(webhook_url) = &config.n8n_restart_webhook_url {
        let client = reqwest::Client::new();
        let payload = json!({
            "accion": "restart_session",
            "session": config.waha_session
        });
        let w_url = webhook_url.clone();
        tokio::spawn(async move {
            info!("Notificando a n8n solicitud de reinicio en {}", w_url);
            if let Err(e) = client.post(&w_url).json(&payload).send().await {
                error!("No se pudo notificar a n8n del reinicio: {}", e);
            }
        });
    }

    Ok((
        StatusCode::OK,
        Json(json!({
            "mensaje": "Solicitud de reconexión registrada. El flujo de n8n procesará la reactivación.",
            "status": "RESTART_REQUESTED"
        })),
    ))
}

/// Webhook consumido por n8n para actualizar el estado de la sesión y el código QR
/// Endpoint: POST /api/webhooks/whatsapp/session
pub async fn webhook_session_update(
    State((pool, _)): State<(DbPool, Arc<Config>)>,
    Json(payload): Json<WebhookWhatsappSessionPayload>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let session_id = payload.session.unwrap_or_else(|| "default".to_string());
    let now = Utc::now();

    info!(
        "Recibida actualización de sesión WhatsApp desde n8n: sesión={}, estado={}, tiene_qr={}",
        session_id,
        payload.status,
        payload.qr.is_some()
    );

    sqlx::query(
        "INSERT INTO whatsapp_sesion (id, estado, qr_code, detalles, actualizado_en)
         VALUES ($1, $2, $3, $4, $5)
         ON CONFLICT (id) DO UPDATE 
         SET estado = EXCLUDED.estado,
             qr_code = EXCLUDED.qr_code,
             detalles = EXCLUDED.detalles,
             actualizado_en = EXCLUDED.actualizado_en",
    )
    .bind(&session_id)
    .bind(&payload.status)
    .bind(&payload.qr)
    .bind(&payload.detalles)
    .bind(now)
    .execute(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Error guardando estado de sesión: {}", e)})),
        )
    })?;

    Ok((
        StatusCode::OK,
        Json(json!({
            "success": true,
            "mensaje": "Estado de sesión WhatsApp actualizado correctamente",
            "session": session_id,
            "status": payload.status
        })),
    ))
}
