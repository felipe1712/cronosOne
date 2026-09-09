use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use std::sync::Arc;
use tracing::error;

use crate::{config::Config, db::DbPool};

/// Obtiene el estado de la sesión de WAHA (CONNECTED, SCAN_QR_CODE, STOPPED, etc.)
pub async fn get_waha_status(
    State((_, config)): State<(DbPool, Arc<Config>)>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let client = reqwest::Client::new();
    let url = format!(
        "{}/api/sessions/{}",
        config.waha_base_url, config.waha_session
    );

    let mut req = client.get(&url);
    if let Some(key) = &config.waha_api_key {
        req = req.header("X-Api-Key", key);
    }

    match req.send().await {
        Ok(resp) => {
            let status = resp.status();
            if status.is_success() {
                let data: serde_json::Value = resp.json().await.unwrap_or(json!({
                    "status": "UNKNOWN"
                }));
                Ok((StatusCode::OK, Json(data)))
            } else {
                Ok((
                    StatusCode::OK,
                    Json(json!({
                        "name": config.waha_session,
                        "status": "DISCONNECTED",
                        "error": format!("Respuesta WAHA: {}", status)
                    })),
                ))
            }
        }
        Err(e) => {
            error!("Error contactando WAHA en {}: {}", url, e);
            Ok((
                StatusCode::OK,
                Json(json!({
                    "name": config.waha_session,
                    "status": "UNREACHABLE",
                    "error": format!("No se pudo conectar con el servicio WAHA: {}", e)
                })),
            ))
        }
    }
}

/// Obtiene el código QR actual de emparejamiento de WhatsApp
pub async fn get_waha_qr(
    State((_, config)): State<(DbPool, Arc<Config>)>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let client = reqwest::Client::new();
    let url = format!(
        "{}/api/{}/auth/qr",
        config.waha_base_url, config.waha_session
    );

    let mut req = client.get(&url);
    if let Some(key) = &config.waha_api_key {
        req = req.header("X-Api-Key", key);
    }

    match req.send().await {
        Ok(resp) => {
            if resp.status().is_success() {
                let data: serde_json::Value = resp.json().await.unwrap_or(json!({}));
                Ok((StatusCode::OK, Json(data)))
            } else {
                Ok((
                    StatusCode::OK,
                    Json(json!({
                        "qr": null,
                        "message": "No hay código QR pendiente (la sesión podría estar ya conectada)"
                    })),
                ))
            }
        }
        Err(e) => {
            Ok((
                StatusCode::OK,
                Json(json!({
                    "qr": null,
                    "error": format!("Error consultando QR en WAHA: {}", e)
                })),
            ))
        }
    }
}

/// Dispara un reinicio de la sesión de WAHA para forzar reconexión
pub async fn restart_waha_session(
    State((_, config)): State<(DbPool, Arc<Config>)>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let client = reqwest::Client::new();
    let url = format!(
        "{}/api/sessions/{}/restart",
        config.waha_base_url, config.waha_session
    );

    let mut req = client.post(&url);
    if let Some(key) = &config.waha_api_key {
        req = req.header("X-Api-Key", key);
    }

    match req.send().await {
        Ok(resp) => {
            let data: serde_json::Value = resp.json().await.unwrap_or(json!({
                "mensaje": "Comando de reinicio enviado a WAHA"
            }));
            Ok((StatusCode::OK, Json(data)))
        }
        Err(e) => {
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": format!("Error reiniciando sesión de WAHA: {}", e)
                })),
            ))
        }
    }
}
