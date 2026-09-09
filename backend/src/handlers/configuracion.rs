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

pub const DEFAULT_SYSTEM_PROMPT: &str = r#"Eres el analista jefe de inteligencia estratégica de ExposureIQ, al servicio del Director de Operaciones de una de las aseguradoras más grandes de México.

Tu misión es leer los extractos temáticos del boletín diario de Coparmex (40+ páginas) y generar un "Briefing Ejecutivo Matutino" diseñado para ser leído directamente en WhatsApp en menos de 2 minutos.

Criterios de filtrado y priorización:
1. RELEVANTE PARA ASEGURADORA:
   - Impacto en Siniestralidad y Riesgos (robo a transporte, inseguridad en carreteras, desastres naturales, salud).
   - Marco Regulatorio y Jurídico (reformas legales, CNSF, SHCP, Condusef, reformas laborales/pensiones).
   - Variables Macroeconómicas (inflación médica/general, tasas de interés, tipo de cambio).
2. DESCARTE DE RUIDO:
   - Declaraciones puramente políticas, eventos de relaciones públicas o discursos genéricos sin impacto operativo.

Reglas estrictas de formato para WhatsApp:
- Usa negritas con un solo asterisco: *Título*
- Usa viñetas claras con guiones: - Punto clave
- No uses encabezados Markdown tipo # o ##
- Incluye 3 o 4 puntos clave máximo, cada uno con su impacto operativo para la aseguradora.
- Termina con un bloque breve de "Acción / Atención sugerida".
- Longitud total: Entre 180 y 300 palabras. Debe verse limpio y ejecutivo."#;

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateConfiguracionPayload {
    pub claude_model: Option<String>,
    pub system_prompt: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TestClaudePayload {
    pub model: Option<String>,
}

pub fn get_available_claude_models() -> Vec<AvailableModel> {
    vec![
        AvailableModel {
            id: "claude-sonnet-4-5-20250929".to_string(),
            name: "Claude Sonnet 4.5".to_string(),
            description: "Equilibrio ideal para análisis de riesgos, síntesis de boletines ejecutivos y detección regulatoria.".to_string(),
            badge: Some("Recomendado".to_string()),
            speed: "Rápido".to_string(),
            intelligence: "Muy Alta".to_string(),
        },
        AvailableModel {
            id: "claude-haiku-4-5-20251001".to_string(),
            name: "Claude Haiku 4.5".to_string(),
            description: "Máxima velocidad y costo mínimo para generación instantánea de briefs ejecutivos de WhatsApp.".to_string(),
            badge: Some("Ultra Rápido".to_string()),
            speed: "Instantáneo".to_string(),
            intelligence: "Alta".to_string(),
        },
        AvailableModel {
            id: "claude-sonnet-4-6".to_string(),
            name: "Claude Sonnet 4.6".to_string(),
            description: "Capacidad analítica avanzada para síntesis estratégica y reportes directivos.".to_string(),
            badge: Some("Avanzado".to_string()),
            speed: "Rápido".to_string(),
            intelligence: "Muy Alta".to_string(),
        },
        AvailableModel {
            id: "claude-sonnet-5".to_string(),
            name: "Claude Sonnet 5".to_string(),
            description: "Generación Sonnet 5 para análisis exhaustivo de riesgos y extracción de patrones operativos.".to_string(),
            badge: Some("Nueva Generación".to_string()),
            speed: "Rápido".to_string(),
            intelligence: "Máxima".to_string(),
        },
        AvailableModel {
            id: "claude-opus-4-5-20251101".to_string(),
            name: "Claude Opus 4.5".to_string(),
            description: "Razonamiento profundo para documentos de alta complejidad jurídica y técnica.".to_string(),
            badge: None,
            speed: "Moderado".to_string(),
            intelligence: "Extrema".to_string(),
        },
        AvailableModel {
            id: "claude-opus-4-6".to_string(),
            name: "Claude Opus 4.6".to_string(),
            description: "Motor Opus optimizado para correlación de riesgos corporativos.".to_string(),
            badge: None,
            speed: "Moderado".to_string(),
            intelligence: "Extrema".to_string(),
        },
        AvailableModel {
            id: "claude-opus-4-7".to_string(),
            name: "Claude Opus 4.7".to_string(),
            description: "Opus 4.7 para análisis multinivel y síntesis ejecutiva sin pérdida de contexto.".to_string(),
            badge: None,
            speed: "Moderado".to_string(),
            intelligence: "Extrema".to_string(),
        },
        AvailableModel {
            id: "claude-opus-4-8".to_string(),
            name: "Claude Opus 4.8".to_string(),
            description: "Máxima precisión en inferencia analítica y redacción directiva.".to_string(),
            badge: None,
            speed: "Moderado".to_string(),
            intelligence: "Extrema".to_string(),
        },
        AvailableModel {
            id: "claude-opus-5".to_string(),
            name: "Claude Opus 5".to_string(),
            description: "El modelo más potente de Anthropic para análisis integral de inteligencia de riesgos.".to_string(),
            badge: Some("Máxima Potencia".to_string()),
            speed: "Moderado".to_string(),
            intelligence: "Cúspide".to_string(),
        },
        AvailableModel {
            id: "claude-fable-5".to_string(),
            name: "Claude Fable 5".to_string(),
            description: "Arquitectura Fable para síntesis narrativa ejecutiva y estilizada.".to_string(),
            badge: None,
            speed: "Rápido".to_string(),
            intelligence: "Alta".to_string(),
        },
        AvailableModel {
            id: "claude-fable-5-1".to_string(),
            name: "Claude Fable 5.1".to_string(),
            description: "Versión refinada de Fable 5 para síntesis ejecutiva clara y concisa.".to_string(),
            badge: None,
            speed: "Rápido".to_string(),
            intelligence: "Alta".to_string(),
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

    let mut current_model = "claude-sonnet-4-5-20250929".to_string();
    let mut current_prompt = DEFAULT_SYSTEM_PROMPT.to_string();
    for r in &rows {
        if r.clave == "CLAUDE_MODEL" {
            current_model = r.valor.clone();
        } else if r.clave == "CLAUDE_SYSTEM_PROMPT" {
            current_prompt = r.valor.clone();
        }
    }

    let available_models = get_available_claude_models();

    Ok((
        StatusCode::OK,
        Json(json!({
            "claude_model": current_model,
            "system_prompt": current_prompt,
            "default_system_prompt": DEFAULT_SYSTEM_PROMPT,
            "available_models": available_models,
            "todas": rows
        })),
    ))
}

pub async fn update_configuracion(
    State((pool, _config)): State<(DbPool, Arc<Config>)>,
    Json(payload): Json<UpdateConfiguracionPayload>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    if let Some(ref model) = payload.claude_model {
        let m = model.trim();
        if !m.is_empty() {
            sqlx::query(
                "INSERT INTO configuraciones_sistema (clave, valor, descripcion, categoria, actualizado_en)
                 VALUES ('CLAUDE_MODEL', $1, 'Modelo de Anthropic Claude seleccionado para la síntesis de boletines', 'ia', now())
                 ON CONFLICT (clave) DO UPDATE SET valor = EXCLUDED.valor, actualizado_en = now()"
            )
            .bind(m)
            .execute(&pool)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": format!("Error guardando modelo: {}", e)})),
                )
            })?;
        }
    }

    if let Some(ref prompt) = payload.system_prompt {
        let p = prompt.trim();
        if !p.is_empty() {
            sqlx::query(
                "INSERT INTO configuraciones_sistema (clave, valor, descripcion, categoria, actualizado_en)
                 VALUES ('CLAUDE_SYSTEM_PROMPT', $1, 'Instrucciones del sistema para el análisis y síntesis de boletines', 'ia', now())
                 ON CONFLICT (clave) DO UPDATE SET valor = EXCLUDED.valor, actualizado_en = now()"
            )
            .bind(p)
            .execute(&pool)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": format!("Error guardando prompt: {}", e)})),
                )
            })?;
        }
    }

    Ok((
        StatusCode::OK,
        Json(json!({
            "mensaje": "Configuración guardada correctamente"
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
