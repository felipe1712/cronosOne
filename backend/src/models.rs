use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// ============================================================================
// 1. Usuarios y Roles
// ============================================================================

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "varchar", rename_all = "snake_case")]
pub enum RolUsuario {
    #[serde(rename = "admin")]
    Admin,
    #[serde(rename = "director")]
    Director,
    #[serde(rename = "analista")]
    Analista,
}

#[allow(dead_code)]
impl RolUsuario {
    pub fn as_str(&self) -> &'static str {
        match self {
            RolUsuario::Admin => "admin",
            RolUsuario::Director => "director",
            RolUsuario::Analista => "analista",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "admin" => Some(RolUsuario::Admin),
            "director" => Some(RolUsuario::Director),
            "analista" => Some(RolUsuario::Analista),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Usuario {
    pub id: Uuid,
    pub nombre: String,
    pub correo: String,
    pub rol: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub activo: bool,
    pub creado_en: DateTime<Utc>,
    pub actualizado_en: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub correo: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub usuario: UsuarioSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsuarioSummary {
    pub id: Uuid,
    pub nombre: String,
    pub correo: String,
    pub rol: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub correo: String,
    pub rol: String,
    pub exp: usize,
}

// ============================================================================
// 2. Boletines Coparmex
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Boletin {
    pub id: Uuid,
    pub fecha_boletin: NaiveDate,
    pub nombre_archivo: String,
    pub ruta_archivo: String,
    pub subido_por: Option<Uuid>,
    pub estado: String,
    pub total_paginas: Option<i32>,
    pub error_mensaje: Option<String>,
    pub creado_en: DateTime<Utc>,
    pub actualizado_en: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Seccion {
    pub id: Uuid,
    pub boletin_id: Uuid,
    pub orden: i32,
    pub tema: Option<String>,
    pub pagina_inicio: Option<i32>,
    pub pagina_fin: Option<i32>,
    pub contenido: serde_json::Value,
    pub creado_en: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SintesisGenerada {
    pub id: Uuid,
    pub boletin_id: Uuid,
    pub texto: String,
    pub temas: Option<Vec<String>>,
    pub modelo_usado: Option<String>,
    pub tokens_usados: Option<i32>,
    pub creado_en: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct BoletinDetailResponse {
    pub boletin: Boletin,
    pub sintesis: Option<SintesisGenerada>,
    pub secciones: Vec<Seccion>,
}

// ============================================================================
// 3. Cola de Mensajes Pendientes (Polling n8n / WAHA)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MensajePendiente {
    pub id: Uuid,
    pub tipo: String, // 'brief' | 'alerta'
    pub referencia_id: Option<Uuid>,
    pub texto: String,
    pub destinatario: String,
    pub estado: String, // 'pendiente' | 'entregado_a_n8n' | 'confirmado' | 'error'
    pub intento_conteo: i32,
    pub error_mensaje: Option<String>,
    pub creado_en: DateTime<Utc>,
    pub entregado_en: Option<DateTime<Utc>>,
    pub confirmado_en: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct ConfirmarMensajeRequest {
    pub estado: String, // 'confirmado' o 'error'
    pub error_mensaje: Option<String>,
}

// ============================================================================
// 4. Entidades Vigiladas y Alertas OSINT (world-intel-mcp)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EntidadVigilada {
    pub id: Uuid,
    pub tipo: String, // 'marca', 'dominio', 'ejecutivo', 'patron_poliza'
    pub valor: String,
    pub descripcion: Option<String>,
    pub activo: bool,
    pub creado_en: DateTime<Utc>,
    pub actualizado_en: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateEntidadRequest {
    pub tipo: String,
    pub valor: String,
    pub descripcion: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AlertaOsint {
    pub id: Uuid,
    pub entidad_id: Option<Uuid>,
    pub fuente: String,
    pub titulo: String,
    pub descripcion: String,
    pub severidad: String, // 'baja', 'media', 'alta', 'critica'
    pub evidencia: Option<serde_json::Value>,
    pub estado: String,    // 'por_validar', 'validada', 'descartada'
    pub validada_por: Option<Uuid>,
    pub notas_analista: Option<String>,
    pub creado_en: DateTime<Utc>,
    pub validada_en: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct ValidarAlertaRequest {
    pub accion: String, // 'validar' | 'descartar'
    pub notas: Option<String>,
}

// ============================================================================
// 5. Estado de Sesión WhatsApp (Alimentado por n8n / WAHA)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WhatsappSesion {
    pub id: String,
    pub estado: String,
    pub qr_code: Option<String>,
    pub detalles: Option<serde_json::Value>,
    pub actualizado_en: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct WebhookWhatsappSessionPayload {
    pub session: Option<String>,
    pub status: String,
    pub qr: Option<String>,
    pub detalles: Option<serde_json::Value>,
}
