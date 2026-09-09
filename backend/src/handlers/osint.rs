use axum::{
    extract::{Extension, Path, Query, State},
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
    middleware::auth::AuthContext,
    models::{AlertaOsint, CreateEntidadRequest, EntidadVigilada, FuenteOsint, ValidarAlertaRequest},
};

#[derive(Debug, Deserialize)]
pub struct AlertasFilter {
    pub estado: Option<String>,
    pub severidad: Option<String>,
}

pub async fn list_alertas(
    Extension(auth_ctx): Extension<AuthContext>,
    State((pool, _)): State<(DbPool, Arc<Config>)>,
    Query(filter): Query<AlertasFilter>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let mut query = String::from(
        "SELECT id, entidad_id, fuente, titulo, descripcion, severidad, evidencia, 
                estado, validada_por, notas_analista, creado_en, validada_en
         FROM alertas_osint WHERE 1=1 ",
    );

    if let Some(estado) = &filter.estado {
        query.push_str(&format!("AND estado = '{}' ", estado.replace('\'', "")));
    }
    if let Some(sev) = &filter.severidad {
        query.push_str(&format!("AND severidad = '{}' ", sev.replace('\'', "")));
    }

    // Si el rol es 'director', solo debe ver alertas que hayan sido validadas por un analista
    if auth_ctx.rol == "director" {
        query.push_str("AND estado = 'validada' ");
    }

    query.push_str("ORDER BY creado_en DESC LIMIT 100");

    let mut alertas = sqlx::query_as::<_, AlertaOsint>(&query)
        .fetch_all(&pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Error consultando alertas: {}", e)})),
            )
        })?;

    // Regla de seguridad: Si el rol es 'director', sanitizar el campo de evidencia cruda (no exponer dumps/credenciales)
    if auth_ctx.rol == "director" {
        for a in &mut alertas {
            if let Some(ev) = &mut a.evidencia {
                if let Some(obj) = ev.as_object_mut() {
                    obj.remove("raw_dump");
                    obj.remove("passwords");
                    obj.remove("leaked_credentials");
                    obj.insert(
                        "resumen_seguridad".to_string(),
                        json!("Detalles técnicos y evidencia resguardados bajo control del analista."),
                    );
                }
            }
        }
    }

    Ok((StatusCode::OK, Json(alertas)))
}

/// Endpoint para que el analista valide o descarte una alerta detectada
pub async fn validar_alerta(
    Extension(auth_ctx): Extension<AuthContext>,
    State((pool, config)): State<(DbPool, Arc<Config>)>,
    Path(id): Path<Uuid>,
    Json(payload): Json<ValidarAlertaRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    if auth_ctx.rol == "director" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({"error": "El rol director no tiene permisos para validar o descartar alertas"})),
        ));
    }

    let nuevo_estado = match payload.accion.as_str() {
        "validar" => "validada",
        "descartar" => "descartada",
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "Acción no reconocida. Use 'validar' o 'descartar'"})),
            ))
        }
    };

    let now = Utc::now();
    let mut tx = pool.begin().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Error de base de datos: {}", e)})),
        )
    })?;

    let alerta = sqlx::query_as::<_, AlertaOsint>(
        "UPDATE alertas_osint
         SET estado = $1, validada_por = $2, notas_analista = $3, validada_en = $4
         WHERE id = $5
         RETURNING id, entidad_id, fuente, titulo, descripcion, severidad, evidencia, 
                   estado, validada_por, notas_analista, creado_en, validada_en",
    )
    .bind(nuevo_estado)
    .bind(auth_ctx.user_id)
    .bind(&payload.notas)
    .bind(now)
    .bind(id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Error actualizando alerta: {}", e)})),
        )
    })?
    .ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Alerta no encontrada"})),
        )
    })?;

    // Si fue validada por el analista, se inserta inmediatamente en la cola de mensajes pendientes para WAHA
    if nuevo_estado == "validada" {
        let mensaje_texto = format!(
            "🚨 *ALERTA URGENTE DE EXPOSICIÓN — ExposureIQ*\n\n\
             *Severidad:* {}\n\
             *Fuente:* {}\n\
             *Hallazgo:* {}\n\n\
             *Descripción:* {}\n\n\
             _Alerta validada por el equipo de análisis de seguridad._",
            alerta.severidad.to_uppercase(),
            alerta.fuente,
            alerta.titulo,
            alerta.descripcion
        );

        sqlx::query(
            "INSERT INTO mensajes_pendientes (tipo, referencia_id, texto, destinatario, estado)
             VALUES ('alerta', $1, $2, $3, 'pendiente')",
        )
        .bind(alerta.id)
        .bind(&mensaje_texto)
        .bind(&config.director_whatsapp)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Error encolando alerta en mensajes: {}", e)})),
            )
        })?;
    }

    tx.commit().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Error confirmando transacción: {}", e)})),
        )
    })?;

    Ok((
        StatusCode::OK,
        Json(json!({
            "mensaje": format!("Alerta marcada como {}", nuevo_estado),
            "alerta": alerta
        })),
    ))
}

// ============================================================================
// CRUD de Entidades Vigiladas
// ============================================================================

pub async fn list_entidades(
    State((pool, _)): State<(DbPool, Arc<Config>)>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let entidades = sqlx::query_as::<_, EntidadVigilada>(
        "SELECT id, tipo, valor, descripcion, activo, creado_en, actualizado_en
         FROM entidades_vigiladas
         ORDER BY creado_en DESC",
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Error consultando entidades: {}", e)})),
        )
    })?;

    Ok((StatusCode::OK, Json(entidades)))
}

pub async fn create_entidad(
    Extension(auth_ctx): Extension<AuthContext>,
    State((pool, _)): State<(DbPool, Arc<Config>)>,
    Json(payload): Json<CreateEntidadRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    if auth_ctx.rol != "admin" && auth_ctx.rol != "analista" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({"error": "Permiso denegado para crear entidades"})),
        ));
    }

    let entidad = sqlx::query_as::<_, EntidadVigilada>(
        "INSERT INTO entidades_vigiladas (tipo, valor, descripcion, activo)
         VALUES ($1, $2, $3, true)
         RETURNING id, tipo, valor, descripcion, activo, creado_en, actualizado_en",
    )
    .bind(&payload.tipo)
    .bind(&payload.valor)
    .bind(&payload.descripcion)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Error guardando entidad: {}", e)})),
        )
    })?;

    Ok((StatusCode::CREATED, Json(entidad)))
}

pub async fn toggle_entidad(
    Extension(auth_ctx): Extension<AuthContext>,
    State((pool, _)): State<(DbPool, Arc<Config>)>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    if auth_ctx.rol != "admin" && auth_ctx.rol != "analista" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({"error": "Permiso denegado"})),
        ));
    }

    let entidad = sqlx::query_as::<_, EntidadVigilada>(
        "UPDATE entidades_vigiladas
         SET activo = NOT activo, actualizado_en = now()
         WHERE id = $1
         RETURNING id, tipo, valor, descripcion, activo, creado_en, actualizado_en",
    )
    .bind(id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Error actualizando entidad: {}", e)})),
        )
    })?
    .ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Entidad no encontrada"})),
        )
    })?;

    Ok((StatusCode::OK, Json(entidad)))
}

pub async fn delete_entidad(
    Extension(auth_ctx): Extension<AuthContext>,
    State((pool, _)): State<(DbPool, Arc<Config>)>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    if auth_ctx.rol != "admin" && auth_ctx.rol != "analista" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({"error": "Permiso denegado para eliminar entidades"})),
        ));
    }

    let affected = sqlx::query("DELETE FROM entidades_vigiladas WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Error eliminando entidad: {}", e)})),
            )
        })?
        .rows_affected();

    if affected == 0 {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Entidad no encontrada"})),
        ));
    }

    Ok((
        StatusCode::OK,
        Json(json!({
            "mensaje": "Entidad eliminada del catálogo exitosamente",
            "id": id
        })),
    ))
}

// ============================================================================
// CRUD de Fuentes OSINT (world-intel-mcp)
// ============================================================================

pub async fn list_fuentes_osint(
    State((pool, _)): State<(DbPool, Arc<Config>)>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let fuentes = sqlx::query_as::<_, FuenteOsint>(
        "SELECT id, clave, nombre, descripcion, tipo, activo, ultimo_escaneo, total_hallazgos, creado_en, actualizado_en
         FROM fuentes_osint
         ORDER BY creado_en ASC",
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Error consultando fuentes OSINT: {}", e)})),
        )
    })?;

    Ok((StatusCode::OK, Json(fuentes)))
}

pub async fn toggle_fuente_osint(
    Extension(auth_ctx): Extension<AuthContext>,
    State((pool, _)): State<(DbPool, Arc<Config>)>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    if auth_ctx.rol != "admin" && auth_ctx.rol != "analista" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({"error": "Permiso denegado para modificar fuentes OSINT"})),
        ));
    }

    let fuente = sqlx::query_as::<_, FuenteOsint>(
        "UPDATE fuentes_osint
         SET activo = NOT activo, actualizado_en = now()
         WHERE id = $1
         RETURNING id, clave, nombre, descripcion, tipo, activo, ultimo_escaneo, total_hallazgos, creado_en, actualizado_en",
    )
    .bind(id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Error actualizando fuente OSINT: {}", e)})),
        )
    })?
    .ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Fuente OSINT no encontrada"})),
        )
    })?;

    Ok((StatusCode::OK, Json(fuente)))
}
