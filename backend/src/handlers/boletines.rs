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
    models::{AprobarBoletinRequest, Boletin, BoletinDetailResponse, Seccion, SintesisGenerada, UpdateSintesisRequest},
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

    let fecha = fecha_boletin.unwrap_or_else(|| Utc::now().date_naive());
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

    // 2. Marcar boletín como aprobado
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

    // 3. Encolar en mensajes_pendientes para que n8n / WAHA lo recoja
    sqlx::query(
        "INSERT INTO mensajes_pendientes (tipo, referencia_id, texto, destinatario, estado)
         VALUES ('brief', $1, $2, $3, 'pendiente')",
    )
    .bind(id)
    .bind(&payload.texto)
    .bind(&config.director_whatsapp)
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Error encolando mensaje para WhatsApp: {}", e)})),
        )
    })?;

    tx.commit().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Error confirmando transacción: {}", e)})),
        )
    })?;

    // Despacho en tiempo real vía Kapso si está configurado
    let pool_clone = pool.clone();
    let texto_clone = payload.texto.clone();
    let b_id = id;

    tokio::spawn(async move {
        use crate::services::kapso::{get_whatsapp_config, KapsoClient};
        let cfg = get_whatsapp_config(&pool_clone).await;
        if cfg.provider == "kapso" && !cfg.api_key.trim().is_empty() && !cfg.phone_number_id.trim().is_empty() {
            let client = KapsoClient::new(cfg.api_key, cfg.phone_number_id);
            match client.send_text(&cfg.director_phone, &texto_clone).await {
                Ok(msg_id) => {
                    let _ = sqlx::query(
                        "UPDATE mensajes_pendientes 
                         SET estado = 'confirmado', proveedor = 'kapso', kapso_message_id = $1, meta_status = 'sent', confirmado_en = now()
                         WHERE referencia_id = $2"
                    )
                    .bind(&msg_id)
                    .bind(b_id)
                    .execute(&pool_clone)
                    .await;
                }
                Err(e) => {
                    let _ = sqlx::query(
                        "UPDATE mensajes_pendientes 
                         SET estado = 'error', error_mensaje = $1, proveedor = 'kapso'
                         WHERE referencia_id = $2"
                    )
                    .bind(e)
                    .bind(b_id)
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
