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
        CreateDestinatarioRequest, CreateGrupoRequest, Destinatario, GrupoDistribucion,
        GrupoResumen, UpdateDestinatarioRequest, UpdateGrupoRequest,
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
    pub system_prompt: Option<String>,
    pub whatsapp_provider: Option<String>,
    pub kapso_api_key: Option<String>,
    pub kapso_phone_number_id: Option<String>,
    pub director_whatsapp_phone: Option<String>,
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
    // 1. Intentar actualizar si ya existe la clave
    let update_res = sqlx::query(
        "UPDATE configuraciones_sistema SET valor = $1, actualizado_en = now() WHERE clave = $2"
    )
    .bind(clean_val)
    .bind(clave)
    .execute(pool)
    .await;

    match update_res {
        Ok(res) if res.rows_affected() > 0 => {
            tracing::info!("Configuración {} actualizada a '{}'", clave, clean_val);
        }
        _ => {
            // 2. Si no existía, insertar
            let insert_res = sqlx::query(
                "INSERT INTO configuraciones_sistema (clave, valor, descripcion, categoria, actualizado_en)
                 VALUES ($1, $2, $3, $4, now())"
            )
            .bind(clave)
            .bind(clean_val)
            .bind(descripcion)
            .bind(categoria)
            .execute(pool)
            .await;

            if let Err(e) = insert_res {
                tracing::warn!("Insert completo falló para {}: {}, intentando inserción básica", clave, e);
                let _ = sqlx::query(
                    "INSERT INTO configuraciones_sistema (clave, valor) VALUES ($1, $2)
                     ON CONFLICT (clave) DO UPDATE SET valor = EXCLUDED.valor"
                )
                .bind(clave)
                .bind(clean_val)
                .execute(pool)
                .await;
            }
        }
    }
}

pub async fn get_configuraciones(
    State((pool, _config)): State<(DbPool, Arc<Config>)>,
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

    let mut current_model = "claude-sonnet-4-5-20250929".to_string();
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
                "CLAUDE_MODEL" => current_model = v,
                "CLAUDE_SYSTEM_PROMPT" => current_prompt = v,
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

    if director_whatsapp_phone.trim().is_empty() {
        if let Ok(env_phone) = std::env::var("DIRECTOR_WHATSAPP_PHONE") {
            if env_phone.trim() != "5215512345678" {
                director_whatsapp_phone = env_phone;
            }
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

    if kapso_api_key.trim().is_empty() {
        if let Ok(env_key) = std::env::var("KAPSO_API_KEY") {
            kapso_api_key = env_key;
        }
    }
    if kapso_phone_number_id.trim().is_empty() {
        if let Ok(env_id) = std::env::var("KAPSO_PHONE_NUMBER_ID") {
            kapso_phone_number_id = env_id;
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
            "whatsapp_provider": whatsapp_provider,
            "kapso_api_key": kapso_api_key,
            "kapso_phone_number_id": kapso_phone_number_id,
            "director_whatsapp_phone": director_whatsapp_phone,
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

    if let Some(ref model) = payload.claude_model {
        let m = model.trim();
        if !m.is_empty() {
            upsert_config(&pool, "CLAUDE_MODEL", m, "Modelo de Anthropic Claude seleccionado para la síntesis de boletines", "ia").await;
        }
    }

    if let Some(ref prompt) = payload.system_prompt {
        let p = prompt.trim();
        if !p.is_empty() {
            upsert_config(&pool, "CLAUDE_SYSTEM_PROMPT", p, "Instrucciones del sistema para el análisis y síntesis de boletines", "ia").await;
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

    match client.send_text(target_phone, &test_body).await {
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

    let mut dests = sqlx::query_as::<_, Destinatario>(
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

    let mut dest = sqlx::query_as::<_, Destinatario>(
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

    Ok((StatusCode::CREATED, Json(json!(dest))))
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

    let mut dest = sqlx::query_as::<_, Destinatario>(
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

pub async fn toggle_destinatario(
    State((pool, _)): State<(DbPool, Arc<Config>)>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let destinatario = sqlx::query_as::<_, Destinatario>(
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
