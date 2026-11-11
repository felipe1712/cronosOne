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
        Extracto, GuardarExtractoRequest, ResumirExtractoRequest,
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
    struct SubRow {
        phone_number: String,
    }

    // Obtener destinatarios de la lista "Audiencia WhatsApp" del CMS (subscribers activos)
    let subscribers_db = sqlx::query_as::<_, SubRow>(
        "SELECT phone_number FROM subscribers WHERE consent_status = 'subscribed' AND trim(phone_number) != '' AND phone_number NOT LIKE '%5512345678' ORDER BY created_at ASC"
    )
    .fetch_all(&mut *tx)
    .await
    .unwrap_or_default();

    let mut target_phones: Vec<String> = if subscribers_db.is_empty() {
        if !whatsapp_cfg.director_phone.trim().is_empty() && whatsapp_cfg.director_phone.trim() != "5215512345678" {
            vec![whatsapp_cfg.director_phone.trim().to_string()]
        } else if !config.director_whatsapp.trim().is_empty() && config.director_whatsapp.trim() != "5215512345678" {
            vec![config.director_whatsapp.trim().to_string()]
        } else {
            vec![]
        }
    } else {
        subscribers_db
            .into_iter()
            .map(|s| s.phone_number)
            .filter(|p| !p.trim().is_empty() && !p.contains("5512345678"))
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

        let fecha_b: NaiveDate = sqlx::query_scalar("SELECT fecha_boletin FROM boletines WHERE id = $1")
            .bind(b_id)
            .fetch_optional(&pool_clone)
            .await
            .unwrap_or(None)
            .unwrap_or_else(|| chrono::Utc::now().date_naive());

        let clean_snippet: String = texto_clone.chars().take(900).collect();

        for phone in &targets_clone {
            if phone == "Sin destinatario configurado" || phone == "5215512345678" {
                continue;
            }

            // Exigir exclusivamente plantilla oficial aprobada por Meta (type: template)
            let send_res = client.send_brief_template(
                Some(&pool_clone),
                phone,
                fecha_b,
                &clean_snippet,
                "envio_boletin_plantilla",
            ).await;

            match send_res {
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

    // Asegurar columnas requeridas en boletines de forma no bloqueante (evita AccessExclusiveLock)
    let _ = sqlx::query(
        "DO $$
        BEGIN
            IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'boletines' AND column_name = 'incluido_en_sintesis') THEN
                ALTER TABLE boletines ADD COLUMN incluido_en_sintesis BOOLEAN NOT NULL DEFAULT TRUE;
            END IF;
            IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'boletines' AND column_name = 'origen') THEN
                ALTER TABLE boletines ADD COLUMN origen VARCHAR(50) NOT NULL DEFAULT 'manual';
            END IF;
        END $$;"
    )
    .execute(pool)
    .await;

    // Asegurar tabla extractos para WhatsApp
    let _ = sqlx::query(
        "CREATE TABLE IF NOT EXISTS extractos (
            id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            sintesis_id     UUID REFERENCES sintesis_diarias(id) ON DELETE CASCADE,
            fecha           DATE UNIQUE NOT NULL,
            texto           TEXT NOT NULL CHECK (char_length(texto) <= 1024),
            origen          VARCHAR(20) NOT NULL DEFAULT 'corte'
                            CHECK (origen IN ('corte', 'claude', 'manual')),
            modelo_usado    VARCHAR(80),
            tokens_usados   INT,
            editado_por     UUID REFERENCES usuarios(id) ON DELETE SET NULL,
            creado_en       TIMESTAMPTZ NOT NULL DEFAULT now(),
            actualizado_en  TIMESTAMPTZ NOT NULL DEFAULT now()
        )"
    )
    .execute(pool)
    .await;

    let _ = sqlx::query("CREATE INDEX IF NOT EXISTS idx_extractos_fecha ON extractos (fecha DESC)")
        .execute(pool)
        .await;

    // Asegurar página de Primeras Planas en el CMS
    let _ = sqlx::query(
        "INSERT INTO news_pages (section_key, title, subtitle, color, icon, orden, active)
         VALUES ('primeras_planas', 'Primeras Planas', 'Lo más relevante de las portadas nacionales del día', '#7c3aed', 'newspaper', 0, true)
         ON CONFLICT (section_key) DO NOTHING"
    )
    .execute(pool)
    .await;

    // Asegurar tabla whatsapp_plantillas y plantilla activa por defecto
    let _ = sqlx::query(
        "CREATE TABLE IF NOT EXISTS whatsapp_plantillas (
            id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            nombre              VARCHAR(512) NOT NULL,
            idioma              VARCHAR(10) NOT NULL DEFAULT 'es_MX',
            activa              BOOLEAN NOT NULL DEFAULT false,
            texto_fijo_body     TEXT NOT NULL DEFAULT '',
            tope_extracto       INT NOT NULL DEFAULT 900 CHECK (tope_extracto BETWEEN 1 AND 1024),
            parametros          JSONB NOT NULL DEFAULT '[]',
            kapso_template_id   VARCHAR(100),
            creado_en           TIMESTAMPTZ NOT NULL DEFAULT now(),
            actualizado_en      TIMESTAMPTZ NOT NULL DEFAULT now(),
            CONSTRAINT uq_plantilla_nombre_idioma UNIQUE (nombre, idioma)
        )"
    )
    .execute(pool)
    .await;

    let _ = sqlx::query("CREATE UNIQUE INDEX IF NOT EXISTS uq_plantilla_activa ON whatsapp_plantillas (activa) WHERE activa = true")
        .execute(pool)
        .await;

    let _ = sqlx::query(
        "UPDATE whatsapp_plantillas
         SET nombre = 'brief_informativo_cp_v2', actualizado_en = now()
         WHERE nombre = 'brief_informativo'
           AND NOT EXISTS (SELECT 1 FROM whatsapp_plantillas WHERE nombre = 'brief_informativo_cp_v2')"
    )
    .execute(pool)
    .await;

    let _ = sqlx::query(
        "UPDATE whatsapp_plantillas SET activa = false WHERE nombre != 'brief_informativo_cp_v2' AND activa = true"
    )
    .execute(pool)
    .await;

    let _ = sqlx::query(
        "INSERT INTO whatsapp_plantillas (nombre, idioma, activa, texto_fijo_body, tope_extracto, parametros)
         VALUES ('brief_informativo_cp_v2', 'es_MX', true, 'PRIMERAS PLANAS \\n\\n {{primeras_planas}} .\\n\\n Si no deseas seguir recibiendo estos mensajes manda la palabra \"STOP\"', 900,
         '[{\"componente\": \"header\", \"nombre\": \"date\", \"fuente\": \"fecha\", \"formato\": \"D de Mes\"}, {\"componente\": \"body\", \"nombre\": \"primeras_planas\", \"fuente\": \"extracto\"}]'::jsonb)
         ON CONFLICT (nombre, idioma) DO UPDATE SET activa = true, actualizado_en = now()"
    )
    .execute(pool)
    .await;
}

pub async fn list_fechas_sintesis(
    State((pool, _)): State<(DbPool, Arc<Config>)>,
    Query(query): Query<ListQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
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
            tracing::warn!("Aviso consultando síntesis diaria completa para {}: {}. Intentando consulta básica...", fecha, e);

            #[derive(sqlx::FromRow)]
            struct BasicSintesis {
                id: Uuid,
                fecha: NaiveDate,
                texto: String,
                estado: String,
                modelo_usado: Option<String>,
                tokens_usados: Option<i32>,
                creado_en: chrono::DateTime<Utc>,
                actualizado_en: chrono::DateTime<Utc>,
            }

            if let Ok(Some(b)) = sqlx::query_as::<_, BasicSintesis>(
                "SELECT id, fecha, texto, estado, modelo_usado, tokens_usados, creado_en, actualizado_en
                 FROM sintesis_diarias
                 WHERE fecha = $1",
            )
            .bind(fecha)
            .fetch_optional(&pool)
            .await {
                Some(SintesisDiaria {
                    id: b.id,
                    fecha: b.fecha,
                    texto: b.texto,
                    temas: Some(vec![]),
                    documentos_ids: Some(vec![]),
                    estado: b.estado,
                    modelo_usado: b.modelo_usado,
                    tokens_usados: b.tokens_usados,
                    aprobado_por: None,
                    creado_en: b.creado_en,
                    actualizado_en: b.actualizado_en,
                })
            } else {
                None
            }
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

    let extracto = sqlx::query_as::<_, Extracto>(
        "SELECT id, sintesis_id, fecha, texto, origen, modelo_usado, tokens_usados, editado_por, creado_en, actualizado_en
         FROM extractos WHERE fecha = $1"
    )
    .bind(fecha)
    .fetch_optional(&pool)
    .await
    .unwrap_or(None);

    let resp = WorkspaceFechaResponse {
        fecha,
        total_documentos: documentos.len(),
        documentos_incluidos: total_incluidos,
        documentos,
        sintesis,
        extracto,
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
    State((_pool, config)): State<(DbPool, Arc<Config>)>,
    Path(fecha_str): Path<String>,
    payload: Option<Json<ConsolidarSintesisFechaRequest>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
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
    Extension(auth_ctx): Extension<AuthContext>,
    Path(fecha_str): Path<String>,
    Json(payload): Json<ActualizarSintesisDiariaRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
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

    if let Some(ref ext) = payload.extracto {
        let clean_ext = ext.replace("\r\n", "\n").trim().to_string();
        let safe_ext: String = clean_ext.chars().take(1024).collect();
        if !safe_ext.is_empty() {
            let _ = sqlx::query(
                "INSERT INTO extractos (sintesis_id, fecha, texto, origen, editado_por, actualizado_en)
                 VALUES ($1, $2, $3, 'manual', $4, now())
                 ON CONFLICT (fecha) DO UPDATE
                 SET texto = EXCLUDED.texto,
                     origen = 'manual',
                     editado_por = EXCLUDED.editado_por,
                     actualizado_en = now()"
            )
            .bind(sintesis.id)
            .bind(fecha)
            .bind(&safe_ext)
            .bind(auth_ctx.user_id)
            .execute(&pool)
            .await;
        }
    }

    sync_sintesis_to_cms(&pool, fecha, &payload.texto).await;

    Ok((StatusCode::OK, Json(json!(sintesis))))
}

async fn sync_sintesis_to_cms(pool: &DbPool, fecha: NaiveDate, texto: &str) {
    if texto.trim().is_empty() {
        return;
    }

    let ed_id: Option<Uuid> = sqlx::query_scalar(
        "INSERT INTO news_editions (edition_date, status, generated_at, updated_at)
         VALUES ($1, 'published', now(), now())
         ON CONFLICT (edition_date) DO UPDATE SET updated_at = now()
         RETURNING id",
    )
    .bind(fecha)
    .fetch_optional(pool)
    .await
    .unwrap_or(None);

    if let Some(edition_id) = ed_id {
        let s_title = format!("Primeras Planas — {}", fecha);
        let s_summary = if texto.chars().count() > 280 {
            format!("{}...", texto.chars().take(280).collect::<String>())
        } else {
            texto.to_string()
        };

        // La síntesis diaria consolidada se publica EXCLUSIVAMENTE en la página 'primeras_planas'
        // protegiendo y aislando las secciones de Política, Economía y Seguridad.
        let _ = sqlx::query(
            "INSERT INTO news_sections (edition_id, section_key, title, summary, full_content, public_slug, published, created_at, updated_at)
             VALUES ($1, 'primeras_planas', $2, $3, $4, 'primeras_planas', true, now(), now())
             ON CONFLICT (edition_id, section_key) DO UPDATE
             SET title = EXCLUDED.title,
                 summary = EXCLUDED.summary,
                 full_content = EXCLUDED.full_content,
                 published = true,
                 updated_at = now()",
        )
        .bind(edition_id)
        .bind(&s_title)
        .bind(&s_summary)
        .bind(texto)
        .execute(pool)
        .await;
    }
}

pub async fn aprobar_sintesis_diaria(
    State((pool, config)): State<(DbPool, Arc<Config>)>,
    Extension(auth_ctx): Extension<AuthContext>,
    Path(fecha_str): Path<String>,
    Json(payload): Json<AprobarSintesisDiariaRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let fecha = NaiveDate::parse_from_str(&fecha_str, "%Y-%m-%d").map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Formato de fecha inválido. Utilice YYYY-MM-DD"})),
        )
    })?;

    // Determinar extracto final a persistir y despachar por WhatsApp
    let final_extracto: String = if let Some(ref ext) = payload.extracto.as_ref().filter(|e| !e.trim().is_empty()) {
        ext.replace("\r\n", "\n").trim().chars().take(1024).collect()
    } else {
        // Consultar extracto previo guardado
        let prev_ext: Option<String> = sqlx::query_scalar("SELECT texto FROM extractos WHERE fecha = $1")
            .bind(fecha)
            .fetch_optional(&pool)
            .await
            .unwrap_or(None);

        if let Some(pe) = prev_ext.filter(|p| !p.trim().is_empty()) {
            pe
        } else {
            // Corte seguro inicial de la síntesis
            let clean_sint = payload.texto.replace("\r\n", "\n").trim().to_string();
            clean_sint.chars().take(900).collect()
        }
    };

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

    // 2. Guardar extracto definitivo en tabla extractos
    let _ = sqlx::query(
        "INSERT INTO extractos (sintesis_id, fecha, texto, origen, editado_por, actualizado_en)
         VALUES ($1, $2, $3, 'manual', $4, now())
         ON CONFLICT (fecha) DO UPDATE
         SET texto = EXCLUDED.texto,
             editado_por = EXCLUDED.editado_por,
             actualizado_en = now()"
    )
    .bind(sintesis.id)
    .bind(fecha)
    .bind(&final_extracto)
    .bind(auth_ctx.user_id)
    .execute(&mut *tx)
    .await;

    // 3. Marcar todos los boletines de esa fecha que están incluidos como 'aprobado'
    let _ = sqlx::query(
        "UPDATE boletines SET estado = 'aprobado', actualizado_en = now() WHERE fecha_boletin = $1 AND (incluido_en_sintesis = TRUE OR incluido_en_sintesis IS NULL)",
    )
    .bind(fecha)
    .execute(&mut *tx)
    .await;

    // 4. Encolar y despachar mensaje WhatsApp si enviar_whatsapp != Some(false)
    let enviar_ws = payload.enviar_whatsapp.unwrap_or(true);
    let mut target_phones: Vec<String> = Vec::new();

    if enviar_ws {
        use crate::services::kapso::get_whatsapp_config;
        let whatsapp_cfg = get_whatsapp_config(&pool).await;

        #[derive(sqlx::FromRow)]
        struct SubRow {
            phone_number: String,
        }

        // Obtener destinatarios de la lista "Audiencia WhatsApp" del CMS (subscribers activos)
        let subscribers_db = sqlx::query_as::<_, SubRow>(
            "SELECT phone_number FROM subscribers WHERE consent_status = 'subscribed' AND trim(phone_number) != '' AND phone_number NOT LIKE '%5512345678' ORDER BY created_at ASC"
        )
        .fetch_all(&mut *tx)
        .await
        .unwrap_or_default();

        target_phones = if subscribers_db.is_empty() {
            if !whatsapp_cfg.director_phone.trim().is_empty() && whatsapp_cfg.director_phone.trim() != "5215512345678" {
                vec![whatsapp_cfg.director_phone.trim().to_string()]
            } else if !config.director_whatsapp.trim().is_empty() && config.director_whatsapp.trim() != "5215512345678" {
                vec![config.director_whatsapp.trim().to_string()]
            } else {
                vec![]
            }
        } else {
            subscribers_db
                .into_iter()
                .map(|s| s.phone_number)
                .filter(|p| !p.trim().is_empty() && !p.contains("5512345678"))
                .collect()
        };

        for phone in &target_phones {
            let _ = sqlx::query(
                "INSERT INTO mensajes_pendientes (tipo, referencia_id, texto, destinatario, estado, proveedor)
                 VALUES ('brief', $1, $2, $3, 'pendiente', 'kapso')",
            )
            .bind(sintesis.id)
            .bind(&final_extracto)
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

    // Sincronizar síntesis aprobada hacia el CMS en Primeras Planas
    sync_sintesis_to_cms(&pool, fecha, &payload.texto).await;

    // Despacho vía WhatsApp en background exclusivamente por plantilla oficial
    if enviar_ws && !target_phones.is_empty() {
        let pool_clone = pool.clone();
        let s_id = sintesis.id;
        let targets_clone = target_phones.clone();
        let extracto_clone = final_extracto.clone();

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

                let send_res = client.send_brief_template(
                    Some(&pool_clone),
                    phone,
                    fecha,
                    &extracto_clone,
                    "envio_sintesis_diaria_plantilla",
                ).await;

                match send_res {
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
            "extracto": final_extracto,
            "destinatarios_notificados": target_phones.len()
        })),
    ))
}

pub async fn get_extracto_fecha(
    State((pool, _)): State<(DbPool, Arc<Config>)>,
    Path(fecha_str): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let fecha = NaiveDate::parse_from_str(&fecha_str, "%Y-%m-%d").map_err(|_| {
        (StatusCode::BAD_REQUEST, Json(json!({"error": "Formato de fecha inválido"})))
    })?;

    let tope: i32 = sqlx::query_scalar("SELECT tope_extracto FROM whatsapp_plantillas WHERE activa = true LIMIT 1")
        .fetch_optional(&pool)
        .await
        .unwrap_or(None)
        .unwrap_or(900);

    let extracto = sqlx::query_as::<_, Extracto>(
        "SELECT id, sintesis_id, fecha, texto, origen, modelo_usado, tokens_usados, editado_por, creado_en, actualizado_en
         FROM extractos WHERE fecha = $1"
    )
    .bind(fecha)
    .fetch_optional(&pool)
    .await
    .unwrap_or(None);

    let sintesis_texto: Option<String> = sqlx::query_scalar("SELECT texto FROM sintesis_diarias WHERE fecha = $1")
        .bind(fecha)
        .fetch_optional(&pool)
        .await
        .unwrap_or(None);

    let sugerencia = if let Some(ref s_texto) = sintesis_texto {
        let clean = s_texto.replace("\r\n", "\n").trim().to_string();
        if clean.chars().count() <= tope as usize {
            clean
        } else {
            // Intentar cortar en el último salto de línea antes del tope
            let sub: String = clean.chars().take(tope as usize).collect();
            if let Some(idx) = sub.rfind("\n\n") {
                sub[..idx].trim().to_string()
            } else if let Some(idx) = sub.rfind('\n') {
                sub[..idx].trim().to_string()
            } else if let Some(idx) = sub.rfind(". ") {
                sub[..=idx].trim().to_string()
            } else {
                sub
            }
        }
    } else {
        String::new()
    };

    Ok((
        StatusCode::OK,
        Json(json!({
            "fecha": fecha,
            "extracto": extracto,
            "sugerencia": sugerencia,
            "tope": tope
        })),
    ))
}

pub async fn guardar_extracto_fecha(
    State((pool, _)): State<(DbPool, Arc<Config>)>,
    Extension(auth_ctx): Extension<AuthContext>,
    Path(fecha_str): Path<String>,
    Json(payload): Json<GuardarExtractoRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let fecha = NaiveDate::parse_from_str(&fecha_str, "%Y-%m-%d").map_err(|_| {
        (StatusCode::BAD_REQUEST, Json(json!({"error": "Formato de fecha inválido"})))
    })?;

    let clean = payload.texto.replace("\r\n", "\n").trim().to_string();
    if clean.chars().count() > 1024 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": format!("El extracto no puede exceder los 1,024 caracteres permitidos por Meta (actual: {} caracteres)", clean.chars().count())
            })),
        ));
    }

    let sintesis_id: Option<Uuid> = sqlx::query_scalar("SELECT id FROM sintesis_diarias WHERE fecha = $1")
        .bind(fecha)
        .fetch_optional(&pool)
        .await
        .unwrap_or(None);

    let origen = payload.origen.unwrap_or_else(|| "manual".to_string());

    let ext = sqlx::query_as::<_, Extracto>(
        "INSERT INTO extractos (sintesis_id, fecha, texto, origen, editado_por, actualizado_en)
         VALUES ($1, $2, $3, $4, $5, now())
         ON CONFLICT (fecha) DO UPDATE
         SET texto = EXCLUDED.texto,
             origen = EXCLUDED.origen,
             editado_por = EXCLUDED.editado_por,
             actualizado_en = now()
         RETURNING id, sintesis_id, fecha, texto, origen, modelo_usado, tokens_usados, editado_por, creado_en, actualizado_en"
    )
    .bind(sintesis_id)
    .bind(fecha)
    .bind(&clean)
    .bind(&origen)
    .bind(auth_ctx.user_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Error guardando extracto: {}", e)}))))?;

    Ok((
        StatusCode::OK,
        Json(json!({
            "ok": true,
            "mensaje": "Extracto guardado correctamente",
            "extracto": ext
        })),
    ))
}

pub async fn resumir_extracto_fecha(
    State((pool, config)): State<(DbPool, Arc<Config>)>,
    Path(fecha_str): Path<String>,
    Json(payload): Json<ResumirExtractoRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let fecha = NaiveDate::parse_from_str(&fecha_str, "%Y-%m-%d").map_err(|_| {
        (StatusCode::BAD_REQUEST, Json(json!({"error": "Formato de fecha inválido"})))
    })?;

    let text_to_summarize = if let Some(t) = payload.texto.filter(|s| !s.trim().is_empty()) {
        t
    } else {
        let sintesis_opt: Option<String> = sqlx::query_scalar("SELECT texto FROM sintesis_diarias WHERE fecha = $1")
            .bind(fecha)
            .fetch_optional(&pool)
            .await
            .unwrap_or(None);

        match sintesis_opt {
            Some(t) if !t.trim().is_empty() => t,
            _ => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "No hay texto disponible para resumir"})))),
        }
    };

    let tope_cfg: i32 = sqlx::query_scalar("SELECT tope_extracto FROM whatsapp_plantillas WHERE activa = true LIMIT 1")
        .fetch_optional(&pool)
        .await
        .unwrap_or(None)
        .unwrap_or(900);

    let max_chars = payload.max_chars.unwrap_or(tope_cfg as usize);

    // Llamar al worker Python para resumir con Claude
    let worker_url = format!("{}/api/resumir-extracto", config.worker_base_url);
    let client = reqwest::Client::new();

    let resp = client
        .post(&worker_url)
        .json(&json!({
            "texto": text_to_summarize,
            "max_chars": max_chars
        }))
        .send()
        .await;

    match resp {
        Ok(r) if r.status().is_success() => {
            let data: serde_json::Value = r.json().await.unwrap_or_default();
            let extracto = data.get("extracto").and_then(|e| e.as_str()).unwrap_or("").to_string();
            let tokens = data.get("tokens_usados").and_then(|t| t.as_i64()).map(|t| t as i32);
            let modelo = data.get("modelo_usado").and_then(|m| m.as_str()).map(|s| s.to_string());

            Ok((
                StatusCode::OK,
                Json(json!({
                    "extracto": extracto,
                    "origen": "claude",
                    "modelo_usado": modelo,
                    "tokens_usados": tokens
                })),
            ))
        }
        _ => {
            // Fallback inteligente en local si el worker no responde
            let clean = text_to_summarize.replace("\r\n", "\n").trim().to_string();
            let cut: String = clean.chars().take(max_chars).collect();
            let fallback_extract = if let Some(idx) = cut.rfind("\n\n") {
                cut[..idx].trim().to_string()
            } else if let Some(idx) = cut.rfind('\n') {
                cut[..idx].trim().to_string()
            } else {
                cut
            };

            Ok((
                StatusCode::OK,
                Json(json!({
                    "extracto": fallback_extract,
                    "origen": "corte",
                    "modelo_usado": "fallback_local",
                    "tokens_usados": 0
                })),
            ))
        }
    }
}

pub async fn eliminar_boletin(
    State((pool, _)): State<(DbPool, Arc<Config>)>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
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


