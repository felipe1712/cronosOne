use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::FromRow;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    config::Config,
    db::DbPool,
    models::{
        CreateDestinatarioRequest, CreateGrupoRequest, Destinatario, DestinatarioRow,
        GrupoDistribucion, GrupoResumen, UpdateDestinatarioRequest, UpdateGrupoRequest,
    },
};

#[allow(dead_code)]
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
    pub claude_max_tokens: Option<i32>,
    pub system_prompt: Option<String>,
    pub whatsapp_provider: Option<String>,
    pub kapso_api_key: Option<String>,
    pub kapso_phone_number_id: Option<String>,
    pub director_whatsapp_phone: Option<String>,
    pub whatsapp_template_name: Option<String>,
    pub whatsapp_template_language: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TestClaudePayload {
    pub model: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TestWhatsappPayload {
    pub phone: Option<String>,
    pub message: Option<String>,
    pub api_key: Option<String>,
    pub phone_number_id: Option<String>,
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

async fn upsert_config(
    pool: &DbPool,
    clave: &str,
    valor: &str,
    descripcion: &str,
    categoria: &str,
) {
    let clean_val = valor.trim();
    // 1. UPSERT directo compatible con PostgreSQL
    let res = sqlx::query(
        "INSERT INTO configuraciones_sistema (clave, valor, descripcion, categoria, actualizado_en)
         VALUES ($1, $2, $3, $4, now())
         ON CONFLICT (clave) DO UPDATE SET 
            valor = EXCLUDED.valor,
            descripcion = COALESCE(EXCLUDED.descripcion, configuraciones_sistema.descripcion),
            categoria = COALESCE(EXCLUDED.categoria, configuraciones_sistema.categoria),
            actualizado_en = now()"
    )
    .bind(clave)
    .bind(clean_val)
    .bind(descripcion)
    .bind(categoria)
    .execute(pool)
    .await;

    if let Err(e) = res {
        tracing::warn!("Upsert con metadatos falló para {}: {}, intentando inserción básica clave/valor", clave, e);
        let fallback_res = sqlx::query(
            "INSERT INTO configuraciones_sistema (clave, valor)
             VALUES ($1, $2)
             ON CONFLICT (clave) DO UPDATE SET valor = EXCLUDED.valor"
        )
        .bind(clave)
        .bind(clean_val)
        .execute(pool)
        .await;

        if let Err(e2) = fallback_res {
            tracing::error!("Error crítico guardando configuración {}: {}", clave, e2);
        } else {
            tracing::info!("Configuración {} guardada exitosamente (fallback clave/valor)", clave);
        }
    } else {
        tracing::info!("Configuración {} guardada exitosamente en BD", clave);
    }
}

pub async fn get_configuraciones(
    State((pool, config)): State<(DbPool, Arc<Config>)>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // Asegurar tabla y columnas si no existen
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

    let _ = sqlx::query("ALTER TABLE configuraciones_sistema ADD COLUMN IF NOT EXISTS descripcion TEXT").execute(&pool).await;
    let _ = sqlx::query("ALTER TABLE configuraciones_sistema ADD COLUMN IF NOT EXISTS categoria VARCHAR(50) DEFAULT 'ia'").execute(&pool).await;
    let _ = sqlx::query("ALTER TABLE configuraciones_sistema ADD COLUMN IF NOT EXISTS actualizado_en TIMESTAMPTZ NOT NULL DEFAULT now()").execute(&pool).await;

    // Asegurar limpieza automática de cualquier valor dummy previo en base de datos
    let _ = sqlx::query(
        "UPDATE configuraciones_sistema SET valor = '' WHERE clave = 'DIRECTOR_WHATSAPP_PHONE' AND valor = '5215512345678'"
    )
    .execute(&pool)
    .await;
    let _ = sqlx::query(
        "DELETE FROM lista_distribucion WHERE telefono = '5215512345678'"
    )
    .execute(&pool)
    .await;
    let _ = sqlx::query(
        "DELETE FROM mensajes_pendientes WHERE destinatario = '5215512345678'"
    )
    .execute(&pool)
    .await;

    let mut current_model = "claude-sonnet-4-5-20250929".to_string();
    let mut current_max_tokens: i32 = 1000;
    let mut current_prompt = DEFAULT_SYSTEM_PROMPT.to_string();
    let mut whatsapp_provider = "kapso".to_string();
    let mut kapso_api_key = String::new();
    let mut kapso_phone_number_id = String::new();
    let mut director_whatsapp_phone = String::new();

    let mut config_map = serde_json::Map::new();

    // Consulta directa de clave/valor a prueba de esquemas parciales
    if let Ok(rows) = sqlx::query("SELECT clave, valor FROM configuraciones_sistema").fetch_all(&pool).await {
        for r in rows {
            use sqlx::Row;
            let c: String = r.try_get("clave").unwrap_or_default();
            let v: String = r.try_get("valor").unwrap_or_default();
            config_map.insert(c.clone(), json!(v));
            match c.as_str() {
                "CLAUDE_MODEL" => {
                    if !v.trim().is_empty() {
                        current_model = v.trim().to_string();
                    }
                }
                "CLAUDE_MAX_TOKENS" => {
                    if let Ok(num) = v.trim().parse::<i32>() {
                        if num >= 200 && num <= 8000 {
                            current_max_tokens = num;
                        }
                    }
                }
                "CLAUDE_SYSTEM_PROMPT" => {
                    if !v.trim().is_empty() {
                        current_prompt = v;
                    }
                }
                "WHATSAPP_PROVIDER" => whatsapp_provider = v,
                "KAPSO_API_KEY" => kapso_api_key = v,
                "KAPSO_PHONE_NUMBER_ID" => kapso_phone_number_id = v,
                "DIRECTOR_WHATSAPP_PHONE" => {
                    if v.trim() != "5215512345678" {
                        director_whatsapp_phone = v;
                    }
                }
                _ => {}
            }
        }
    }

    if !config_map.contains_key("CLAUDE_MODEL") || current_model.trim().is_empty() {
        upsert_config(&pool, "CLAUDE_MODEL", &current_model, "Modelo de Anthropic Claude seleccionado para la síntesis de boletines", "ia").await;
        config_map.insert("CLAUDE_MODEL".to_string(), json!(&current_model));
    }

    if !config_map.contains_key("CLAUDE_MAX_TOKENS") {
        upsert_config(&pool, "CLAUDE_MAX_TOKENS", &current_max_tokens.to_string(), "Límite máximo de tokens de salida por síntesis ejecutiva", "ia").await;
        config_map.insert("CLAUDE_MAX_TOKENS".to_string(), json!(&current_max_tokens.to_string()));
    }

    if !config_map.contains_key("CLAUDE_SYSTEM_PROMPT") || current_prompt.trim().is_empty() {
        current_prompt = DEFAULT_SYSTEM_PROMPT.to_string();
        upsert_config(&pool, "CLAUDE_SYSTEM_PROMPT", DEFAULT_SYSTEM_PROMPT, "Instrucciones del sistema para el análisis y síntesis de boletines", "ia").await;
        config_map.insert("CLAUDE_SYSTEM_PROMPT".to_string(), json!(DEFAULT_SYSTEM_PROMPT));
    }

    if !config_map.contains_key("WHATSAPP_PROVIDER") || whatsapp_provider.trim().is_empty() {
        upsert_config(&pool, "WHATSAPP_PROVIDER", "kapso", "Proveedor activo de WhatsApp: kapso o waha", "whatsapp").await;
        config_map.insert("WHATSAPP_PROVIDER".to_string(), json!("kapso"));
    }

    if director_whatsapp_phone.trim().is_empty() {
        if let Ok(env_phone) = std::env::var("DIRECTOR_WHATSAPP_PHONE") {
            if env_phone.trim() != "5215512345678" && !env_phone.trim().is_empty() {
                director_whatsapp_phone = env_phone;
            }
        } else if config.director_whatsapp.trim() != "5215512345678" && !config.director_whatsapp.trim().is_empty() {
            director_whatsapp_phone = config.director_whatsapp.trim().to_string();
        }
    }

    if director_whatsapp_phone.trim().is_empty() {
        if let Ok(Some(row)) = sqlx::query_as::<_, (String,)>(
            "SELECT telefono FROM lista_distribucion WHERE activo = true AND telefono != '5215512345678' ORDER BY creado_en ASC LIMIT 1"
        )
        .fetch_optional(&pool)
        .await {
            director_whatsapp_phone = row.0;
        }
    }

    if !director_whatsapp_phone.trim().is_empty() {
        // Persistir físicamente en PostgreSQL para que nunca quede vacía en configuraciones_sistema
        upsert_config(&pool, "DIRECTOR_WHATSAPP_PHONE", &director_whatsapp_phone, "Número de WhatsApp de destino del Director en formato E.164", "whatsapp").await;
        config_map.insert("DIRECTOR_WHATSAPP_PHONE".to_string(), json!(&director_whatsapp_phone));
    }

    if kapso_api_key.trim().is_empty() {
        if let Ok(env_key) = std::env::var("KAPSO_API_KEY") {
            kapso_api_key = env_key.trim().to_string();
        } else if !config.kapso_api_key.trim().is_empty() {
            kapso_api_key = config.kapso_api_key.trim().to_string();
        }
        if !kapso_api_key.is_empty() {
            upsert_config(&pool, "KAPSO_API_KEY", &kapso_api_key, "Clave de API del proyecto en Kapso (X-API-Key)", "whatsapp").await;
            config_map.insert("KAPSO_API_KEY".to_string(), json!(&kapso_api_key));
        }
    }

    if kapso_phone_number_id.trim().is_empty() {
        if let Ok(env_id) = std::env::var("KAPSO_PHONE_NUMBER_ID") {
            kapso_phone_number_id = env_id.trim().to_string();
        } else if !config.kapso_phone_number_id.trim().is_empty() {
            kapso_phone_number_id = config.kapso_phone_number_id.trim().to_string();
        }
        if !kapso_phone_number_id.is_empty() {
            upsert_config(&pool, "KAPSO_PHONE_NUMBER_ID", &kapso_phone_number_id, "Identificador de número telefónico de WhatsApp en Kapso / Meta", "whatsapp").await;
            config_map.insert("KAPSO_PHONE_NUMBER_ID".to_string(), json!(&kapso_phone_number_id));
        }
    }

    let whatsapp_template_name = config_map
        .get("WHATSAPP_TEMPLATE_NAME")
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .or_else(|| std::env::var("WHATSAPP_TEMPLATE_NAME").ok())
        .unwrap_or_else(|| "hello_world".to_string());

    if !config_map.contains_key("WHATSAPP_TEMPLATE_NAME") {
        upsert_config(&pool, "WHATSAPP_TEMPLATE_NAME", &whatsapp_template_name, "Nombre exacto de la plantilla aprobada en Meta Cloud API / WABA", "whatsapp").await;
        config_map.insert("WHATSAPP_TEMPLATE_NAME".to_string(), json!(&whatsapp_template_name));
    }

    let whatsapp_template_language = config_map
        .get("WHATSAPP_TEMPLATE_LANGUAGE")
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .or_else(|| std::env::var("WHATSAPP_TEMPLATE_LANGUAGE").ok())
        .unwrap_or_else(|| "es_MX".to_string());

    if !config_map.contains_key("WHATSAPP_TEMPLATE_LANGUAGE") {
        upsert_config(&pool, "WHATSAPP_TEMPLATE_LANGUAGE", &whatsapp_template_language, "Código de idioma de la plantilla (ej: es_MX, en_US)", "whatsapp").await;
        config_map.insert("WHATSAPP_TEMPLATE_LANGUAGE".to_string(), json!(&whatsapp_template_language));
    }

    let available_models = get_available_claude_models();

    Ok((
        StatusCode::OK,
        Json(json!({
            "claude_model": current_model,
            "claude_max_tokens": current_max_tokens,
            "system_prompt": current_prompt,
            "default_system_prompt": DEFAULT_SYSTEM_PROMPT,
            "available_models": available_models,
            "whatsapp_provider": whatsapp_provider,
            "kapso_api_key": kapso_api_key,
            "kapso_phone_number_id": kapso_phone_number_id,
            "director_whatsapp_phone": director_whatsapp_phone,
            "whatsapp_template_name": whatsapp_template_name,
            "whatsapp_template_language": whatsapp_template_language,
            "todas": config_map
        })),
    ))
}

pub async fn update_configuracion(
    State((pool, _config)): State<(DbPool, Arc<Config>)>,
    Json(payload): Json<UpdateConfiguracionPayload>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
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

    let _ = sqlx::query("ALTER TABLE configuraciones_sistema ADD COLUMN IF NOT EXISTS descripcion TEXT").execute(&pool).await;
    let _ = sqlx::query("ALTER TABLE configuraciones_sistema ADD COLUMN IF NOT EXISTS categoria VARCHAR(50) DEFAULT 'ia'").execute(&pool).await;
    let _ = sqlx::query("ALTER TABLE configuraciones_sistema ADD COLUMN IF NOT EXISTS actualizado_en TIMESTAMPTZ NOT NULL DEFAULT now()").execute(&pool).await;

    if let Some(ref model) = payload.claude_model {
        let m = model.trim();
        if !m.is_empty() {
            upsert_config(&pool, "CLAUDE_MODEL", m, "Modelo de Anthropic Claude seleccionado para la síntesis de boletines", "ia").await;
            tracing::info!("CLAUDE_MODEL actualizado correctamente en BD con '{}'", m);
        }
    }

    if let Some(tokens) = payload.claude_max_tokens {
        if tokens >= 200 && tokens <= 8000 {
            upsert_config(&pool, "CLAUDE_MAX_TOKENS", &tokens.to_string(), "Límite máximo de tokens de salida por síntesis ejecutiva", "ia").await;
            tracing::info!("CLAUDE_MAX_TOKENS actualizado correctamente en BD con '{}'", tokens);
        }
    }

    if let Some(ref prompt) = payload.system_prompt {
        let p = prompt.trim();
        if !p.is_empty() {
            upsert_config(&pool, "CLAUDE_SYSTEM_PROMPT", p, "Instrucciones del sistema para el análisis y síntesis de boletines", "ia").await;
            tracing::info!("CLAUDE_SYSTEM_PROMPT actualizado correctamente en BD con {} caracteres", p.len());
        }
    }

    if let Some(ref provider) = payload.whatsapp_provider {
        let clean_provider = provider.trim();
        if !clean_provider.is_empty() {
            upsert_config(&pool, "WHATSAPP_PROVIDER", clean_provider, "Proveedor activo de WhatsApp: kapso o waha", "whatsapp").await;
        }
    }

    if let Some(ref key) = payload.kapso_api_key {
        let clean_key = key.trim();
        if !clean_key.is_empty() {
            upsert_config(&pool, "KAPSO_API_KEY", clean_key, "Clave de API del proyecto en Kapso (X-API-Key)", "whatsapp").await;
        }
    }

    if let Some(ref phone_id) = payload.kapso_phone_number_id {
        let clean_id = phone_id.trim();
        if !clean_id.is_empty() {
            upsert_config(&pool, "KAPSO_PHONE_NUMBER_ID", clean_id, "Identificador de número telefónico de WhatsApp en Kapso / Meta", "whatsapp").await;
        }
    }

    if let Some(ref tpl_name) = payload.whatsapp_template_name {
        let clean_name = tpl_name.trim();
        if !clean_name.is_empty() {
            upsert_config(&pool, "WHATSAPP_TEMPLATE_NAME", clean_name, "Nombre exacto de la plantilla aprobada en Meta Cloud API / WABA", "whatsapp").await;
        }
    }

    if let Some(ref tpl_lang) = payload.whatsapp_template_language {
        let clean_lang = tpl_lang.trim();
        if !clean_lang.is_empty() {
            upsert_config(&pool, "WHATSAPP_TEMPLATE_LANGUAGE", clean_lang, "Código de idioma de la plantilla (ej: es_MX, en_US)", "whatsapp").await;
        }
    }

    if let Some(ref phone) = payload.director_whatsapp_phone {
        let clean_phone = phone.trim();
        let val_to_save = if clean_phone == "5215512345678" { "" } else { clean_phone };
        upsert_config(&pool, "DIRECTOR_WHATSAPP_PHONE", val_to_save, "Número de WhatsApp de destino del Director en formato E.164", "whatsapp").await;

        if !val_to_save.is_empty() {
            // Sincronizar también en lista_distribucion
            let affected = sqlx::query(
                "UPDATE lista_distribucion 
                 SET telefono = $1, actualizado_en = now()
                 WHERE nombre ILIKE '%Director%'"
            )
            .bind(val_to_save)
            .execute(&pool)
            .await
            .map(|r| r.rows_affected())
            .unwrap_or(0);

            if affected == 0 {
                let _ = sqlx::query(
                    "INSERT INTO lista_distribucion (nombre, telefono, cargo, activo, notas)
                     VALUES ('Director General', $1, 'Dirección Ejecutiva', true, 'Contacto principal')"
                )
                .bind(val_to_save)
                .execute(&pool)
                .await;
            }
        }
    }

    Ok((
        StatusCode::OK,
        Json(json!({
            "mensaje": "Configuración guardada correctamente"
        })),
    ))
}

pub async fn test_whatsapp(
    State((pool, _config)): State<(DbPool, Arc<Config>)>,
    Json(payload): Json<TestWhatsappPayload>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    use crate::services::kapso::{get_whatsapp_config, KapsoClient};
    use std::time::Instant;

    let cfg = get_whatsapp_config(&pool).await;

    let api_key = payload
        .api_key
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .unwrap_or(cfg.api_key.trim());

    let phone_number_id = payload
        .phone_number_id
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .unwrap_or(cfg.phone_number_id.trim());

    let target_phone = payload
        .phone
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty() && *s != "5215512345678")
        .unwrap_or(cfg.director_phone.trim());

    if target_phone.is_empty() || target_phone == "5215512345678" {
        return Ok((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "ok": false,
                "error": "No se ha configurado un número de teléfono de destino válido. Ingresa el número con código de país (ej: 5255...)."
            })),
        ));
    }

    let test_body = payload.message.unwrap_or_else(|| {
        "🔔 *Prueba de Conexión ExposureIQ — WhatsApp Cloud API via Kapso*\n\nEste es un mensaje de prueba para validar la integración oficial en tiempo real.".to_string()
    });

    let client = KapsoClient::new(api_key.to_string(), phone_number_id.to_string());
    let start = Instant::now();

    match client.send_text_with_log(Some(&pool), target_phone, &test_body, "test_whatsapp").await {
        Ok(msg_id) => {
            let latency_ms = start.elapsed().as_millis();
            Ok((
                StatusCode::OK,
                Json(json!({
                    "ok": true,
                    "mensaje": format!("Mensaje entregado exitosamente a {} ({}) en {}ms", target_phone, msg_id, latency_ms),
                    "message_id": msg_id,
                    "latency_ms": latency_ms
                })),
            ))
        }
        Err(e) => {
            let latency_ms = start.elapsed().as_millis();
            Ok((
                StatusCode::OK,
                Json(json!({
                    "ok": false,
                    "error": e.clone(),
                    "mensaje": format!("Fallo al enviar mensaje por Kapso: {}", e),
                    "latency_ms": latency_ms
                })),
            ))
        }
    }
}

pub async fn test_template(
    State((pool, _config)): State<(DbPool, Arc<Config>)>,
    Json(payload): Json<Option<crate::models::TestTemplatePayload>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    use crate::services::kapso::{get_whatsapp_config, KapsoClient};
    use std::time::Instant;

    let req = payload.unwrap_or_default();
    let cfg = get_whatsapp_config(&pool).await;

    let api_key = req
        .api_key
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .unwrap_or(cfg.api_key.trim());

    let phone_number_id = req
        .phone_number_id
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .unwrap_or(cfg.phone_number_id.trim());

    let target_phone = req
        .phone
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty() && *s != "5215512345678")
        .unwrap_or(cfg.director_phone.trim());

    if target_phone.is_empty() || target_phone == "5215512345678" {
        return Ok((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "ok": false,
                "error": "No se ha configurado un número de teléfono de destino válido. Ingresa el número con código de país (ej: 5255...)."
            })),
        ));
    }

    let tpl_name = req
        .template_name
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .unwrap_or(cfg.template_name.trim());

    let tpl_lang = req
        .template_language
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .unwrap_or(cfg.template_language.trim());

    let client = KapsoClient::new(api_key.to_string(), phone_number_id.to_string());
    let start = Instant::now();

    match client.send_template_with_log(Some(&pool), target_phone, tpl_name, tpl_lang, req.variables.as_deref(), "test_plantilla").await {
        Ok(msg_id) => {
            let latency_ms = start.elapsed().as_millis();
            Ok((
                StatusCode::OK,
                Json(json!({
                    "ok": true,
                    "mensaje": format!("Plantilla '{}' ({}) enviada exitosamente a {} ({}) en {}ms", tpl_name, tpl_lang, target_phone, msg_id, latency_ms),
                    "message_id": msg_id,
                    "latency_ms": latency_ms
                })),
            ))
        }
        Err(e) => {
            let latency_ms = start.elapsed().as_millis();
            Ok((
                StatusCode::OK,
                Json(json!({
                    "ok": false,
                    "error": e.clone(),
                    "mensaje": format!("Fallo al enviar plantilla: {}", e),
                    "latency_ms": latency_ms
                })),
            ))
        }
    }
}

pub async fn test_claude(
    State((pool, config)): State<(DbPool, Arc<Config>)>,
    Json(payload): Json<TestClaudePayload>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    ensure_api_logs_schema(&pool).await;

    let worker_url = format!("{}/api/test-claude", config.worker_base_url);
    let client = reqwest::Client::new();
    let start = std::time::Instant::now();
    let model_name = payload.model.clone().unwrap_or_else(|| "claude-3-5-sonnet-20241022".to_string());
    let req_payload_str = json!({ "model": &model_name, "prompt": "Prueba de conexión con Claude" }).to_string();

    let res = match client.post(&worker_url).json(&payload).send().await {
        Ok(r) => r,
        Err(e) => {
            let latency = start.elapsed().as_millis() as i32;
            let err_msg = format!("No se pudo contactar al worker en {}: {}", worker_url, e);
            let _ = sqlx::query(
                "INSERT INTO api_logs (
                    servicio, accion, modelo_o_proveedor, estado, codigo_http,
                    latencia_ms, tokens_input, tokens_output, tokens_total,
                    destinatario, peticion_payload, respuesta_payload, error_mensaje, creado_en
                )
                VALUES ('claude', 'test_conexion', $1, 'error', 503, $2, 0, 0, 0, 'worker', $3, '', $4, now())"
            )
            .bind(&model_name)
            .bind(latency)
            .bind(&req_payload_str)
            .bind(&err_msg)
            .execute(&pool)
            .await;

            return Err((
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({
                    "ok": false,
                    "error": err_msg
                })),
            ));
        }
    };

    let status = res.status();
    let latency = start.elapsed().as_millis() as i32;
    let body: serde_json::Value = res.json().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "ok": false,
                "error": format!("Error leyendo respuesta del worker: {}", e)
            })),
        )
    })?;

    let body_str = body.to_string();
    let is_ok = body.get("ok").and_then(|v| v.as_bool()).unwrap_or(status.is_success());
    let estado_str = if is_ok { "ok" } else { "error" };
    let error_msg = body.get("error").and_then(|v| v.as_str());

    let _ = sqlx::query(
        "INSERT INTO api_logs (
            servicio, accion, modelo_o_proveedor, estado, codigo_http,
            latencia_ms, tokens_input, tokens_output, tokens_total,
            max_tokens_configurado, destinatario, peticion_payload, respuesta_payload, error_mensaje, creado_en
        )
        VALUES ('claude', 'test_conexion', $1, $2, $3, $4, 15, 20, 35, 60, 'test', $5, $6, $7, now())"
    )
    .bind(&model_name)
    .bind(estado_str)
    .bind(status.as_u16() as i32)
    .bind(latency)
    .bind(&req_payload_str)
    .bind(&body_str)
    .bind(error_msg)
    .execute(&pool)
    .await;

    Ok((StatusCode::OK, Json(body)))
}

// ============================================================================
// CRUD Lista de Distribución y Grupos Temáticos WhatsApp
// ============================================================================

pub async fn ensure_tablas_distribucion(pool: &DbPool) {
    let _ = sqlx::query(
        "CREATE TABLE IF NOT EXISTS lista_distribucion (
            id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            nombre          VARCHAR(100) NOT NULL,
            telefono        VARCHAR(50) NOT NULL,
            cargo           VARCHAR(100),
            activo          BOOLEAN NOT NULL DEFAULT true,
            notas           TEXT,
            creado_en       TIMESTAMPTZ NOT NULL DEFAULT now(),
            actualizado_en  TIMESTAMPTZ NOT NULL DEFAULT now()
        )"
    ).execute(pool).await;

    let _ = sqlx::query(
        "CREATE TABLE IF NOT EXISTS grupos_distribucion (
            id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            nombre          VARCHAR(100) NOT NULL UNIQUE,
            descripcion     TEXT,
            color           VARCHAR(30) NOT NULL DEFAULT '#0284c7',
            activo          BOOLEAN NOT NULL DEFAULT true,
            creado_en       TIMESTAMPTZ NOT NULL DEFAULT now()
        )"
    ).execute(pool).await;

    let _ = sqlx::query(
        "CREATE TABLE IF NOT EXISTS destinatarios_grupos (
            destinatario_id UUID NOT NULL REFERENCES lista_distribucion(id) ON DELETE CASCADE,
            grupo_id        UUID NOT NULL REFERENCES grupos_distribucion(id) ON DELETE CASCADE,
            creado_en       TIMESTAMPTZ NOT NULL DEFAULT now(),
            PRIMARY KEY (destinatario_id, grupo_id)
        )"
    ).execute(pool).await;

    let _ = sqlx::query(
        "INSERT INTO grupos_distribucion (nombre, descripcion, color)
         VALUES 
            ('Comité Directivo', 'Recepción de briefings matutinos ejecutivos y decisiones clave', '#0284c7'),
            ('Operaciones & Siniestros', 'Alertas tempranas de seguridad, incidentes carreteros y siniestros', '#16a34a'),
            ('Legal & Regulatorio', 'Reformas jurídicas, CNSF, SHCP, jurisprudencia y laboral', '#9333ea')
         ON CONFLICT (nombre) DO NOTHING"
    ).execute(pool).await;
}

pub async fn list_destinatarios(
    State((pool, _)): State<(DbPool, Arc<Config>)>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    ensure_tablas_distribucion(&pool).await;

    let dest_rows = sqlx::query_as::<_, DestinatarioRow>(
        "SELECT id, nombre, telefono, cargo, activo, notas, creado_en, actualizado_en
         FROM lista_distribucion
         ORDER BY creado_en ASC",
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Error consultando lista de distribución: {}", e)})),
        )
    })?;

    let mut dests: Vec<Destinatario> = dest_rows.into_iter().map(Destinatario::from).collect();

    #[derive(sqlx::FromRow)]
    struct RelRow {
        destinatario_id: Uuid,
        id: Uuid,
        nombre: String,
        color: String,
    }

    let rels = sqlx::query_as::<_, RelRow>(
        "SELECT dg.destinatario_id, g.id, g.nombre, g.color
         FROM destinatarios_grupos dg
         JOIN grupos_distribucion g ON g.id = dg.grupo_id
         ORDER BY g.nombre ASC"
    )
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    for d in &mut dests {
        let matching_grupos: Vec<GrupoResumen> = rels
            .iter()
            .filter(|r| r.destinatario_id == d.id)
            .map(|r| GrupoResumen {
                id: r.id,
                nombre: r.nombre.clone(),
                color: r.color.clone(),
            })
            .collect();
        d.grupos = Some(matching_grupos);
    }

    Ok((StatusCode::OK, Json(json!(dests))))
}

pub async fn create_destinatario(
    State((pool, _)): State<(DbPool, Arc<Config>)>,
    Json(payload): Json<CreateDestinatarioRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    ensure_tablas_distribucion(&pool).await;

    let nombre = payload.nombre.trim();
    let telefono = payload.telefono.trim();

    if nombre.is_empty() || telefono.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "El nombre y el teléfono son obligatorios"})),
        ));
    }

    let activo = payload.activo.unwrap_or(true);

    let dest_row = sqlx::query_as::<_, DestinatarioRow>(
        "INSERT INTO lista_distribucion (nombre, telefono, cargo, activo, notas)
         VALUES ($1, $2, $3, $4, $5)
         RETURNING id, nombre, telefono, cargo, activo, notas, creado_en, actualizado_en",
    )
    .bind(nombre)
    .bind(telefono)
    .bind(payload.cargo.as_deref().map(|s| s.trim()))
    .bind(activo)
    .bind(payload.notas.as_deref().map(|s| s.trim()))
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Error registrando destinatario: {}", e)})),
        )
    })?;

    let mut dest = Destinatario::from(dest_row);

    let mut grupos_resumen = Vec::new();
    if let Some(ref g_ids) = payload.grupo_ids {
        for gid in g_ids {
            let _ = sqlx::query(
                "INSERT INTO destinatarios_grupos (destinatario_id, grupo_id) VALUES ($1, $2) ON CONFLICT DO NOTHING"
            )
            .bind(dest.id)
            .bind(gid)
            .execute(&pool)
            .await;

            if let Ok(Some(g)) = sqlx::query_as::<_, GrupoResumen>(
                "SELECT id, nombre, color FROM grupos_distribucion WHERE id = $1"
            )
            .bind(gid)
            .fetch_optional(&pool)
            .await {
                grupos_resumen.push(g);
            }
        }
    }
    dest.grupos = Some(grupos_resumen);

    // Enviar plantilla oficial si fue solicitado al dar de alta
    let mut plantilla_enviada = false;
    let mut plantilla_error: Option<String> = None;
    let mut kapso_msg_id: Option<String> = None;

    if payload.enviar_plantilla.unwrap_or(false) {
        use crate::services::kapso::{get_whatsapp_config, KapsoClient};
        let w_cfg = get_whatsapp_config(&pool).await;

        if w_cfg.api_key.trim().is_empty() || w_cfg.phone_number_id.trim().is_empty() {
            plantilla_error = Some("No se pudo enviar la plantilla: credenciales de WhatsApp Cloud API (Kapso) no configuradas".to_string());
        } else {
            let client = KapsoClient::new(w_cfg.api_key.clone(), w_cfg.phone_number_id.clone());
            let tpl_name = payload
                .template_name
                .as_deref()
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .unwrap_or(w_cfg.template_name.as_str());

            let tpl_lang = payload
                .template_language
                .as_deref()
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .unwrap_or(w_cfg.template_language.as_str());

            let default_vars = vec![nombre.to_string()];
            let vars = payload.template_variables.as_ref().unwrap_or(&default_vars);

            match client.send_template_with_log(Some(&pool), telefono, tpl_name, tpl_lang, Some(vars), "plantilla_bienvenida").await {
                Ok(msg_id) => {
                    plantilla_enviada = true;
                    kapso_msg_id = Some(msg_id.clone());

                    let _ = sqlx::query(
                        "INSERT INTO mensajes_pendientes (tipo, referencia_id, texto, destinatario, estado, proveedor, kapso_message_id, meta_status, confirmado_en)
                         VALUES ('alerta', $1, $2, $3, 'confirmado', 'kapso', $4, 'sent', now())",
                    )
                    .bind(dest.id)
                    .bind(format!("Plantilla de apertura: {}", tpl_name))
                    .bind(telefono)
                    .bind(&msg_id)
                    .execute(&pool)
                    .await;
                }
                Err(e) => {
                    plantilla_error = Some(e.clone());
                    let _ = sqlx::query(
                        "INSERT INTO mensajes_pendientes (tipo, referencia_id, texto, destinatario, estado, proveedor, error_mensaje)
                         VALUES ('alerta', $1, $2, $3, 'error', 'kapso', $4)",
                    )
                    .bind(dest.id)
                    .bind(format!("Intento de plantilla de apertura: {}", tpl_name))
                    .bind(telefono)
                    .bind(&e)
                    .execute(&pool)
                    .await;
                }
            }
        }
    }

    Ok((StatusCode::CREATED, Json(json!({
        "id": dest.id,
        "nombre": dest.nombre,
        "telefono": dest.telefono,
        "cargo": dest.cargo,
        "activo": dest.activo,
        "notas": dest.notas,
        "grupos": dest.grupos,
        "creado_en": dest.creado_en,
        "actualizado_en": dest.actualizado_en,
        "plantilla_enviada": plantilla_enviada,
        "plantilla_error": plantilla_error,
        "kapso_message_id": kapso_msg_id,
    }))))
}

pub async fn update_destinatario(
    State((pool, _)): State<(DbPool, Arc<Config>)>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateDestinatarioRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    ensure_tablas_distribucion(&pool).await;

    let nombre = payload.nombre.trim();
    let telefono = payload.telefono.trim();

    if nombre.is_empty() || telefono.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "El nombre y el teléfono son obligatorios"})),
        ));
    }

    let dest_row = sqlx::query_as::<_, DestinatarioRow>(
        "UPDATE lista_distribucion
         SET nombre = $1, telefono = $2, cargo = $3, activo = $4, notas = $5, actualizado_en = now()
         WHERE id = $6
         RETURNING id, nombre, telefono, cargo, activo, notas, creado_en, actualizado_en",
    )
    .bind(nombre)
    .bind(telefono)
    .bind(payload.cargo.as_deref().map(|s| s.trim()))
    .bind(payload.activo)
    .bind(payload.notas.as_deref().map(|s| s.trim()))
    .bind(id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Error actualizando destinatario: {}", e)})),
        )
    })?
    .ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Destinatario no encontrado"})),
        )
    })?;

    let mut dest = Destinatario::from(dest_row);

    if let Some(ref g_ids) = payload.grupo_ids {
        let _ = sqlx::query("DELETE FROM destinatarios_grupos WHERE destinatario_id = $1")
            .bind(dest.id)
            .execute(&pool)
            .await;

        let mut grupos_resumen = Vec::new();
        for gid in g_ids {
            let _ = sqlx::query(
                "INSERT INTO destinatarios_grupos (destinatario_id, grupo_id) VALUES ($1, $2) ON CONFLICT DO NOTHING"
            )
            .bind(dest.id)
            .bind(gid)
            .execute(&pool)
            .await;

            if let Ok(Some(g)) = sqlx::query_as::<_, GrupoResumen>(
                "SELECT id, nombre, color FROM grupos_distribucion WHERE id = $1"
            )
            .bind(gid)
            .fetch_optional(&pool)
            .await {
                grupos_resumen.push(g);
            }
        }
        dest.grupos = Some(grupos_resumen);
    } else {
        // Cargar grupos existentes
        let rels = sqlx::query_as::<_, GrupoResumen>(
            "SELECT g.id, g.nombre, g.color
             FROM destinatarios_grupos dg
             JOIN grupos_distribucion g ON g.id = dg.grupo_id
             WHERE dg.destinatario_id = $1
             ORDER BY g.nombre ASC"
        )
        .bind(dest.id)
        .fetch_all(&pool)
        .await
        .unwrap_or_default();
        dest.grupos = Some(rels);
    }

    Ok((StatusCode::OK, Json(json!(dest))))
}

/// Envía una plantilla oficial de WhatsApp al contacto seleccionado para abrir la ventana de 24 horas
pub async fn enviar_plantilla_destinatario(
    State((pool, _)): State<(DbPool, Arc<Config>)>,
    Path(id): Path<Uuid>,
    Json(payload): Json<Option<crate::models::EnviarPlantillaRequest>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    use crate::services::kapso::{get_whatsapp_config, KapsoClient};

    let req = payload.unwrap_or_default();

    let dest_opt = sqlx::query_as::<_, DestinatarioRow>(
        "SELECT id, nombre, telefono, cargo, activo, notas, creado_en, actualizado_en 
         FROM lista_distribucion WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"ok": false, "error": format!("Error buscando destinatario: {}", e)})),
        )
    })?;

    let dest = match dest_opt {
        Some(d) => d,
        None => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(json!({"ok": false, "error": "Destinatario no encontrado en la lista"})),
            ));
        }
    };

    let w_cfg = get_whatsapp_config(&pool).await;

    let api_key = req
        .api_key
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .unwrap_or(w_cfg.api_key.trim());

    let phone_number_id = req
        .phone_number_id
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .unwrap_or(w_cfg.phone_number_id.trim());

    if api_key.is_empty() || phone_number_id.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "ok": false,
                "error": "No hay credenciales de WhatsApp Cloud API (Kapso) configuradas"
            })),
        ));
    }

    let target_phone = req
        .telefono
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .unwrap_or(dest.telefono.trim());

    let tpl_name = req
        .template_name
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .unwrap_or(w_cfg.template_name.as_str());

    let tpl_lang = req
        .template_language
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .unwrap_or(w_cfg.template_language.as_str());

    let default_vars = vec![dest.nombre.clone()];
    let vars = req.variables.as_ref().unwrap_or(&default_vars);

    let client = KapsoClient::new(api_key.to_string(), phone_number_id.to_string());

    match client.send_template_with_log(Some(&pool), target_phone, tpl_name, tpl_lang, Some(vars), "plantilla_destinatario").await {
        Ok(msg_id) => {
            let _ = sqlx::query(
                "INSERT INTO mensajes_pendientes (tipo, referencia_id, texto, destinatario, estado, proveedor, kapso_message_id, meta_status, confirmado_en)
                 VALUES ('alerta', $1, $2, $3, 'confirmado', 'kapso', $4, 'sent', now())",
            )
            .bind(dest.id)
            .bind(format!("Plantilla de apertura enviada: {}", tpl_name))
            .bind(target_phone)
            .bind(&msg_id)
            .execute(&pool)
            .await;

            Ok((
                StatusCode::OK,
                Json(json!({
                    "ok": true,
                    "mensaje": format!("Plantilla '{}' ({}) enviada exitosamente a {} ({})", tpl_name, tpl_lang, dest.nombre, target_phone),
                    "destinatario": dest.nombre,
                    "telefono": target_phone,
                    "kapso_message_id": msg_id,
                })),
            ))
        }
        Err(e) => {
            let _ = sqlx::query(
                "INSERT INTO mensajes_pendientes (tipo, referencia_id, texto, destinatario, estado, proveedor, error_mensaje)
                 VALUES ('alerta', $1, $2, $3, 'error', 'kapso', $4)",
            )
            .bind(dest.id)
            .bind(format!("Fallo al enviar plantilla: {}", tpl_name))
            .bind(target_phone)
            .bind(&e)
            .execute(&pool)
            .await;

            Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "ok": false,
                    "error": e.clone(),
                    "mensaje": format!("Error enviando plantilla a {}: {}", dest.nombre, e),
                })),
            ))
        }
    }
}

pub async fn toggle_destinatario(
    State((pool, _)): State<(DbPool, Arc<Config>)>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let row = sqlx::query_as::<_, DestinatarioRow>(
        "UPDATE lista_distribucion
         SET activo = NOT activo, actualizado_en = now()
         WHERE id = $1
         RETURNING id, nombre, telefono, cargo, activo, notas, creado_en, actualizado_en",
    )
    .bind(id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Error modificando estado del destinatario: {}", e)})),
        )
    })?
    .ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Destinatario no encontrado"})),
        )
    })?;

    let destinatario = Destinatario::from(row);

    Ok((StatusCode::OK, Json(json!(destinatario))))
}

pub async fn delete_destinatario(
    State((pool, _)): State<(DbPool, Arc<Config>)>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let rows_affected = sqlx::query(
        "DELETE FROM lista_distribucion WHERE id = $1",
    )
    .bind(id)
    .execute(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Error eliminando destinatario: {}", e)})),
        )
    })?
    .rows_affected();

    if rows_affected == 0 {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Destinatario no encontrado"})),
        ));
    }

    Ok((
        StatusCode::OK,
        Json(json!({"mensaje": "Destinatario eliminado de la lista de distribución"})),
    ))
}

// ============================================================================
// CRUD Listas / Grupos de Distribución
// ============================================================================

pub async fn list_grupos(
    State((pool, _)): State<(DbPool, Arc<Config>)>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    ensure_tablas_distribucion(&pool).await;

    let grupos = sqlx::query_as::<_, GrupoDistribucion>(
        "SELECT g.id, g.nombre, g.descripcion, g.color, g.activo, g.creado_en,
                COUNT(dg.destinatario_id) as total_miembros
         FROM grupos_distribucion g
         LEFT JOIN destinatarios_grupos dg ON dg.grupo_id = g.id
         GROUP BY g.id, g.nombre, g.descripcion, g.color, g.activo, g.creado_en
         ORDER BY g.nombre ASC",
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Error consultando listas de distribución: {}", e)})),
        )
    })?;

    Ok((StatusCode::OK, Json(json!(grupos))))
}

pub async fn create_grupo(
    State((pool, _)): State<(DbPool, Arc<Config>)>,
    Json(payload): Json<CreateGrupoRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    ensure_tablas_distribucion(&pool).await;

    let nombre = payload.nombre.trim();
    if nombre.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "El nombre de la lista de distribución es obligatorio"})),
        ));
    }

    let color = payload.color.as_deref().unwrap_or("#0284c7");
    let activo = payload.activo.unwrap_or(true);

    let grupo = sqlx::query_as::<_, GrupoDistribucion>(
        "INSERT INTO grupos_distribucion (nombre, descripcion, color, activo)
         VALUES ($1, $2, $3, $4)
         RETURNING id, nombre, descripcion, color, activo, creado_en, 0::bigint as total_miembros",
    )
    .bind(nombre)
    .bind(payload.descripcion.as_deref().map(|s| s.trim()))
    .bind(color)
    .bind(activo)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Error creando lista de distribución (posible nombre duplicado): {}", e)})),
        )
    })?;

    Ok((StatusCode::CREATED, Json(json!(grupo))))
}

pub async fn update_grupo(
    State((pool, _)): State<(DbPool, Arc<Config>)>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateGrupoRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let nombre = payload.nombre.trim();
    if nombre.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "El nombre de la lista de distribución es obligatorio"})),
        ));
    }

    let color = payload.color.as_deref().unwrap_or("#0284c7");

    let grupo = sqlx::query_as::<_, GrupoDistribucion>(
        "UPDATE grupos_distribucion
         SET nombre = $1, descripcion = $2, color = $3, activo = $4
         WHERE id = $5
         RETURNING id, nombre, descripcion, color, activo, creado_en, 
                   (SELECT COUNT(*) FROM destinatarios_grupos WHERE grupo_id = $5) as total_miembros",
    )
    .bind(nombre)
    .bind(payload.descripcion.as_deref().map(|s| s.trim()))
    .bind(color)
    .bind(payload.activo)
    .bind(id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Error actualizando lista: {}", e)})),
        )
    })?
    .ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Lista de distribución no encontrada"})),
        )
    })?;

    Ok((StatusCode::OK, Json(json!(grupo))))
}

pub async fn delete_grupo(
    State((pool, _)): State<(DbPool, Arc<Config>)>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let rows_affected = sqlx::query("DELETE FROM grupos_distribucion WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Error eliminando lista: {}", e)})),
            )
        })?
        .rows_affected();

    if rows_affected == 0 {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Lista de distribución no encontrada"})),
        ));
    }

    Ok((
        StatusCode::OK,
        Json(json!({"mensaje": "Lista de distribución eliminada exitosamente"})),
    ))
}

// ============================================================================
// Métodos para Auditoría y Logs de APIs (Claude y WhatsApp)
// ============================================================================

pub async fn ensure_api_logs_schema(pool: &DbPool) {
    let _ = sqlx::query("CREATE EXTENSION IF NOT EXISTS \"pgcrypto\"").execute(pool).await;

    let res = sqlx::query(
        "CREATE TABLE IF NOT EXISTS api_logs (
            id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            servicio                VARCHAR(50) NOT NULL,
            accion                  VARCHAR(100) NOT NULL,
            modelo_o_proveedor      VARCHAR(100),
            estado                  VARCHAR(50) NOT NULL,
            codigo_http             INT,
            latencia_ms             INT,
            tokens_input            INT DEFAULT 0,
            tokens_output           INT DEFAULT 0,
            tokens_total            INT DEFAULT 0,
            max_tokens_configurado  INT,
            destinatario            VARCHAR(100),
            peticion_payload        TEXT,
            respuesta_payload       TEXT,
            error_mensaje           TEXT,
            creado_en               TIMESTAMPTZ NOT NULL DEFAULT now()
        )"
    )
    .execute(pool)
    .await;

    if let Err(e) = res {
        tracing::error!("❌ Error asegurando tabla api_logs: {}", e);
    }

    let _ = sqlx::query("CREATE INDEX IF NOT EXISTS idx_api_logs_servicio_fecha ON api_logs (servicio, creado_en DESC)").execute(pool).await;
    let _ = sqlx::query("CREATE INDEX IF NOT EXISTS idx_api_logs_creado_en ON api_logs (creado_en DESC)").execute(pool).await;
    let _ = sqlx::query("CREATE INDEX IF NOT EXISTS idx_api_logs_estado ON api_logs (estado)").execute(pool).await;
}

pub async fn get_api_logs(
    State((pool, _)): State<(DbPool, Arc<Config>)>,
    axum::extract::Query(query): axum::extract::Query<crate::models::GetApiLogsQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    ensure_api_logs_schema(&pool).await;

    let limit = query.limite.unwrap_or(50).clamp(1, 200);
    let offset = query.offset.unwrap_or(0).max(0);

    let mut sql = "SELECT id, servicio, accion, modelo_o_proveedor, estado, codigo_http, latencia_ms, tokens_input, tokens_output, tokens_total, max_tokens_configurado, destinatario, peticion_payload, respuesta_payload, error_mensaje, creado_en FROM api_logs".to_string();
    let mut count_sql = "SELECT COUNT(*) FROM api_logs".to_string();
    let mut conditions = Vec::new();

    if let Some(ref s) = query.servicio {
        let clean = s.trim().to_lowercase();
        if !clean.is_empty() && clean != "todos" {
            conditions.push(format!("servicio = '{}'", clean));
        }
    }

    if let Some(ref st) = query.estado {
        let clean = st.trim().to_lowercase();
        if !clean.is_empty() && clean != "todos" {
            if clean == "ok" || clean == "exitoso" {
                conditions.push("estado IN ('ok', 'exitoso')".to_string());
            } else {
                conditions.push(format!("estado = '{}'", clean));
            }
        }
    }

    if !conditions.is_empty() {
        let where_clause = format!(" WHERE {}", conditions.join(" AND "));
        sql.push_str(&where_clause);
        count_sql.push_str(&where_clause);
    }

    sql.push_str(&format!(" ORDER BY creado_en DESC LIMIT {} OFFSET {}", limit, offset));

    let rows = match sqlx::query_as::<_, crate::models::ApiLogRow>(&sql).fetch_all(&pool).await {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("Error consultando api_logs: {}", e);
            Vec::new()
        }
    };

    let total: i64 = sqlx::query_scalar(&count_sql).fetch_one(&pool).await.unwrap_or(0);

    #[derive(sqlx::FromRow)]
    struct RawStats {
        total_claude: Option<i64>,
        tokens_claude: Option<i64>,
        exitosos_claude: Option<i64>,
        errores_claude: Option<i64>,
        total_whatsapp: Option<i64>,
        exitosos_whatsapp: Option<i64>,
        errores_whatsapp: Option<i64>,
    }

    let stats_row = sqlx::query_as::<_, RawStats>(
        "SELECT 
            COUNT(*) FILTER (WHERE servicio = 'claude') as total_claude,
            COALESCE(SUM(tokens_total) FILTER (WHERE servicio = 'claude'), 0) as tokens_claude,
            COUNT(*) FILTER (WHERE servicio = 'claude' AND estado IN ('ok', 'exitoso')) as exitosos_claude,
            COUNT(*) FILTER (WHERE servicio = 'claude' AND estado = 'error') as errores_claude,
            COUNT(*) FILTER (WHERE servicio = 'whatsapp') as total_whatsapp,
            COUNT(*) FILTER (WHERE servicio = 'whatsapp' AND estado IN ('ok', 'exitoso')) as exitosos_whatsapp,
            COUNT(*) FILTER (WHERE servicio = 'whatsapp' AND estado = 'error') as errores_whatsapp
         FROM api_logs"
    )
    .fetch_optional(&pool)
    .await
    .unwrap_or(None);

    let stats = if let Some(s) = stats_row {
        crate::models::ApiLogStats {
            total_claude: s.total_claude.unwrap_or(0),
            tokens_claude: s.tokens_claude.unwrap_or(0),
            exitosos_claude: s.exitosos_claude.unwrap_or(0),
            errores_claude: s.errores_claude.unwrap_or(0),
            total_whatsapp: s.total_whatsapp.unwrap_or(0),
            exitosos_whatsapp: s.exitosos_whatsapp.unwrap_or(0),
            errores_whatsapp: s.errores_whatsapp.unwrap_or(0),
        }
    } else {
        crate::models::ApiLogStats::default()
    };

    let logs: Vec<crate::models::ApiLogItem> = rows.into_iter().map(|r| crate::models::ApiLogItem {
        id: r.id.to_string(),
        servicio: r.servicio,
        accion: r.accion,
        modelo_o_proveedor: r.modelo_o_proveedor.unwrap_or_default(),
        estado: r.estado,
        codigo_http: r.codigo_http,
        latencia_ms: r.latencia_ms,
        tokens_input: r.tokens_input.unwrap_or(0),
        tokens_output: r.tokens_output.unwrap_or(0),
        tokens_total: r.tokens_total.unwrap_or(0),
        max_tokens_configurado: r.max_tokens_configurado,
        destinatario: r.destinatario,
        peticion_payload: r.peticion_payload.unwrap_or_default(),
        respuesta_payload: r.respuesta_payload.unwrap_or_default(),
        error_mensaje: r.error_mensaje,
        creado_en: r.creado_en.to_rfc3339(),
    }).collect();

    Ok((
        StatusCode::OK,
        Json(json!({
            "logs": logs,
            "stats": stats,
            "total": total
        }))
    ))
}

pub async fn create_api_log(
    State((pool, _)): State<(DbPool, Arc<Config>)>,
    Json(payload): Json<crate::models::CreateApiLogPayload>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    ensure_api_logs_schema(&pool).await;

    let res = sqlx::query(
        "INSERT INTO api_logs (
            servicio, accion, modelo_o_proveedor, estado, codigo_http,
            latencia_ms, tokens_input, tokens_output, tokens_total,
            max_tokens_configurado, destinatario, peticion_payload,
            respuesta_payload, error_mensaje, creado_en
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, now())"
    )
    .bind(&payload.servicio)
    .bind(&payload.accion)
    .bind(&payload.modelo_o_proveedor)
    .bind(&payload.estado)
    .bind(payload.codigo_http)
    .bind(payload.latencia_ms)
    .bind(payload.tokens_input.unwrap_or(0))
    .bind(payload.tokens_output.unwrap_or(0))
    .bind(payload.tokens_total.unwrap_or(0))
    .bind(payload.max_tokens_configurado)
    .bind(&payload.destinatario)
    .bind(&payload.peticion_payload)
    .bind(&payload.respuesta_payload)
    .bind(&payload.error_mensaje)
    .execute(&pool)
    .await;

    match res {
        Ok(_) => Ok((StatusCode::CREATED, Json(json!({"ok": true})))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"ok": false, "error": e.to_string()})))),
    }
}

pub async fn clear_api_logs(
    State((pool, _)): State<(DbPool, Arc<Config>)>,
    axum::extract::Query(query): axum::extract::Query<crate::models::ClearApiLogsQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    ensure_api_logs_schema(&pool).await;

    let affected = if let Some(ref s) = query.servicio {
        let clean = s.trim().to_lowercase();
        if clean == "claude" || clean == "whatsapp" {
            sqlx::query("DELETE FROM api_logs WHERE servicio = $1").bind(clean).execute(&pool).await
        } else {
            sqlx::query("DELETE FROM api_logs").execute(&pool).await
        }
    } else {
        sqlx::query("DELETE FROM api_logs").execute(&pool).await
    };

    match affected {
        Ok(res) => Ok((StatusCode::OK, Json(json!({"ok": true, "eliminados": res.rows_affected()})))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"ok": false, "error": e.to_string()})))),
    }
}
