use reqwest::Client;
use serde_json::json;
use sqlx::PgPool;
use tracing::{error, info};

#[derive(Debug, Clone)]
pub struct KapsoConfig {
    pub provider: String,
    pub api_key: String,
    pub phone_number_id: String,
    pub director_phone: String,
    pub template_name: String,
    pub template_language: String,
}

pub struct KapsoClient {
    client: Client,
    pub api_key: String,
    pub phone_number_id: String,
}

fn extract_error_detail(status: reqwest::StatusCode, body: &serde_json::Value) -> String {
    if let Some(err_obj) = body.get("error") {
        if let Some(msg) = err_obj.get("message").and_then(|m| m.as_str()) {
            let code = err_obj
                .get("code")
                .and_then(|c| c.as_i64())
                .map(|c| format!(" (código Meta {})", c))
                .unwrap_or_default();
            format!("Error {}: {}{}", status, msg, code)
        } else if let Some(err_str) = err_obj.as_str() {
            format!("Error {}: {}", status, err_str)
        } else {
            format!("Error {}: {}", status, err_obj)
        }
    } else if let Some(msg) = body.get("message").and_then(|m| m.as_str()) {
        format!("Error {}: {}", status, msg)
    } else {
        format!("Error {}: {}", status, body)
    }
}

impl KapsoClient {
    pub fn new(api_key: String, phone_number_id: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            phone_number_id,
        }
    }

    pub fn sanitize_phone(phone: &str) -> String {
        let digits: String = phone.chars().filter(|c| c.is_ascii_digit()).collect();
        // Si el usuario ingresa un número de 10 dígitos (estándar nacional en México),
        // anteponer el código de país 52 requerido por Meta Cloud API
        if digits.len() == 10 {
            format!("52{}", digits)
        } else {
            digits
        }
    }

    pub fn split_text_for_whatsapp(text: &str, max_chars: usize) -> Vec<String> {
        let trimmed = text.trim();
        if trimmed.chars().count() <= max_chars {
            return vec![trimmed.to_string()];
        }

        let mut chunks = Vec::new();
        let mut remaining = trimmed;

        while !remaining.is_empty() {
            let total_chars = remaining.chars().count();
            if total_chars <= max_chars {
                chunks.push(remaining.trim().to_string());
                break;
            }

            // Mapear hasta max_chars caracteres sin romper caracteres UTF-8 multi-byte
            let char_indices: Vec<(usize, char)> = remaining.char_indices().collect();
            let boundary_byte_idx = if char_indices.len() > max_chars {
                char_indices[max_chars].0
            } else {
                remaining.len()
            };

            let window = &remaining[..boundary_byte_idx];

            // Intentar cortar limpiamente en salto de párrafo "\n\n", o salto de línea "\n", o punto y espacio ". ", o espacio " "
            let split_pos = if let Some(pos) = window.rfind("\n\n") {
                pos + 2
            } else if let Some(pos) = window.rfind('\n') {
                pos + 1
            } else if let Some(pos) = window.rfind(". ") {
                pos + 2
            } else if let Some(pos) = window.rfind(' ') {
                pos + 1
            } else {
                boundary_byte_idx
            };

            let split_pos = if split_pos == 0 { boundary_byte_idx } else { split_pos };

            let chunk = remaining[..split_pos].trim();
            if !chunk.is_empty() {
                chunks.push(chunk.to_string());
            }

            remaining = remaining[split_pos..].trim_start();
        }

        if chunks.is_empty() {
            vec![trimmed.to_string()]
        } else {
            chunks
        }
    }

    async fn send_single_text_chunk(
        &self,
        pool: Option<&PgPool>,
        clean_phone: &str,
        chunk: &str,
        accion: &str,
    ) -> Result<String, String> {
        let start = std::time::Instant::now();
        // Garantizar de forma estricta que jamás exceda el límite de 4096 caracteres de Meta
        let safe_body: String = chunk.chars().take(4096).collect();
        let payload = json!({
            "messaging_product": "whatsapp",
            "to": clean_phone,
            "type": "text",
            "text": {
                "body": safe_body
            }
        });
        let payload_str = payload.to_string();

        let url = format!(
            "https://api.kapso.ai/meta/whatsapp/v24.0/{}/messages",
            self.phone_number_id.trim()
        );

        info!("Enviando mensaje WhatsApp oficial vía Kapso a: {} (longitud: {} chars)", clean_phone, safe_body.chars().count());

        let response = match self
            .client
            .post(&url)
            .header("X-API-Key", self.api_key.trim())
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
        {
            Ok(resp) => resp,
            Err(e) => {
                let latency = start.elapsed().as_millis() as i32;
                let err_str = format!("Error de conexión HTTP con Kapso: {}", e);
                if let Some(p) = pool {
                    log_whatsapp_api(p, accion, clean_phone, "error", Some(502), Some(latency), &payload_str, "", Some(&err_str)).await;
                }
                return Err(err_str);
            }
        };

        let status = response.status();
        let latency = start.elapsed().as_millis() as i32;
        let body: serde_json::Value = match response.json().await {
            Ok(b) => b,
            Err(e) => {
                let err_str = format!("Error decodificando respuesta de Kapso: {}", e);
                if let Some(p) = pool {
                    log_whatsapp_api(p, accion, clean_phone, "error", Some(status.as_u16() as i32), Some(latency), &payload_str, "", Some(&err_str)).await;
                }
                return Err(err_str);
            }
        };
        let body_str = body.to_string();

        if !status.is_success() {
            let err_detail = extract_error_detail(status, &body);
            error!("Error en Kapso API (HTTP {}): {:?}", status, body);
            if let Some(p) = pool {
                log_whatsapp_api(p, accion, clean_phone, "error", Some(status.as_u16() as i32), Some(latency), &payload_str, &body_str, Some(&err_detail)).await;
            }
            return Err(err_detail);
        }

        let msg_id = body
            .get("messages")
            .and_then(|m| m.as_array())
            .and_then(|arr| arr.first())
            .and_then(|first| first.get("id"))
            .and_then(|id| id.as_str())
            .unwrap_or("kapso_msg_ok")
            .to_string();

        info!("✅ Mensaje entregado a Kapso con éxito. Message ID: {}", msg_id);
        if let Some(p) = pool {
            log_whatsapp_api(p, accion, clean_phone, "ok", Some(status.as_u16() as i32), Some(latency), &payload_str, &body_str, None).await;
        }
        Ok(msg_id)
    }

    pub async fn send_text_with_log(
        &self,
        pool: Option<&PgPool>,
        to: &str,
        text: &str,
        accion: &str,
    ) -> Result<String, String> {
        let clean_phone = Self::sanitize_phone(to);
        if clean_phone.is_empty() {
            let err = "Número de teléfono de destino vacío o inválido".to_string();
            if let Some(p) = pool {
                log_whatsapp_api(p, accion, to, "error", Some(400), Some(0), "", "", Some(&err)).await;
            }
            return Err(err);
        }
        if self.api_key.trim().is_empty() {
            let err = "KAPSO_API_KEY no configurada en el sistema".to_string();
            if let Some(p) = pool {
                log_whatsapp_api(p, accion, to, "error", Some(500), Some(0), "", "", Some(&err)).await;
            }
            return Err(err);
        }
        if self.phone_number_id.trim().is_empty() {
            let err = "KAPSO_PHONE_NUMBER_ID no configurado en el sistema".to_string();
            if let Some(p) = pool {
                log_whatsapp_api(p, accion, to, "error", Some(500), Some(0), "", "", Some(&err)).await;
            }
            return Err(err);
        }

        // Límite oficial de Meta Cloud API: text.body <= 4096 caracteres
        // Si el texto supera 4096 caracteres, se divide inteligentemente en partes consecutivas
        let chunks = Self::split_text_for_whatsapp(text, 4096);
        let mut last_msg_id = String::new();

        for (idx, chunk) in chunks.iter().enumerate() {
            let part_accion = if chunks.len() > 1 {
                format!("{}_parte_{}_de_{}", accion, idx + 1, chunks.len())
            } else {
                accion.to_string()
            };

            let msg_id = self.send_single_text_chunk(pool, &clean_phone, chunk, &part_accion).await?;
            last_msg_id = msg_id;

            if chunks.len() > 1 && idx + 1 < chunks.len() {
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            }
        }

        Ok(last_msg_id)
    }

    pub async fn send_text(&self, to: &str, text: &str) -> Result<String, String> {
        self.send_text_with_log(None, to, text, "mensaje_texto").await
    }

    /// Envía una plantilla oficial aprobada en Meta Cloud API / Kapso registrando log opcional en BD
    pub async fn send_template_with_log(
        &self,
        pool: Option<&PgPool>,
        to: &str,
        template_name: &str,
        language_code: &str,
        variables: Option<&[String]>,
        accion: &str,
    ) -> Result<String, String> {
        let start = std::time::Instant::now();
        let clean_phone = Self::sanitize_phone(to);
        let raw_name = template_name.trim();
        let t_name = if raw_name.is_empty() || raw_name == "hello_world" {
            "servicio_monitoreo"
        } else {
            raw_name
        };
        let lang = if language_code.trim().is_empty() {
            "es_MX"
        } else {
            language_code.trim()
        };

        let mut template_obj = json!({
            "name": t_name,
            "language": {
                "code": lang
            }
        });

        // La plantilla aprobada 'servicio_monitoreo' fue aprobada sin variables en Meta.
        // Bajo ninguna circunstancia debe adjuntarse el bloque 'components'.
        if t_name.to_lowercase() != "servicio_monitoreo" {
            if let Some(vars) = variables {
                let non_empty_vars: Vec<&String> = vars.iter().filter(|v| !v.trim().is_empty()).collect();
                if !non_empty_vars.is_empty() {
                    let params: Vec<serde_json::Value> = non_empty_vars
                        .iter()
                        .map(|v| {
                            json!({
                                "type": "text",
                                "text": v.trim()
                            })
                        })
                        .collect();

                    template_obj["components"] = json!([
                        {
                            "type": "body",
                            "parameters": params
                        }
                    ]);
                }
            }
        }

        let payload = json!({
            "messaging_product": "whatsapp",
            "to": clean_phone,
            "type": "template",
            "template": template_obj
        });
        let payload_str = payload.to_string();

        if clean_phone.is_empty() {
            let err = "Número de teléfono de destino vacío o inválido".to_string();
            if let Some(p) = pool {
                log_whatsapp_api(p, accion, to, "error", Some(400), Some(0), &payload_str, "", Some(&err)).await;
            }
            return Err(err);
        }
        if self.api_key.trim().is_empty() {
            let err = "KAPSO_API_KEY no configurada en el sistema".to_string();
            if let Some(p) = pool {
                log_whatsapp_api(p, accion, to, "error", Some(500), Some(0), &payload_str, "", Some(&err)).await;
            }
            return Err(err);
        }
        if self.phone_number_id.trim().is_empty() {
            let err = "KAPSO_PHONE_NUMBER_ID no configurado en el sistema".to_string();
            if let Some(p) = pool {
                log_whatsapp_api(p, accion, to, "error", Some(500), Some(0), &payload_str, "", Some(&err)).await;
            }
            return Err(err);
        }
        if t_name.is_empty() {
            let err = "El nombre de la plantilla de WhatsApp es obligatorio".to_string();
            if let Some(p) = pool {
                log_whatsapp_api(p, accion, to, "error", Some(400), Some(0), &payload_str, "", Some(&err)).await;
            }
            return Err(err);
        }

        let url = format!(
            "https://api.kapso.ai/meta/whatsapp/v24.0/{}/messages",
            self.phone_number_id.trim()
        );

        info!(
            "Enviando plantilla WhatsApp '{}' ({}) vía Kapso a: {}",
            t_name, lang, clean_phone
        );

        let response = match self
            .client
            .post(&url)
            .header("X-API-Key", self.api_key.trim())
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
        {
            Ok(resp) => resp,
            Err(e) => {
                let latency = start.elapsed().as_millis() as i32;
                let err_str = format!("Error de conexión HTTP con Kapso: {}", e);
                if let Some(p) = pool {
                    log_whatsapp_api(p, accion, to, "error", Some(502), Some(latency), &payload_str, "", Some(&err_str)).await;
                }
                return Err(err_str);
            }
        };

        let status = response.status();
        let latency = start.elapsed().as_millis() as i32;
        let body: serde_json::Value = match response.json().await {
            Ok(b) => b,
            Err(e) => {
                let err_str = format!("Error decodificando respuesta de Kapso: {}", e);
                if let Some(p) = pool {
                    log_whatsapp_api(p, accion, to, "error", Some(status.as_u16() as i32), Some(latency), &payload_str, "", Some(&err_str)).await;
                }
                return Err(err_str);
            }
        };
        let body_str = body.to_string();

        if !status.is_success() {
            let err_detail = extract_error_detail(status, &body);
            error!("Error en Kapso Template API (HTTP {}): {:?}", status, body);
            if let Some(p) = pool {
                log_whatsapp_api(p, accion, to, "error", Some(status.as_u16() as i32), Some(latency), &payload_str, &body_str, Some(&err_detail)).await;
            }
            return Err(err_detail);
        }

        let msg_id = body
            .get("messages")
            .and_then(|m| m.as_array())
            .and_then(|arr| arr.first())
            .and_then(|first| first.get("id"))
            .and_then(|id| id.as_str())
            .unwrap_or("kapso_template_ok")
            .to_string();

        info!("✅ Plantilla WhatsApp entregada a Kapso con éxito. Message ID: {}", msg_id);
        if let Some(p) = pool {
            log_whatsapp_api(p, accion, to, "ok", Some(status.as_u16() as i32), Some(latency), &payload_str, &body_str, None).await;
        }
        Ok(msg_id)
    }

    pub async fn send_template(
        &self,
        to: &str,
        template_name: &str,
        language_code: &str,
        variables: Option<&[String]>,
    ) -> Result<String, String> {
        self.send_template_with_log(None, to, template_name, language_code, variables, "plantilla_whatsapp").await
    }
}

pub async fn log_whatsapp_api(
    pool: &PgPool,
    accion: &str,
    destinatario: &str,
    estado: &str,
    codigo_http: Option<i32>,
    latencia_ms: Option<i32>,
    peticion_payload: &str,
    respuesta_payload: &str,
    error_mensaje: Option<&str>,
) {
    crate::handlers::configuracion::ensure_api_logs_schema(pool).await;

    let p_safe = if peticion_payload.len() > 100000 { &peticion_payload[..100000] } else { peticion_payload };
    let r_safe = if respuesta_payload.len() > 100000 { &respuesta_payload[..100000] } else { respuesta_payload };
    let norm_estado = if estado == "exitoso" { "ok" } else { estado };

    let res = sqlx::query(
        "INSERT INTO api_logs (
            servicio, accion, modelo_o_proveedor, estado, codigo_http,
            latencia_ms, tokens_input, tokens_output, tokens_total,
            destinatario, peticion_payload, respuesta_payload, error_mensaje, creado_en
        )
        VALUES ('whatsapp', $1, 'kapso', $2, $3, $4, 0, 0, 0, $5, $6, $7, $8, now())"
    )
    .bind(accion)
    .bind(norm_estado)
    .bind(codigo_http)
    .bind(latencia_ms)
    .bind(destinatario)
    .bind(p_safe)
    .bind(r_safe)
    .bind(error_mensaje)
    .execute(pool)
    .await;

    if let Err(e) = res {
        tracing::error!("❌ [api_logs] Error al registrar log de WhatsApp en BD: {}", e);
    } else {
        tracing::info!("✅ [api_logs] Log de WhatsApp registrado exitosamente (accion={}, dest={})", accion, destinatario);
    }
}

pub async fn get_whatsapp_config(pool: &PgPool) -> KapsoConfig {
    #[derive(sqlx::FromRow)]
    struct Row {
        clave: String,
        valor: String,
    }

    let rows = sqlx::query_as::<_, Row>(
        "SELECT clave, valor FROM configuraciones_sistema",
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let mut provider = "kapso".to_string();
    let mut api_key = String::new();
    let mut phone_number_id = String::new();
    let mut director_phone = String::new();
    let mut template_name = "servicio_monitoreo".to_string();
    let mut template_language = "es_MX".to_string();

    for r in rows {
        match r.clave.as_str() {
            "WHATSAPP_PROVIDER" => provider = r.valor,
            "KAPSO_API_KEY" => api_key = r.valor,
            "KAPSO_PHONE_NUMBER_ID" => phone_number_id = r.valor,
            "DIRECTOR_WHATSAPP_PHONE" => {
                if r.valor.trim() != "5215512345678" {
                    director_phone = r.valor;
                }
            }
            "WHATSAPP_TEMPLATE_NAME" => {
                let v = r.valor.trim();
                if !v.is_empty() && v != "hello_world" {
                    template_name = v.to_string();
                }
            }
            "WHATSAPP_TEMPLATE_LANGUAGE" => {
                if !r.valor.trim().is_empty() {
                    template_language = r.valor.trim().to_string();
                }
            }
            _ => {}
        }
    }

    if api_key.trim().is_empty() {
        if let Ok(env_key) = std::env::var("KAPSO_API_KEY") {
            api_key = env_key;
        }
    }
    if phone_number_id.trim().is_empty() {
        if let Ok(env_id) = std::env::var("KAPSO_PHONE_NUMBER_ID") {
            phone_number_id = env_id;
        }
    }
    if director_phone.trim().is_empty() {
        if let Ok(env_phone) = std::env::var("DIRECTOR_WHATSAPP_PHONE") {
            if env_phone.trim() != "5215512345678" {
                director_phone = env_phone;
            }
        }
    }
    if director_phone.trim().is_empty() {
        if let Ok(Some(row)) = sqlx::query_as::<_, (String,)>(
            "SELECT telefono FROM lista_distribucion WHERE activo = true AND telefono != '5215512345678' ORDER BY creado_en ASC LIMIT 1"
        )
        .fetch_optional(pool)
        .await {
            director_phone = row.0;
        }
    }

    if let Ok(env_tpl) = std::env::var("WHATSAPP_TEMPLATE_NAME") {
        let v = env_tpl.trim();
        if !v.is_empty() && v != "hello_world" {
            template_name = v.to_string();
        }
    }
    if let Ok(env_lang) = std::env::var("WHATSAPP_TEMPLATE_LANGUAGE") {
        if !env_lang.trim().is_empty() {
            template_language = env_lang.trim().to_string();
        }
    }

    if template_name.is_empty() || template_name == "hello_world" {
        template_name = "servicio_monitoreo".to_string();
    }
    if template_language.is_empty() {
        template_language = "es_MX".to_string();
    }

    KapsoConfig {
        provider,
        api_key,
        phone_number_id,
        director_phone,
        template_name,
        template_language,
    }
}
