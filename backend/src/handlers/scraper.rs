use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use tracing::{error, info};

use crate::{config::Config, db::DbPool};

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct ScraperJobRequest {
    pub fecha: Option<String>,
    pub secciones: Option<Vec<String>>,
}

/// Dispara el scraper del Senado en el worker Python
pub async fn ejecutar_scraper_senado(
    State((_pool, config)): State<(DbPool, Arc<Config>)>,
    Json(payload): Json<Option<ScraperJobRequest>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let worker_url = format!("{}/api/scrape-senado", config.worker_base_url);
    let client = reqwest::Client::new();
    let body = payload.unwrap_or_default();

    info!("Iniciando scraper del Senado en worker: {}", worker_url);

    let resp = client
        .post(&worker_url)
        .json(&body)
        .send()
        .await
        .map_err(|e| {
            error!("Error llamando a worker Python: {}", e);
            (
                StatusCode::BAD_GATEWAY,
                Json(json!({"error": format!("No se pudo contactar al worker Python: {}", e)})),
            )
        })?;

    let status = resp.status();
    let res_json: serde_json::Value = resp.json().await.unwrap_or_else(|_| {
        json!({"mensaje": "Scraper del Senado disparado en segundo plano"})
    });

    if status.is_success() {
        Ok((StatusCode::OK, Json(res_json)))
    } else {
        Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Error reportado por el worker", "detalles": res_json})),
        ))
    }
}

/// Consulta el estado actual de la última ejecución del scraper
pub async fn get_scraper_senado_status(
    State((_pool, config)): State<(DbPool, Arc<Config>)>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let worker_url = format!("{}/api/scrape-senado/status", config.worker_base_url);
    let client = reqwest::Client::new();

    match client.get(&worker_url).send().await {
        Ok(resp) => {
            let res_json: serde_json::Value = resp.json().await.unwrap_or_else(|_| {
                json!({"estado": "desconocido", "mensaje": "Respuesta no JSON del worker"})
            });
            Ok((StatusCode::OK, Json(res_json)))
        }
        Err(e) => {
            Ok((
                StatusCode::OK,
                Json(json!({
                    "estado": "idle",
                    "mensaje": format!("Worker no disponible temporalmente: {}", e),
                    "en_progreso": false,
                    "archivos_procesados": 0
                })),
            ))
        }
    }
}
