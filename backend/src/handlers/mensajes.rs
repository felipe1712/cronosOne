use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    config::Config,
    db::DbPool,
    models::{ConfirmarMensajeRequest, MensajePendiente},
};

#[derive(Debug, Deserialize)]
pub struct PendientesQuery {
    /// Si es true, marca automáticamente el mensaje retornado como 'entregado_a_n8n'
    pub auto_lock: Option<bool>,
    /// Cantidad máxima de mensajes a traer (default 1 para procesamiento secuencial de WhatsApp)
    pub limit: Option<i64>,
}

/// Endpoint consumido periódicamente por el cron de n8n
/// Retorna el siguiente mensaje listo en la cola (brief diario o alerta validada).
/// Si no hay nada pendiente, retorna una lista vacía `[]` con status 200.
pub async fn get_mensajes_pendientes(
    State((pool, _)): State<(DbPool, Arc<Config>)>,
    Query(query): Query<PendientesQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let limit = query.limit.unwrap_or(1);
    let auto_lock = query.auto_lock.unwrap_or(true);

    let mut tx = pool.begin().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Error de transacción: {}", e)})),
        )
    })?;

    // Selecciona los mensajes pendientes más antiguos primero (FIFO con prioridad implícita por creación)
    // Se ordenan alertas con mayor prioridad si comparten timestamp
    let mensajes = sqlx::query_as::<_, MensajePendiente>(
        "SELECT id, tipo, referencia_id, texto, destinatario, estado, 
                intento_conteo, error_mensaje, creado_en, entregado_en, confirmado_en
         FROM mensajes_pendientes
         WHERE estado = 'pendiente'
         ORDER BY (CASE WHEN tipo = 'alerta' THEN 0 ELSE 1 END) ASC, creado_en ASC
         LIMIT $1
         FOR UPDATE SKIP LOCKED",
    )
    .bind(limit)
    .fetch_all(&mut *tx)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Error consultando mensajes: {}", e)})),
        )
    })?;

    if auto_lock && !mensajes.is_empty() {
        let now = Utc::now();
        for m in &mensajes {
            sqlx::query(
                "UPDATE mensajes_pendientes 
                 SET estado = 'entregado_a_n8n', entregado_en = $1, intento_conteo = intento_conteo + 1
                 WHERE id = $2",
            )
            .bind(now)
            .bind(m.id)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": format!("Error actualizando estado de mensaje: {}", e)})),
                )
            })?;
        }
    }

    tx.commit().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Error haciendo commit: {}", e)})),
        )
    })?;

    Ok((StatusCode::OK, Json(mensajes)))
}

/// Endpoint para que n8n confirme la entrega exitosa del mensaje tras llamar a WAHA
pub async fn confirmar_mensaje(
    State((pool, _)): State<(DbPool, Arc<Config>)>,
    Path(id): Path<Uuid>,
    Json(payload): Json<ConfirmarMensajeRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let now = Utc::now();
    let nuevo_estado = if payload.estado == "confirmado" {
        "confirmado"
    } else {
        "error"
    };

    let result = sqlx::query(
        "UPDATE mensajes_pendientes
         SET estado = $1, confirmado_en = $2, error_mensaje = $3
         WHERE id = $4",
    )
    .bind(nuevo_estado)
    .bind(now)
    .bind(&payload.error_mensaje)
    .bind(id)
    .execute(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Error actualizando confirmación: {}", e)})),
        )
    })?;

    if result.rows_affected() == 0 {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Mensaje no encontrado"})),
        ));
    }

    Ok((
        StatusCode::OK,
        Json(json!({
            "mensaje": "Estado de mensaje actualizado correctamente",
            "id": id,
            "estado": nuevo_estado
        })),
    ))
}

/// Listado histórico de mensajes para el panel web
pub async fn list_historial_mensajes(
    State((pool, _)): State<(DbPool, Arc<Config>)>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let mensajes = sqlx::query_as::<_, MensajePendiente>(
        "SELECT id, tipo, referencia_id, texto, destinatario, estado, 
                intento_conteo, error_mensaje, creado_en, entregado_en, confirmado_en
         FROM mensajes_pendientes
         ORDER BY creado_en DESC
         LIMIT 50",
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Error consultando historial: {}", e)})),
        )
    })?;

    Ok((StatusCode::OK, Json(mensajes)))
}
