use axum::{
    extract::{Extension, Multipart, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::{NaiveDate, Utc};
use serde::Deserialize;
use serde_json::json;
use std::{fs, path::PathBuf, sync::Arc};
use tokio::io::AsyncWriteExt;
use tracing::{error, info};
use uuid::Uuid;

use crate::{
    config::Config,
    db::DbPool,
    middleware::auth::AuthContext,
    models::{
        ActualizarSintesisDiariaRequest, AprobarBoletinRequest, AprobarSintesisDiariaRequest,
        Boletin, BoletinDetailResponse, BoletinEnWorkspace, ConsolidarSintesisFechaRequest,
        Seccion, SintesisDiaria, SintesisDiariaResumen, SintesisGenerada,
        ToggleDocumentoSeleccionRequest, UpdateSintesisRequest, WorkspaceFechaResponse,
    },
};

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn list_boletines(
    State((pool, _)): State<(DbPool, Arc<Config>)>,
    Query(query): Query<ListQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let limit = query.limit.unwrap_or(20);
    let offset = query.offset.unwrap_or(0);

    let boletines = sqlx::query_as::<_, Boletin>(
        "SELECT id, fecha_boletin, nombre_archivo, ruta_archivo, subido_por, estado, 
                total_paginas, error_mensaje, creado_en, actualizado_en
         FROM boletines
         ORDER BY fecha_boletin DESC, creado_en DESC
         LIMIT $1 OFFSET $2",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Error al obtener boletines: {}", e)})),
        )
    })?;

    Ok((StatusCode::OK, Json(boletines)))
}

pub async fn get_boletin(
    State((pool, _)): State<(DbPool, Arc<Config>)>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let boletin = sqlx::query_as::<_, Boletin>(
        "SELECT id, fecha_boletin, nombre_archivo, ruta_archivo, subido_por, estado, 
                total_paginas, error_mensaje, creado_en, actualizado_en
         FROM boletines WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Error al consultar boletín: {}", e)})),
        )
    })?
    .ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Boletín no encontrado"})),
        )
    })?;

    let sintesis = sqlx::query_as::<_, SintesisGenerada>(
        "SELECT id, boletin_id, texto, temas, modelo_usado, tokens_usados, creado_en
         FROM sintesis_generadas WHERE boletin_id = $1
         ORDER BY creado_en DESC LIMIT 1",
    )
    .bind(id)
    .fetch_optional(&pool)
    .await
    .unwrap_or(None);

    let secciones = sqlx::query_as::<_, Seccion>(
        "SELECT id, boletin_id, orden, tema, pagina_inicio, pagina_fin, contenido, creado_en
         FROM secciones WHERE boletin_id = $1
         ORDER BY orden ASC",
    )
    .bind(id)
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    let response = BoletinDetailResponse {
        boletin,
        sintesis,
        secciones,
    };

    Ok((StatusCode::OK, Json(response)))
}

pub async fn upload_boletin(
    Extension(auth_ctx): Extension<AuthContext>,
    State((pool, config)): State<(DbPool, Arc<Config>)>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let mut file_name = String::new();
    let mut file_bytes: Vec<u8> = Vec::new();
    let mut fecha_boletin: Option<NaiveDate> = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": format!("Error leyendo multipart: {}", e)})),
        )
    })? {
        let name = field.name().unwrap_or("").to_string();

        if name == "fecha" {
            let text = field.text().await.unwrap_or_default();
            if let Ok(d) = NaiveDate::parse_from_str(&text, "%Y-%m-%d") {
                fecha_boletin = Some(d);
            }
        } else if name == "file" || name == "archivo" {
            file_name = field
                .file_name()
                .unwrap_or("boletin_coparmex.pdf")
                .to_string();
            file_bytes = field.bytes().await.map_err(|e| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(json!({"error": format!("Error descargando archivo: {}", e)})),
                )
            })?.to_vec();
        }
    }

    if file_bytes.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "No se recibió ningún archivo PDF"})),
        ));
    }

    if !file_name.to_lowercase().ends_with(".pdf") {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "El archivo debe tener formato .pdf"})),
        ));
    }

    // Por defecto usar la fecha local de CDMX (UTC-6) si no se especifica
    let fecha = fecha_boletin.unwrap_or_else(|| (Utc::now() - chrono::Duration::hours(6)).date_naive());
    let boletin_id = Uuid::new_v4();

    // Crear directorio si no existe
    let upload_dir = PathBuf::from(&config.upload_dir);
    if let Err(e) = fs::create_dir_all(&upload_dir) {
        error!("Error creando directorio de subidas: {}", e);
    }

    let safe_file_name = format!("{}_{}", boletin_id, file_name);
    let target_path = upload_dir.join(&safe_file_name);

    let mut file = tokio::fs::File::create(&target_path)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Error guardando archivo en disco: {}", e)})),
            )
        })?;

    file.write_all(&file_bytes).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Error escribiendo datos de archivo: {}", e)})),
        )
    })?;

    // Obtener ruta canónica absoluta para que los workers en otros directorios la encuentren siempre
    let target_path_abs = std::fs::canonicalize(&target_path).unwrap_or_else(|_| target_path.clone());
    let target_path_str = target_path_abs.to_string_lossy().to_string();

    // Registrar en PostgreSQL
    let boletin = sqlx::query_as::<_, Boletin>(
        "INSERT INTO boletines (id, fecha_boletin, nombre_archivo, ruta_archivo, subido_por, estado)
         VALUES ($1, $2, $3, $4, $5, 'pendiente_ocr')
         RETURNING id, fecha_boletin, nombre_archivo, ruta_archivo, subido_por, estado, 
                   total_paginas, error_mensaje, creado_en, actualizado_en",
    )
    .bind(boletin_id)
    .bind(fecha)
    .bind(&file_name)
    .bind(&target_path_str)
    .bind(auth_ctx.user_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Error insertando en base de datos: {}", e)})),
        )
    })?;

    // Disparar llamada asíncrona al worker Python en background
    let worker_url = format!("{}/api/process-boletin", config.worker_base_url);
    let client = reqwest::Client::new();
    let b_id_str = boletin_id.to_string();
    let f_path = target_path_str.clone();

    tokio::spawn(async move {
        info!("Notificando a worker Python de nuevo boletín {}", b_id_str);
        let payload = json!({
            "boletin_id": b_id_str,
            "ruta_archivo": f_path
        });
        if let Err(e) = client.post(&worker_url).json(&payload).send().await {
            error!("No se pudo contactar al worker Python: {}", e);
        }
    });

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "mensaje": "Boletín subido exitosamente y encolado para OCR",
            "boletin": boletin
        })),
    ))
}

pub async fn actualizar_sintesis(
    State((pool, _)): State<(DbPool, Arc<Config>)>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateSintesisRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let affected = sqlx::query(
        "UPDATE sintesis_generadas SET texto = $1 WHERE boletin_id = $2",
    )
    .bind(&payload.texto)
    .bind(id)
    .execute(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Error guardando síntesis: {}", e)})),
        )
    })?
    .rows_affected();

    if affected == 0 {
        // Si no existía, insertarla
        sqlx::query(
            "INSERT INTO sintesis_generadas (boletin_id, texto, modelo_usado) VALUES ($1, $2, 'manual_edit')",
        )
        .bind(id)
        .bind(&payload.texto)
        .execute(&pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Error insertando síntesis: {}", e)})),
            )
        })?;
    }

    Ok((
        StatusCode::OK,
        Json(json!({
            "mensaje": "Síntesis actualizada exitosamente",
            "boletin_id": id
        })),
    ))
}

pub async fn aprobar_boletin(
    State((pool, config)): State<(DbPool, Arc<Config>)>,
    Path(id): Path<Uuid>,
    Json(payload): Json<AprobarBoletinRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let mut tx = pool.begin().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Error de base de datos: {}", e)})),
        )
    })?;

    // 1. Guardar o actualizar la síntesis definitiva
    let affected = sqlx::query(
        "UPDATE sintesis_generadas SET texto = $1 WHERE boletin_id = $2",
    )
    .bind(&payload.texto)
    .bind(id)
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Error actualizando síntesis: {}", e)})),
        )
    })?
    .rows_affected();

    if affected == 0 {
        sqlx::query(
            "INSERT INTO sintesis_generadas (boletin_id, texto, modelo_usado) VALUES ($1, $2, 'approved_edit')",
        )
        .bind(id)
        .bind(&payload.texto)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Error registrando síntesis: {}", e)})),
            )
        })?;
    }

    // 2. Asegurar que la restricción CHECK admita el estado 'aprobado'
    let _ = sqlx::query("ALTER TABLE boletines DROP CONSTRAINT IF EXISTS boletines_estado_check")
        .execute(&mut *tx)
        .await;
    let _ = sqlx::query("ALTER TABLE boletines ADD CONSTRAINT boletines_estado_check CHECK (estado IN ('pendiente_ocr', 'en_ocr', 'ocr_completo', 'error_ocr', 'sintesis_lista', 'error_sintesis', 'aprobado', 'enviado'))")
        .execute(&mut *tx)
        .await;

    // Marcar boletín como aprobado
    sqlx::query(
        "UPDATE boletines SET estado = 'aprobado', actualizado_en = now() WHERE id = $1",
    )
    .bind(id)
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Error marcando boletín como aprobado: {}", e)})),
        )
    })?;

    // 3. Resolver destinatarios reales (lista de distribución o teléfono del director configurado)
    use crate::services::kapso::{get_whatsapp_config, KapsoClient};
    let whatsapp_cfg = get_whatsapp_config(&pool).await;

    #[derive(sqlx::FromRow)]
    struct DestRow {
        telefono: String,
    }

    let destinatarios_db = sqlx::query_as::<_, DestRow>(
        "SELECT telefono FROM lista_distribucion WHERE activo = true AND telefono != '5215512345678' AND trim(telefono) != ''"
    )
    .fetch_all(&mut *tx)
    .await
    .unwrap_or_default();

    let mut target_phones: Vec<String> = if destinatarios_db.is_empty() {
        if !whatsapp_cfg.director_phone.trim().is_empty() && whatsapp_cfg.director_phone.trim() != "5215512345678" {
            vec![whatsapp_cfg.director_phone.trim().to_string()]
        } else if !config.director_whatsapp.trim().is_empty() && config.director_whatsapp.trim() != "5215512345678" {
            vec![config.director_whatsapp.trim().to_string()]
        } else {
            vec![]
        }
    } else {
        destinatarios_db
            .into_iter()
            .map(|d| d.telefono)
            .filter(|t| !t.trim().is_empty() && t.trim() != "5215512345678")
            .collect()
    };

    if target_phones.is_empty() {
        target_phones.push("Sin destinatario configurado".to_string());
    }

    for phone in &target_phones {
        sqlx::query(
            "INSERT INTO mensajes_pendientes (tipo, referencia_id, texto, destinatario, estado, proveedor)
             VALUES ('brief', $1, $2, $3, 'pendiente', 'kapso')",
        )
        .bind(id)
        .bind(&payload.texto)
        .bind(phone)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Error encolando mensaje para WhatsApp: {}", e)})),
            )
        })?;
    }

    tx.commit().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Error confirmando transacción: {}", e)})),
        )
    })?;

    // Despacho directo en tiempo real vía Kapso WhatsApp Cloud API
    let pool_clone = pool.clone();
    let texto_clone = payload.texto.clone();
    let b_id = id;
    let targets_clone = target_phones.clone();

    tokio::spawn(async move {
        let cfg = get_whatsapp_config(&pool_clone).await;
        if cfg.api_key.trim().is_empty() || cfg.phone_number_id.trim().is_empty() {
            tracing::warn!("KAPSO_API_KEY o PHONE_NUMBER_ID no configurados. Mensajes en cola marcados como pendientes de credenciales.");
            let _ = sqlx::query(
                "UPDATE mensajes_pendientes 
                 SET estado = 'error', error_mensaje = 'WhatsApp Cloud API / Kapso no configurado (Falta API Key o Phone ID)', proveedor = 'kapso'
                 WHERE referencia_id = $1 AND estado = 'pendiente'"
            )
            .bind(b_id)
            .execute(&pool_clone)
            .await;
            return;
        }

        let client = KapsoClient::new(cfg.api_key, cfg.phone_number_id);

        for phone in &targets_clone {
            if phone == "Sin destinatario configurado" || phone == "5215512345678" {
                continue;
            }

            match client.send_text(phone, &texto_clone).await {
                Ok(msg_id) => {
                    let _ = sqlx::query(
                        "UPDATE mensajes_pendientes 
                         SET estado = 'confirmado', proveedor = 'kapso', kapso_message_id = $1, meta_status = 'sent', confirmado_en = now()
                         WHERE referencia_id = $2 AND destinatario = $3"
                    )
                    .bind(&msg_id)
                    .bind(b_id)
                    .bind(phone)
                    .execute(&pool_clone)
                    .await;
                }
                Err(e) => {
                    let _ = sqlx::query(
                        "UPDATE mensajes_pendientes 
                         SET estado = 'error', error_mensaje = $1, proveedor = 'kapso'
                         WHERE referencia_id = $2 AND destinatario = $3"
                    )
                    .bind(&e)
                    .bind(b_id)
                    .bind(phone)
                    .execute(&pool_clone)
                    .await;
                }
            }
        }
    });

    Ok((
        StatusCode::OK,
        Json(json!({
            "mensaje": "Boletín aprobado y encolado para entrega vía WhatsApp",
            "boletin_id": id,
            "estado": "aprobado"
        })),
    ))
}

pub async fn procesar_boletin(
    State((pool, config)): State<(DbPool, Arc<Config>)>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let boletin = sqlx::query_as::<_, Boletin>(
        "SELECT id, fecha_boletin, nombre_archivo, ruta_archivo, subido_por, estado, 
                total_paginas, error_mensaje, creado_en, actualizado_en
         FROM boletines WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Error consultando boletín: {}", e)})),
        )
    })?
    .ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Boletín no encontrado"})),
        )
    })?;

    let worker_url = format!("{}/api/process-boletin", config.worker_base_url);
    let client = reqwest::Client::new();
    let b_id_str = boletin.id.to_string();
    let f_path = boletin.ruta_archivo.clone();

    tokio::spawn(async move {
        let payload = json!({
            "boletin_id": b_id_str,
            "ruta_archivo": f_path
        });
        if let Err(e) = client.post(&worker_url).json(&payload).send().await {
            tracing::error!("Error notificando a worker Python: {}", e);
        }
    });

    Ok((
        StatusCode::OK,
        Json(json!({
            "mensaje": "Procesamiento OCR y síntesis iniciado en el worker",
            "boletin_id": id
        })),
    ))
}

// ============================================================================
// Métodos para Síntesis Diarias Consolidadas (Monitoreo Ejecutivo por Fecha)
// ============================================================================

pub async fn ensure_sintesis_diarias_schema(pool: &DbPool) {
    let _ = sqlx::query("CREATE EXTENSION IF NOT EXISTS \"pgcrypto\"")
        .execute(pool)
        .await;

    if let Err(e) = sqlx::query(
        "CREATE TABLE IF NOT EXISTS sintesis_diarias (
            id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            fecha           DATE UNIQUE NOT NULL,
            texto           TEXT NOT NULL DEFAULT '',
            temas           TEXT[] DEFAULT '{}',
            documentos_ids  UUID[] DEFAULT '{}',
            estado          VARCHAR(30) NOT NULL DEFAULT 'borrador',
            modelo_usado    VARCHAR(50) DEFAULT 'claude-sonnet-4-5-20250929',
            tokens_usados   INT,
            aprobado_por    UUID,
            creado_en       TIMESTAMPTZ NOT NULL DEFAULT now(),
            actualizado_en  TIMESTAMPTZ NOT NULL DEFAULT now()
        )"
    )
    .execute(pool)
    .await {
        tracing::error!("Error asegurando tabla sintesis_diarias: {}", e);
    }

    if let Err(e) = sqlx::query("ALTER TABLE boletines ADD COLUMN IF NOT EXISTS incluido_en_sintesis BOOLEAN NOT NULL DEFAULT TRUE")
        .execute(pool)
        .await {
        tracing::warn!("Aviso columna incluido_en_sintesis: {}", e);
    }

    if let Err(e) = sqlx::query("ALTER TABLE boletines ADD COLUMN IF NOT EXISTS origen VARCHAR(50) NOT NULL DEFAULT 'manual'")
        .execute(pool)
        .await {
        tracing::warn!("Aviso columna origen: {}", e);
    }

    if let Err(e) = sqlx::query("ALTER TABLE sintesis_diarias ADD COLUMN IF NOT EXISTS temas TEXT[] DEFAULT '{}'")
        .execute(pool)
        .await {
        tracing::warn!("Aviso columna temas: {}", e);
    }

    let _ = sqlx::query("ALTER TABLE sintesis_diarias ADD COLUMN IF NOT EXISTS documentos_ids UUID[] DEFAULT '{}'")
        .execute(pool)
        .await;

    let _ = sqlx::query("ALTER TABLE sintesis_diarias ADD COLUMN IF NOT EXISTS estado VARCHAR(30) NOT NULL DEFAULT 'borrador'")
        .execute(pool)
        .await;

    let _ = sqlx::query("ALTER TABLE sintesis_diarias ADD COLUMN IF NOT EXISTS modelo_usado VARCHAR(50) DEFAULT 'claude-sonnet-4-5-20250929'")
        .execute(pool)
        .await;

    let _ = sqlx::query("ALTER TABLE sintesis_diarias ADD COLUMN IF NOT EXISTS tokens_usados INT")
        .execute(pool)
        .await;

    let _ = sqlx::query("ALTER TABLE sintesis_diarias ADD COLUMN IF NOT EXISTS aprobado_por UUID")
        .execute(pool)
        .await;
}

pub async fn list_fechas_sintesis(
    State((pool, _)): State<(DbPool, Arc<Config>)>,
    Query(query): Query<ListQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    ensure_sintesis_diarias_schema(&pool).await;
    let limit = query.limit.unwrap_or(30);
    let offset = query.offset.unwrap_or(0);

    #[derive(sqlx::FromRow)]
    struct ResumenDbRow {
        fecha: NaiveDate,
        total_documentos: i64,
        total_senado: i64,
        total_manual: i64,
        total_paginas: i64,
        sintesis_id: Option<Uuid>,
        estado_sintesis: Option<String>,
        texto_preview: Option<String>,
        modelo_usado: Option<String>,
        actualizado_en: Option<chrono::DateTime<Utc>>,
    }

    let rows = match sqlx::query_as::<_, ResumenDbRow>(
        "SELECT 
            b.fecha_boletin AS fecha,
            COUNT(b.id) AS total_documentos,
            COUNT(CASE WHEN b.origen = 'senado' THEN 1 END) AS total_senado,
            COUNT(CASE WHEN b.origen != 'senado' THEN 1 END) AS total_manual,
            COALESCE(SUM(b.total_paginas), 0) AS total_paginas,
            sd.id AS sintesis_id,
            COALESCE(sd.estado, 'pendiente') AS estado_sintesis,
            SUBSTRING(sd.texto, 1, 160) AS texto_preview,
            sd.modelo_usado,
            COALESCE(sd.actualizado_en, MAX(b.actualizado_en)) AS actualizado_en
         FROM boletines b
         LEFT JOIN sintesis_diarias sd ON sd.fecha = b.fecha_boletin
         GROUP BY b.fecha_boletin, sd.id, sd.estado, sd.texto, sd.modelo_usado, sd.actualizado_en
         ORDER BY b.fecha_boletin DESC
         LIMIT $1 OFFSET $2",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&pool)
    .await {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!("Aviso listando fechas con sintesis_diarias: {}. Reintentando con consulta base...", e);
            ensure_sintesis_diarias_schema(&pool).await;

            #[derive(sqlx::FromRow)]
            struct SimpleResumenRow {
                fecha: NaiveDate,
                total_documentos: i64,
                total_paginas: i64,
                actualizado_en: Option<chrono::DateTime<Utc>>,
            }

            let s_rows = sqlx::query_as::<_, SimpleResumenRow>(
                "SELECT 
                    fecha_boletin AS fecha,
                    COUNT(id) AS total_documentos,
                    COALESCE(SUM(total_paginas), 0) AS total_paginas,
                    MAX(actualizado_en) AS actualizado_en
                 FROM boletines
                 GROUP BY fecha_boletin
                 ORDER BY fecha_boletin DESC
                 LIMIT $1 OFFSET $2",
            )
            .bind(limit)
            .bind(offset)
            .fetch_all(&pool)
            .await
            .map_err(|err| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": format!("Error al obtener fechas de síntesis: {}", err)})),
                )
            })?;

            s_rows.into_iter().map(|r| ResumenDbRow {
                fecha: r.fecha,
                total_documentos: r.total_documentos,
                total_senado: 0,
                total_manual: r.total_documentos,
                total_paginas: r.total_paginas,
                sintesis_id: None,
                estado_sintesis: Some("pendiente".to_string()),
                texto_preview: None,
                modelo_usado: None,
                actualizado_en: r.actualizado_en,
            }).collect()
        }
    };

    let result: Vec<SintesisDiariaResumen> = rows
        .into_iter()
        .map(|r| SintesisDiariaResumen {
            fecha: r.fecha,
            total_documentos: r.total_documentos,
            total_senado: r.total_senado,
            total_manual: r.total_manual,
            total_paginas: r.total_paginas,
            sintesis_id: r.sintesis_id,
            estado_sintesis: r.estado_sintesis,
            texto_preview: r.texto_preview,
            modelo_usado: r.modelo_usado,
            actualizado_en: r.actualizado_en,
        })
        .collect();

    Ok((StatusCode::OK, Json(json!(result))))
}

pub async fn get_workspace_fecha(
    State((pool, _)): State<(DbPool, Arc<Config>)>,
    Path(fecha_str): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    ensure_sintesis_diarias_schema(&pool).await;

    let fecha = NaiveDate::parse_from_str(&fecha_str, "%Y-%m-%d").map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Formato de fecha inválido. Utilice YYYY-MM-DD"})),
        )
    })?;

    // 1. Obtener la síntesis diaria si ya existe (no abortar si la tabla aún se está creando)
    let sintesis = match sqlx::query_as::<_, SintesisDiaria>(
        "SELECT id, fecha, texto, temas, documentos_ids, estado, modelo_usado, tokens_usados, aprobado_por, creado_en, actualizado_en
         FROM sintesis_diarias
         WHERE fecha = $1",
    )
    .bind(fecha)
    .fetch_optional(&pool)
    .await {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!("Aviso consultando síntesis diaria para {}: {}. Intentando asegurar esquema...", fecha, e);
            ensure_sintesis_diarias_schema(&pool).await;
            None
        }
    };

    // 2. Obtener todos los boletines de esa fecha
    #[derive(sqlx::FromRow)]
    struct BoletinRow {
        id: Uuid,
        fecha_boletin: NaiveDate,
        nombre_archivo: String,
        ruta_archivo: String,
        estado: String,
        origen: Option<String>,
        incluido_en_sintesis: Option<bool>,
        total_paginas: Option<i32>,
        error_mensaje: Option<String>,
        creado_en: chrono::DateTime<Utc>,
    }

    let docs_rows = match sqlx::query_as::<_, BoletinRow>(
        "SELECT id, fecha_boletin, nombre_archivo, ruta_archivo, estado, origen, incluido_en_sintesis, total_paginas, error_mensaje, creado_en
         FROM boletines
         WHERE fecha_boletin = $1
         ORDER BY creado_en ASC",
    )
    .bind(fecha)
    .fetch_all(&pool)
    .await {
        Ok(rows) => rows,
        Err(e) => {
            tracing::warn!("Aviso consultando boletines con origen para {}: {}. Reintentando con consulta básica...", fecha, e);
            ensure_sintesis_diarias_schema(&pool).await;

            #[derive(sqlx::FromRow)]
            struct FallbackBoletinRow {
                id: Uuid,
                fecha_boletin: NaiveDate,
                nombre_archivo: String,
                ruta_archivo: String,
                estado: String,
                total_paginas: Option<i32>,
                error_mensaje: Option<String>,
                creado_en: chrono::DateTime<Utc>,
            }

            let fb_rows = sqlx::query_as::<_, FallbackBoletinRow>(
                "SELECT id, fecha_boletin, nombre_archivo, ruta_archivo, estado, total_paginas, error_mensaje, creado_en
                 FROM boletines
                 WHERE fecha_boletin = $1
                 ORDER BY creado_en ASC",
            )
            .bind(fecha)
            .fetch_all(&pool)
            .await
            .map_err(|err| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": format!("Error consultando documentos de la fecha: {}", err)})),
                )
            })?;

            fb_rows.into_iter().map(|r| BoletinRow {
                id: r.id,
                fecha_boletin: r.fecha_boletin,
                nombre_archivo: r.nombre_archivo,
                ruta_archivo: r.ruta_archivo,
                estado: r.estado,
                origen: Some("manual".to_string()),
                incluido_en_sintesis: Some(true),
                total_paginas: r.total_paginas,
                error_mensaje: r.error_mensaje,
                creado_en: r.creado_en,
            }).collect()
        }
    };

    let mut total_incluidos = 0;
    let documentos: Vec<BoletinEnWorkspace> = docs_rows
        .into_iter()
        .map(|r| {
            let inc = r.incluido_en_sintesis.unwrap_or(true);
            if inc {
                total_incluidos += 1;
            }
            BoletinEnWorkspace {
                id: r.id,
                fecha_boletin: r.fecha_boletin,
                nombre_archivo: r.nombre_archivo,
                ruta_archivo: r.ruta_archivo,
                estado: r.estado,
                origen: r.origen.unwrap_or_else(|| "manual".to_string()),
                incluido_en_sintesis: inc,
                total_paginas: r.total_paginas,
                error_mensaje: r.error_mensaje,
                creado_en: r.creado_en,
            }
        })
        .collect();

    let resp = WorkspaceFechaResponse {
        fecha,
        total_documentos: documentos.len(),
        documentos_incluidos: total_incluidos,
        documentos,
        sintesis,
    };

    Ok((StatusCode::OK, Json(json!(resp))))
}

pub async fn upload_multiple_boletines(
    State((pool, config)): State<(DbPool, Arc<Config>)>,
    Extension(auth_ctx): Extension<AuthContext>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let mut fecha_boletin: Option<NaiveDate> = None;
    let mut origen = "manual".to_string();
    let mut uploaded_files: Vec<(String, Vec<u8>)> = Vec::new();

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        error!("Error en multipart.next_field: {}", e);
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": format!("Error leyendo formulario multipart: {}", e)})),
        )
    })? {
        let name = field.name().unwrap_or("").to_string();

        if name == "fecha" || name == "fecha_boletin" {
            if let Ok(text) = field.text().await {
                if let Ok(d) = NaiveDate::parse_from_str(text.trim(), "%Y-%m-%d") {
                    fecha_boletin = Some(d);
                }
            }
        } else if name == "origen" {
            if let Ok(text) = field.text().await {
                let clean = text.trim();
                if !clean.is_empty() {
                    origen = clean.to_string();
                }
            }
        } else if name == "file" || name == "files" || name == "archivos" || name == "archivo" {
            let original_name = field
                .file_name()
                .unwrap_or("documento.pdf")
                .to_string();

            let base_name = std::path::Path::new(&original_name)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(&original_name)
                .to_string();

            if base_name.to_lowercase().ends_with(".pdf") {
                let bytes = field.bytes().await.map_err(|e| {
                    error!("Error leyendo bytes de {}: {}", base_name, e);
                    (
                        StatusCode::BAD_REQUEST,
                        Json(json!({"error": format!("Error leyendo bytes de {}: {}", base_name, e)})),
                    )
                })?;
                if !bytes.is_empty() {
                    uploaded_files.push((base_name, bytes.to_vec()));
                }
            } else {
                tracing::warn!("Archivo descartado por no tener extensión .pdf: {}", base_name);
            }
        }
    }

    if uploaded_files.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "No se recibieron archivos PDF válidos para subir (verifique que los archivos tengan extensión .pdf)"})),
        ));
    }

    // Por defecto usar la fecha local de CDMX (UTC-6) si no se especifica
    let fecha = fecha_boletin.unwrap_or_else(|| (Utc::now() - chrono::Duration::hours(6)).date_naive());
    let upload_dir = PathBuf::from(&config.upload_dir);
    if let Err(e) = fs::create_dir_all(&upload_dir) {
        error!("Error creando directorio de subidas: {}", e);
    }

    let mut creados = Vec::new();
    let client = reqwest::Client::new();
    let worker_url = format!("{}/api/process-boletin", config.worker_base_url);

    for (file_name, file_bytes) in uploaded_files {
        let boletin_id = Uuid::new_v4();
        let safe_file_name = format!("{}_{}", boletin_id, file_name.replace(" ", "_"));
        let target_path = upload_dir.join(&safe_file_name);

        match tokio::fs::File::create(&target_path).await {
            Ok(mut f) => {
                if let Err(e) = f.write_all(&file_bytes).await {
                    error!("Error escribiendo archivo {} en disco: {}", safe_file_name, e);
                }
            }
            Err(e) => {
                error!("Error creando archivo {} en disco: {}", target_path.display(), e);
            }
        }

        let target_path_abs = std::fs::canonicalize(&target_path).unwrap_or_else(|_| target_path.clone());
        let target_path_str = target_path_abs.to_string_lossy().to_string();

        let insert_res = match sqlx::query(
            "INSERT INTO boletines (id, fecha_boletin, nombre_archivo, ruta_archivo, subido_por, estado, origen, incluido_en_sintesis)
             VALUES ($1, $2, $3, $4, $5, 'pendiente_ocr', $6, TRUE)",
        )
        .bind(boletin_id)
        .bind(fecha)
        .bind(&file_name)
        .bind(&target_path_str)
        .bind(auth_ctx.user_id)
        .bind(&origen)
        .execute(&pool)
        .await {
            Ok(_) => Ok(()),
            Err(e) => {
                tracing::warn!("Aviso insertando con subido_por ({}): {}. Reintentando con subido_por = NULL...", auth_ctx.user_id, e);
                sqlx::query(
                    "INSERT INTO boletines (id, fecha_boletin, nombre_archivo, ruta_archivo, subido_por, estado, origen, incluido_en_sintesis)
                     VALUES ($1, $2, $3, $4, NULL, 'pendiente_ocr', $5, TRUE)",
                )
                .bind(boletin_id)
                .bind(fecha)
                .bind(&file_name)
                .bind(&target_path_str)
                .bind(&origen)
                .execute(&pool)
                .await
                .map(|_| ())
            }
        };

        match insert_res {
            Ok(_) => {
                info!("Boletín manual registrado: {} (ID: {}) para fecha {}", file_name, boletin_id, fecha);
                creados.push(json!({
                    "id": boletin_id,
                    "nombre_archivo": file_name,
                    "estado": "pendiente_ocr",
                    "origen": origen
                }));

                // Notificar worker Python en background
                let b_id_str = boletin_id.to_string();
                let f_path = target_path_str.clone();
                let w_url = worker_url.clone();
                let cli = client.clone();
                tokio::spawn(async move {
                    let resp = cli.post(&w_url).json(&json!({
                        "boletin_id": b_id_str,
                        "ruta_archivo": f_path
                    })).send().await;
                    if let Err(err) = resp {
                        tracing::error!("Error notificando a worker Python para {}: {}", b_id_str, err);
                    }
                });
            }
            Err(err) => {
                error!("Error definitivo insertando boletín {} en BD: {}", file_name, err);
            }
        }
    }

    if creados.is_empty() {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "No se pudo registrar ningún documento en la base de datos"})),
        ));
    }

    Ok((
        StatusCode::OK,
        Json(json!({
            "mensaje": format!("Se subieron exitosamente {} documentos para la fecha {}", creados.len(), fecha),
            "fecha": fecha,
            "documentos": creados
        })),
    ))
}

pub async fn toggle_documento_seleccion(
    State((pool, _)): State<(DbPool, Arc<Config>)>,
    Path(id): Path<Uuid>,
    Json(payload): Json<ToggleDocumentoSeleccionRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    ensure_sintesis_diarias_schema(&pool).await;

    sqlx::query(
        "UPDATE boletines SET incluido_en_sintesis = $1, actualizado_en = now() WHERE id = $2",
    )
    .bind(payload.incluido)
    .bind(id)
    .execute(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Error actualizando selección del documento: {}", e)})),
        )
    })?;

    Ok((
        StatusCode::OK,
        Json(json!({
            "id": id,
            "incluido_en_sintesis": payload.incluido
        })),
    ))
}

pub async fn consolidar_sintesis_fecha(
    State((pool, config)): State<(DbPool, Arc<Config>)>,
    Path(fecha_str): Path<String>,
    payload: Option<Json<ConsolidarSintesisFechaRequest>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    ensure_sintesis_diarias_schema(&pool).await;

    let worker_url = format!("{}/api/consolidar-sintesis-diaria", config.worker_base_url);
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());

    let doc_ids = payload.and_then(|p| p.0.documentos_ids);
    let request_body = json!({
        "fecha": fecha_str,
        "documentos_ids": doc_ids
    });

    info!("Solicitando consolidación de síntesis para {} a worker: {}", fecha_str, worker_url);

    let resp = client
        .post(&worker_url)
        .json(&request_body)
        .send()
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_GATEWAY,
                Json(json!({"error": format!("No se pudo conectar con el worker Python: {}", e)})),
            )
        })?;

    let status = resp.status();
    let body_json: serde_json::Value = resp.json().await.unwrap_or_else(|_| {
        json!({"error": "Respuesta no JSON del worker"})
    });

    if status.is_success() {
        Ok((StatusCode::OK, Json(body_json)))
    } else {
        let detalle = body_json.get("detail")
            .and_then(|d| d.as_str())
            .or_else(|| body_json.get("error").and_then(|e| e.as_str()))
            .unwrap_or("Error interno en worker");
        tracing::error!("Error devuelto por worker al consolidar síntesis de {}: {}", fecha_str, detalle);
        Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": format!("Error del worker al consolidar síntesis: {}", detalle),
                "detalles": body_json
            })),
        ))
    }
}

pub async fn actualizar_sintesis_diaria(
    State((pool, _)): State<(DbPool, Arc<Config>)>,
    Path(fecha_str): Path<String>,
    Json(payload): Json<ActualizarSintesisDiariaRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    ensure_sintesis_diarias_schema(&pool).await;

    let fecha = NaiveDate::parse_from_str(&fecha_str, "%Y-%m-%d").map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Formato de fecha inválido. Utilice YYYY-MM-DD"})),
        )
    })?;

    let sintesis = sqlx::query_as::<_, SintesisDiaria>(
        "INSERT INTO sintesis_diarias (fecha, texto, estado, actualizado_en)
         VALUES ($1, $2, 'borrador', now())
         ON CONFLICT (fecha) DO UPDATE
         SET texto = EXCLUDED.texto,
             actualizado_en = now()
         RETURNING id, fecha, texto, temas, documentos_ids, estado, modelo_usado, tokens_usados, aprobado_por, creado_en, actualizado_en",
    )
    .bind(fecha)
    .bind(&payload.texto)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Error actualizando síntesis diaria: {}", e)})),
        )
    })?;

    Ok((StatusCode::OK, Json(json!(sintesis))))
}

pub async fn aprobar_sintesis_diaria(
    State((pool, config)): State<(DbPool, Arc<Config>)>,
    Extension(auth_ctx): Extension<AuthContext>,
    Path(fecha_str): Path<String>,
    Json(payload): Json<AprobarSintesisDiariaRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    ensure_sintesis_diarias_schema(&pool).await;

    let fecha = NaiveDate::parse_from_str(&fecha_str, "%Y-%m-%d").map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Formato de fecha inválido. Utilice YYYY-MM-DD"})),
        )
    })?;

    let mut tx = pool.begin().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Error de base de datos: {}", e)})),
        )
    })?;

    // 1. Upsert en sintesis_diarias con estado 'aprobado'
    let sintesis = sqlx::query_as::<_, SintesisDiaria>(
        "INSERT INTO sintesis_diarias (fecha, texto, estado, aprobado_por, actualizado_en)
         VALUES ($1, $2, 'aprobado', $3, now())
         ON CONFLICT (fecha) DO UPDATE
         SET texto = EXCLUDED.texto,
             estado = 'aprobado',
             aprobado_por = EXCLUDED.aprobado_por,
             actualizado_en = now()
         RETURNING id, fecha, texto, temas, documentos_ids, estado, modelo_usado, tokens_usados, aprobado_por, creado_en, actualizado_en",
    )
    .bind(fecha)
    .bind(&payload.texto)
    .bind(auth_ctx.user_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Error aprobando síntesis diaria: {}", e)})),
        )
    })?;

    // 2. Marcar todos los boletines de esa fecha que están incluidos como 'aprobado'
    let _ = sqlx::query(
        "UPDATE boletines SET estado = 'aprobado', actualizado_en = now() WHERE fecha_boletin = $1 AND (incluido_en_sintesis = TRUE OR incluido_en_sintesis IS NULL)",
    )
    .bind(fecha)
    .execute(&mut *tx)
    .await;

    // 3. Encolar y despachar mensaje WhatsApp si enviar_whatsapp != Some(false)
    let enviar_ws = payload.enviar_whatsapp.unwrap_or(true);
    let mut target_phones: Vec<String> = Vec::new();

    if enviar_ws {
        use crate::services::kapso::get_whatsapp_config;
        let whatsapp_cfg = get_whatsapp_config(&pool).await;

        #[derive(sqlx::FromRow)]
        struct DestRow {
            telefono: String,
        }

        let destinatarios_db = sqlx::query_as::<_, DestRow>(
            "SELECT telefono FROM lista_distribucion WHERE activo = true AND telefono != '5215512345678' AND trim(telefono) != ''"
        )
        .fetch_all(&mut *tx)
        .await
        .unwrap_or_default();

        target_phones = if destinatarios_db.is_empty() {
            if !whatsapp_cfg.director_phone.trim().is_empty() && whatsapp_cfg.director_phone.trim() != "5215512345678" {
                vec![whatsapp_cfg.director_phone.trim().to_string()]
            } else if !config.director_whatsapp.trim().is_empty() && config.director_whatsapp.trim() != "5215512345678" {
                vec![config.director_whatsapp.trim().to_string()]
            } else {
                vec![]
            }
        } else {
            destinatarios_db
                .into_iter()
                .map(|d| d.telefono)
                .filter(|t| !t.trim().is_empty() && t.trim() != "5215512345678")
                .collect()
        };

        for phone in &target_phones {
            let _ = sqlx::query(
                "INSERT INTO mensajes_pendientes (tipo, referencia_id, texto, destinatario, estado, proveedor)
                 VALUES ('brief', $1, $2, $3, 'pendiente', 'kapso')",
            )
            .bind(sintesis.id)
            .bind(&payload.texto)
            .bind(phone)
            .execute(&mut *tx)
            .await;
        }
    }

    tx.commit().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Error confirmando transacción: {}", e)})),
        )
    })?;

    // Despacho vía WhatsApp en background
    if enviar_ws && !target_phones.is_empty() {
        let pool_clone = pool.clone();
        let texto_clone = payload.texto.clone();
        let s_id = sintesis.id;
        let targets_clone = target_phones.clone();

        tokio::spawn(async move {
            use crate::services::kapso::{get_whatsapp_config, KapsoClient};
            let cfg = get_whatsapp_config(&pool_clone).await;
            if cfg.api_key.trim().is_empty() || cfg.phone_number_id.trim().is_empty() {
                tracing::warn!("Kapso API no configurada para despacho de síntesis consolidada");
                let _ = sqlx::query(
                    "UPDATE mensajes_pendientes SET estado = 'error', error_mensaje = 'Falta API Key o Phone ID' WHERE referencia_id = $1"
                )
                .bind(s_id)
                .execute(&pool_clone)
                .await;
                return;
            }

            let client = KapsoClient::new(cfg.api_key, cfg.phone_number_id);
            for phone in &targets_clone {
                if phone == "Sin destinatario configurado" || phone == "5215512345678" {
                    continue;
                }
                match client.send_text(phone, &texto_clone).await {
                    Ok(msg_id) => {
                        let _ = sqlx::query(
                            "UPDATE mensajes_pendientes SET estado = 'confirmado', kapso_message_id = $1, meta_status = 'sent', confirmado_en = now() WHERE referencia_id = $2 AND destinatario = $3"
                        )
                        .bind(&msg_id)
                        .bind(s_id)
                        .bind(phone)
                        .execute(&pool_clone)
                        .await;
                    }
                    Err(e) => {
                        let _ = sqlx::query(
                            "UPDATE mensajes_pendientes SET estado = 'error', error_mensaje = $1 WHERE referencia_id = $2 AND destinatario = $3"
                        )
                        .bind(&e)
                        .bind(s_id)
                        .bind(phone)
                        .execute(&pool_clone)
                        .await;
                    }
                }
            }
        });
    }

    Ok((
        StatusCode::OK,
        Json(json!({
            "mensaje": "Síntesis diaria consolidada aprobada exitosamente",
            "sintesis": sintesis,
            "destinatarios_notificados": target_phones.len()
        })),
    ))
}

pub async fn eliminar_boletin(
    State((pool, _)): State<(DbPool, Arc<Config>)>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    ensure_sintesis_diarias_schema(&pool).await;

    #[derive(sqlx::FromRow)]
    struct DocInfo {
        fecha_boletin: NaiveDate,
        nombre_archivo: String,
        ruta_archivo: String,
    }

    let doc = match sqlx::query_as::<_, DocInfo>(
        "SELECT fecha_boletin, nombre_archivo, ruta_archivo FROM boletines WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&pool)
    .await
    {
        Ok(Some(d)) => d,
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(json!({"error": "Documento no encontrado"})),
            ));
        }
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Error consultando documento: {}", e)})),
            ));
        }
    };

    // 1. Eliminar de la base de datos (secciones y sintesis_generadas tienen ON DELETE CASCADE)
    let res = sqlx::query("DELETE FROM boletines WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Error eliminando documento de base de datos: {}", e)})),
            )
        })?;

    // 2. Remover ID del array documentos_ids en sintesis_diarias si existe
    let _ = sqlx::query(
        "UPDATE sintesis_diarias SET documentos_ids = array_remove(documentos_ids, $1), actualizado_en = now() WHERE fecha = $2",
    )
    .bind(id)
    .bind(doc.fecha_boletin)
    .execute(&pool)
    .await;

    // 3. Eliminar archivo físico de disco si existe
    let _ = tokio::fs::remove_file(&doc.ruta_archivo).await;

    info!(
        "Documento '{}' ({}) de la fecha {} eliminado exitosamente",
        doc.nombre_archivo, id, doc.fecha_boletin
    );

    Ok((
        StatusCode::OK,
        Json(json!({
            "mensaje": format!("Documento '{}' eliminado exitosamente", doc.nombre_archivo),
            "id": id,
            "nombre_archivo": doc.nombre_archivo,
            "fecha": doc.fecha_boletin,
            "filas_afectadas": res.rows_affected()
        })),
    ))
}

#[derive(Debug, Deserialize, Default)]
pub struct DeleteSintesisQuery {
    pub eliminar_documentos: Option<bool>,
}

pub async fn eliminar_sintesis_diaria(
    State((pool, _)): State<(DbPool, Arc<Config>)>,
    Path(fecha_str): Path<String>,
    Query(params): Query<DeleteSintesisQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    ensure_sintesis_diarias_schema(&pool).await;

    let fecha = NaiveDate::parse_from_str(&fecha_str, "%Y-%m-%d").map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Formato de fecha inválido. Utilice YYYY-MM-DD"})),
        )
    })?;

    let res = sqlx::query("DELETE FROM sintesis_diarias WHERE fecha = $1")
        .bind(fecha)
        .execute(&pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Error eliminando síntesis diaria: {}", e)})),
            )
        })?;

    let mut docs_eliminados = 0;

    // Si se solicitó eliminar también los documentos asociados (para limpiar el día por completo)
    if params.eliminar_documentos.unwrap_or(false) {
        #[derive(sqlx::FromRow)]
        struct FilePathRow {
            ruta_archivo: String,
        }

        let files = sqlx::query_as::<_, FilePathRow>(
            "SELECT ruta_archivo FROM boletines WHERE fecha_boletin = $1",
        )
        .bind(fecha)
        .fetch_all(&pool)
        .await
        .unwrap_or_default();

        docs_eliminados = files.len();

        for f in files {
            let _ = tokio::fs::remove_file(&f.ruta_archivo).await;
        }

        let _ = sqlx::query("DELETE FROM boletines WHERE fecha_boletin = $1")
            .bind(fecha)
            .execute(&pool)
            .await;
    } else {
        // Revertir estado de los boletines que estaban en 'aprobado' para esa fecha a 'ocr_completo'
        let _ = sqlx::query("UPDATE boletines SET estado = 'ocr_completo' WHERE fecha_boletin = $1 AND estado = 'aprobado'")
            .bind(fecha)
            .execute(&pool)
            .await;
    }

    Ok((
        StatusCode::OK,
        Json(json!({
            "mensaje": if params.eliminar_documentos.unwrap_or(false) {
                format!("Fecha {} y sus {} documentos eliminados correctamente", fecha, docs_eliminados)
            } else {
                format!("Síntesis diaria del {} eliminada exitosamente", fecha)
            },
            "filas_afectadas": res.rows_affected(),
            "documentos_eliminados": docs_eliminados,
            "fecha": fecha
        })),
    ))
}


