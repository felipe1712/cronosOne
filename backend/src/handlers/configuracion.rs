use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::FromRow;
use std::sync::Arc;

use crate::{config::Config, db::DbPool};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ConfiguracionRow {
    pub clave: String,
    pub valor: String,
    pub descripcion: Option<String>,
    pub categoria: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AvailableModel {
    pub id: String,
    pub name: String,
    pub description: String,
    pub badge: Option<String>,
    pub speed: String,
    pub intelligence: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateConfiguracionPayload {
    pub claude_model: String,
}

#[derive(Debug, Deserialize)]
pub struct TestClaudePayload {
    pub model: Option<String>,
}

pub fn get_available_claude_models() -> Vec<AvailableModel> {
    vec![
        AvailableModel {
            id: "claude-3-7-sonnet-20250219".to_string(),
            name: "Claude 3.7 Sonnet".to_string(),
            description: "Modelo híbrido de vanguardia con capacidad de razonamiento profundo y síntesis de alta precisión.".to_string(),
            badge: Some("Último Lanzamiento".to_string()),
            speed: "Rápido".to_string(),
            intelligence: "Máxima (Híbrida)".to_string(),
        },
        AvailableModel {
            id: "claude-3-5-sonnet-20241022".to_string(),
            name: "Claude 3.5 Sonnet (v2)".to_string(),
            description: "Equilibrio ideal para análisis de riesgos, síntesis de boletines ejecutivos y comprensión legal.".to_string(),
            badge: Some("Recomendado".to_string()),
            speed: "Rápido".to_string(),
            intelligence: "Muy Alta".to_string(),
        },
        AvailableModel {
            id: "claude-3-5-sonnet-20240620".to_string(),
            name: "Claude 3.5 Sonnet (v1)".to_string(),
            description: "Primera versión de Claude 3.5 Sonnet, alta compatibilidad con cuentas existentes.".to_string(),
            badge: None,
            speed: "Rápido".to_string(),
            intelligence: "Muy Alta".to_string(),
        },
        AvailableModel {
            id: "claude-3-5-sonnet-latest".to_string(),
            name: "Claude 3.5 Sonnet (Latest)".to_string(),
            description: "Alias oficial que siempre apunta a la versión más actualizada de Claude 3.5 Sonnet.".to_string(),
            badge: None,
            speed: "Rápido".to_string(),
            intelligence: "Muy Alta".to_string(),
        },
        AvailableModel {
            id: "claude-3-5-haiku-20241022".to_string(),
            name: "Claude 3.5 Haiku".to_string(),
            description: "Velocidad extrema y bajo costo. Ideal para generación instantánea de briefs ejecutivos cortos.".to_string(),
            badge: Some("Ultra Rápido".to_string()),
            speed: "Instantáneo".to_string(),
            intelligence: "Alta".to_string(),
        },
        AvailableModel {
            id: "claude-3-haiku-20240307".to_string(),
            name: "Claude 3 Haiku".to_string(),
            description: "Versión compacta clásica con máxima disponibilidad global en todas las cuentas de Anthropic.".to_string(),
            badge: Some("Económico".to_string()),
            speed: "Instantáneo".to_string(),
            intelligence: "Media-Alta".to_string(),
        },
        AvailableModel {
            id: "claude-3-opus-20240229".to_string(),
            name: "Claude 3 Opus".to_string(),
            description: "Modelo para tareas altamente complejas y análisis exhaustivo de documentos extensos.".to_string(),
            badge: None,
            speed: "Moderado".to_string(),
            intelligence: "Extrema".to_string(),
        },
    ]
}

pub async fn get_configuraciones(
    State((pool, _config)): State<(DbPool, Arc<Config>)>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // Asegurar tabla si no existe
    let _ = sqlx::query(
        "CREATE TABLE IF NOT EXISTS configuraciones_sistema (
            clave VARCHAR(100) PRIMARY KEY,
            valor TEXT NOT NULL,
            descripcion TEXT,
            categoria VARCHAR(50) DEFAULT 'ia',
            actualizado_en TIMESTAMPTZ NOT NULL DEFAULT now()
        )",
    )
    .execute(&pool)
    .await;

    let rows = sqlx::query_as::<_, ConfiguracionRow>(
        "SELECT clave, valor, descripcion, categoria FROM configuraciones_sistema",
    )
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    let mut current_model = "claude-3-5-sonnet-20241022".to_string();
    for r in &rows {
        if r.clave == "CLAUDE_MODEL" {
            current_model = r.valor.clone();
        }
    }

    let available_models = get_available_claude_models();

    Ok((
        StatusCode::OK,
        Json(json!({
            "claude_model": current_model,
            "available_models": available_models,
            "todas": rows
        })),
    ))
}

pub async fn update_configuracion(
    State((pool, _config)): State<(DbPool, Arc<Config>)>,
    Json(payload): Json<UpdateConfiguracionPayload>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let model = payload.claude_model.trim();
    if model.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "El nombre del modelo no puede estar vacío"})),
        ));
    }

    sqlx::query(
        "INSERT INTO configuraciones_sistema (clave, valor, descripcion, categoria, actualizado_en)
         VALUES ('CLAUDE_MODEL', $1, 'Modelo de Anthropic Claude seleccionado para la síntesis de boletines', 'ia', now())
         ON CONFLICT (clave) DO UPDATE SET valor = EXCLUDED.valor, actualizado_en = now()"
    )
    .bind(model)
    .execute(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Error guardando configuración en base de datos: {}", e)})),
        )
    })?;

    Ok((
        StatusCode::OK,
        Json(json!({
            "mensaje": "Configuración actualizada correctamente",
            "claude_model": model
        })),
    ))
}

pub async fn test_claude(
    State((_pool, config)): State<(DbPool, Arc<Config>)>,
    Json(payload): Json<TestClaudePayload>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let worker_url = format!("{}/api/test-claude", config.worker_base_url);
    let client = reqwest::Client::new();

    let res = client
        .post(&worker_url)
        .json(&payload)
        .send()
        .await
        .map_err(|e| {
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({
                    "ok": false,
                    "error": format!("No se pudo contactar al worker en {}: {}", worker_url, e)
                })),
            )
        })?;

    let body: serde_json::Value = res.json().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "ok": false,
                "error": format!("Error leyendo respuesta del worker: {}", e)
            })),
        )
    })?;

    Ok((StatusCode::OK, Json(body)))
}
