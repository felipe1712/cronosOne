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
        phone.chars().filter(|c| c.is_ascii_digit()).collect()
    }

    pub async fn send_text_with_log(
        &self,
        pool: Option<&PgPool>,
        to: &str,
        text: &str,
        accion: &str,
    ) -> Result<String, String> {
        let start = std::time::Instant::now();
        let clean_phone = Self::sanitize_phone(to);
        let payload = json!({
            "messaging_product": "whatsapp",
            "to": clean_phone,
            "type": "text",
            "text": {
                "body": text
            }
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

        let url = format!(
            "https://api.kapso.ai/meta/whatsapp/v24.0/{}/messages",
            self.phone_number_id.trim()
        );

        info!("Enviando mensaje WhatsApp oficial vía Kapso a: {}", clean_phone);

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
            error!("Error en Kapso API (HTTP {}): {:?}", status, body);
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
            .unwrap_or("kapso_msg_ok")
            .to_string();

        info!("✅ Mensaje entregado a Kapso con éxito. Message ID: {}", msg_id);
        if let Some(p) = pool {
            log_whatsapp_api(p, accion, to, "ok", Some(status.as_u16() as i32), Some(latency), &payload_str, &body_str, None).await;
        }
        Ok(msg_id)
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
        let t_name = template_name.trim();
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

        if let Some(vars) = variables {
            if !vars.is_empty() {
                let params: Vec<serde_json::Value> = vars
                    .iter()
                    .map(|v| {
                        json!({
                            "type": "text",
                            "text": v
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
    let mut template_name = "hello_world".to_string();
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
                if !r.valor.trim().is_empty() {
                    template_name = r.valor.trim().to_string();
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
        if !env_tpl.trim().is_empty() {
            template_name = env_tpl.trim().to_string();
        }
    }
    if let Ok(env_lang) = std::env::var("WHATSAPP_TEMPLATE_LANGUAGE") {
        if !env_lang.trim().is_empty() {
            template_language = env_lang.trim().to_string();
        }
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
