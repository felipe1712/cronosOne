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

#[derive(Debug, Deserialize)]
pub struct AprobarBoletinRequest {
    pub texto: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSintesisRequest {
    pub texto: String,
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

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FuenteOsint {
    pub id: Uuid,
    pub clave: String,
    pub nombre: String,
    pub descripcion: String,
    pub dominio: String,
    pub tipo: String,
    pub activo: bool,
    pub ultimo_escaneo: Option<DateTime<Utc>>,
    pub total_hallazgos: i32,
    pub creado_en: DateTime<Utc>,
    pub actualizado_en: DateTime<Utc>,
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

// ============================================================================
// 6. Lista de Distribución y Grupos Temáticos WhatsApp
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct GrupoDistribucion {
    pub id: Uuid,
    pub nombre: String,
    pub descripcion: Option<String>,
    pub color: String,
    pub activo: bool,
    pub creado_en: DateTime<Utc>,
    #[sqlx(default)]
    pub total_miembros: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct CreateGrupoRequest {
    pub nombre: String,
    pub descripcion: Option<String>,
    pub color: Option<String>,
    pub activo: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateGrupoRequest {
    pub nombre: String,
    pub descripcion: Option<String>,
    pub color: Option<String>,
    pub activo: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct GrupoResumen {
    pub id: Uuid,
    pub nombre: String,
    pub color: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct DestinatarioRow {
    pub id: Uuid,
    pub nombre: String,
    pub telefono: String,
    pub cargo: Option<String>,
    pub activo: bool,
    pub notas: Option<String>,
    pub creado_en: DateTime<Utc>,
    pub actualizado_en: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Destinatario {
    pub id: Uuid,
    pub nombre: String,
    pub telefono: String,
    pub cargo: Option<String>,
    pub activo: bool,
    pub notas: Option<String>,
    pub creado_en: DateTime<Utc>,
    pub actualizado_en: DateTime<Utc>,
    pub grupos: Option<Vec<GrupoResumen>>,
}

impl From<DestinatarioRow> for Destinatario {
    fn from(r: DestinatarioRow) -> Self {
        Self {
            id: r.id,
            nombre: r.nombre,
            telefono: r.telefono,
            cargo: r.cargo,
            activo: r.activo,
            notas: r.notas,
            creado_en: r.creado_en,
            actualizado_en: r.actualizado_en,
            grupos: None,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateDestinatarioRequest {
    pub nombre: String,
    pub telefono: String,
    pub cargo: Option<String>,
    pub notas: Option<String>,
    pub activo: Option<bool>,
    pub grupo_ids: Option<Vec<Uuid>>,
    pub enviar_plantilla: Option<bool>,
    pub template_name: Option<String>,
    pub template_language: Option<String>,
    pub template_variables: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateDestinatarioRequest {
    pub nombre: String,
    pub telefono: String,
    pub cargo: Option<String>,
    pub notas: Option<String>,
    pub activo: bool,
    pub grupo_ids: Option<Vec<Uuid>>,
}

#[derive(Debug, Deserialize, Default)]
pub struct EnviarPlantillaRequest {
    pub telefono: Option<String>,
    pub template_name: Option<String>,
    pub template_language: Option<String>,
    pub variables: Option<Vec<String>>,
    pub api_key: Option<String>,
    pub phone_number_id: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct TestTemplatePayload {
    pub phone: Option<String>,
    pub template_name: Option<String>,
    pub template_language: Option<String>,
    pub variables: Option<Vec<String>>,
    pub api_key: Option<String>,
    pub phone_number_id: Option<String>,
}

// ============================================================================
// 6. Síntesis Diarias Consolidadas (Monitoreo Ejecutivo por Fecha)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SintesisDiaria {
    pub id: Uuid,
    pub fecha: NaiveDate,
    pub texto: String,
    pub temas: Option<Vec<String>>,
    pub documentos_ids: Option<Vec<Uuid>>,
    pub estado: String,
    pub modelo_usado: Option<String>,
    pub tokens_usados: Option<i32>,
    pub aprobado_por: Option<Uuid>,
    pub creado_en: DateTime<Utc>,
    pub actualizado_en: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoletinEnWorkspace {
    pub id: Uuid,
    pub fecha_boletin: NaiveDate,
    pub nombre_archivo: String,
    pub ruta_archivo: String,
    pub estado: String,
    pub origen: String,
    pub incluido_en_sintesis: bool,
    pub total_paginas: Option<i32>,
    pub error_mensaje: Option<String>,
    pub creado_en: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SintesisDiariaResumen {
    pub fecha: NaiveDate,
    pub total_documentos: i64,
    pub total_senado: i64,
    pub total_manual: i64,
    pub total_paginas: i64,
    pub sintesis_id: Option<Uuid>,
    pub estado_sintesis: Option<String>,
    pub texto_preview: Option<String>,
    pub modelo_usado: Option<String>,
    pub actualizado_en: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct WorkspaceFechaResponse {
    pub fecha: NaiveDate,
    pub documentos: Vec<BoletinEnWorkspace>,
    pub sintesis: Option<SintesisDiaria>,
    pub extracto: Option<Extracto>,
    pub total_documentos: usize,
    pub documentos_incluidos: usize,
}

#[derive(Debug, Deserialize)]
pub struct ToggleDocumentoSeleccionRequest {
    pub incluido: bool,
}

#[derive(Debug, Deserialize, Default)]
pub struct ConsolidarSintesisFechaRequest {
    pub documentos_ids: Option<Vec<Uuid>>,
}

#[derive(Debug, Deserialize)]
pub struct ActualizarSintesisDiariaRequest {
    pub texto: String,
    pub extracto: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AprobarSintesisDiariaRequest {
    pub texto: String,
    pub extracto: Option<String>,
    pub enviar_whatsapp: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Extracto {
    pub id: Uuid,
    pub sintesis_id: Option<Uuid>,
    pub fecha: NaiveDate,
    pub texto: String,
    pub origen: String,
    pub modelo_usado: Option<String>,
    pub tokens_usados: Option<i32>,
    pub editado_por: Option<Uuid>,
    pub creado_en: DateTime<Utc>,
    pub actualizado_en: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct GuardarExtractoRequest {
    pub texto: String,
    pub origen: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ResumirExtractoRequest {
    pub texto: Option<String>,
    pub max_chars: Option<usize>,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct ResumirExtractoResponse {
    pub extracto: String,
    pub origen: String,
    pub modelo_usado: Option<String>,
    pub tokens_usados: Option<i32>,
}

// ============================================================================
// 7. Logs de APIs (Claude y WhatsApp)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ApiLogRow {
    pub id: Uuid,
    pub servicio: String,
    pub accion: String,
    pub modelo_o_proveedor: Option<String>,
    pub estado: String,
    pub codigo_http: Option<i32>,
    pub latencia_ms: Option<i32>,
    pub tokens_input: Option<i32>,
    pub tokens_output: Option<i32>,
    pub tokens_total: Option<i32>,
    pub max_tokens_configurado: Option<i32>,
    pub destinatario: Option<String>,
    pub peticion_payload: Option<String>,
    pub respuesta_payload: Option<String>,
    pub error_mensaje: Option<String>,
    pub creado_en: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiLogItem {
    pub id: String,
    pub servicio: String,
    pub accion: String,
    pub modelo_o_proveedor: String,
    pub estado: String,
    pub codigo_http: Option<i32>,
    pub latencia_ms: Option<i32>,
    pub tokens_input: i32,
    pub tokens_output: i32,
    pub tokens_total: i32,
    pub max_tokens_configurado: Option<i32>,
    pub destinatario: Option<String>,
    pub peticion_payload: String,
    pub respuesta_payload: String,
    pub error_mensaje: Option<String>,
    pub creado_en: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApiLogStats {
    pub total_claude: i64,
    pub tokens_claude: i64,
    pub exitosos_claude: i64,
    pub errores_claude: i64,
    pub total_whatsapp: i64,
    pub exitosos_whatsapp: i64,
    pub errores_whatsapp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct ApiLogsResponse {
    pub logs: Vec<ApiLogItem>,
    pub stats: ApiLogStats,
    pub total: i64,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct GetApiLogsQuery {
    pub servicio: Option<String>,
    pub estado: Option<String>,
    pub limite: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ClearApiLogsQuery {
    pub servicio: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct CreateApiLogPayload {
    pub servicio: String,
    pub accion: String,
    pub modelo_o_proveedor: Option<String>,
    pub estado: String,
    pub codigo_http: Option<i32>,
    pub latencia_ms: Option<i32>,
    pub tokens_input: Option<i32>,
    pub tokens_output: Option<i32>,
    pub tokens_total: Option<i32>,
    pub max_tokens_configurado: Option<i32>,
    pub destinatario: Option<String>,
    pub peticion_payload: Option<String>,
    pub respuesta_payload: Option<String>,
    pub error_mensaje: Option<String>,
}



// ============================================================================
// 8. Modelos CMS de Páginas, Ediciones Diarias y Broadcasts (Kapso Platform)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct NewsPage {
    pub id: Uuid,
    pub section_key: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub color: String,
    pub icon: String,
    pub orden: i32,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateNewsPageRequest {
    pub section_key: String,
    pub title: String,
    pub subtitle: Option<String>,
    #[serde(alias = "color_hex")]
    pub color: Option<String>,
    #[serde(alias = "icon_name")]
    pub icon: Option<String>,
    pub orden: Option<i32>,
    #[serde(alias = "is_active")]
    pub active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateNewsPageRequest {
    pub title: String,
    pub subtitle: Option<String>,
    #[serde(alias = "color_hex")]
    pub color: Option<String>,
    #[serde(alias = "icon_name")]
    pub icon: Option<String>,
    pub orden: Option<i32>,
    #[serde(alias = "is_active")]
    pub active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct NewsEdition {
    pub id: Uuid,
    pub edition_date: NaiveDate,
    pub status: String,
    pub generated_at: Option<DateTime<Utc>>,
    pub published_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct NewsSection {
    pub id: Uuid,
    pub edition_id: Uuid,
    pub section_key: String,
    pub title: String,
    pub summary: String,
    pub full_content: String,
    pub public_slug: String,
    pub published: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct NewsEditionWithSections {
    pub edition: NewsEdition,
    pub sections: Vec<NewsSection>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateNewsSectionRequest {
    pub title: String,
    pub summary: String,
    pub full_content: String,
    pub published: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct PublishEditionRequest {
    pub send_broadcast: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicSectionInfo {
    pub section_key: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub color: String,
    pub icon: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicBriefItem {
    pub id: Uuid,
    pub edition_date: NaiveDate,
    pub section_key: String,
    pub section_title: String,
    pub section_color: String,
    pub section_icon: String,
    pub title: String,
    pub summary: String,
    pub full_content: String,
    pub public_slug: String,
    pub published_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct PublicWeekSummary {
    pub week_number: u32,
    pub year: i32,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub label: String,
    pub is_current_week: bool,
    pub items: Vec<PublicBriefItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Subscriber {
    pub id: Uuid,
    pub phone_number: String,
    pub display_name: Option<String>,
    pub consent_status: String,
    pub consent_source: String,
    pub consent_text_version: String,
    pub consented_at: DateTime<Utc>,
    pub unsubscribed_at: Option<DateTime<Utc>>,
    pub last_marketing_preference: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSubscriberRequest {
    pub phone_number: String,
    pub display_name: Option<String>,
    pub consent_status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSubscriberRequest {
    pub phone_number: String,
    pub display_name: Option<String>,
    pub consent_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct WhatsappBroadcastRecord {
    pub id: Uuid,
    pub edition_id: Option<Uuid>,
    pub broadcast_id: String,
    pub phone_number_id: String,
    pub template_id: String,
    pub edition_date: NaiveDate,
    pub status: String,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub total_recipients: Option<i32>,
    pub sent_count: Option<i32>,
    pub delivered_count: Option<i32>,
    pub read_count: Option<i32>,
    pub failed_count: Option<i32>,
    pub suppressed_count: Option<i32>,
    pub responded_count: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct TriggerBroadcastRequest {
    pub edition_date: Option<NaiveDate>,
    pub send_immediately: Option<bool>,
    pub test_phone: Option<String>,
}

// ============================================================================
// 9. Administrador de Plantillas WhatsApp (Kapso / Meta WABA)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct WhatsappPlantilla {
    pub id: Uuid,
    pub nombre: String,
    pub idioma: String,
    pub activa: bool,
    pub texto_fijo_body: String,
    pub tope_extracto: i32,
    pub parametros: serde_json::Value,
    pub kapso_template_id: Option<String>,
    #[sqlx(default)]
    pub meta_template_id: Option<String>,
    #[sqlx(default)]
    pub status: Option<String>,
    #[sqlx(default)]
    pub category: Option<String>,
    #[sqlx(default)]
    pub components: Option<serde_json::Value>,
    #[sqlx(default)]
    pub quality_score: Option<String>,
    #[sqlx(default)]
    pub phone_number_id: Option<String>,
    #[sqlx(default)]
    pub waba_id: Option<String>,
    #[sqlx(default)]
    pub synced_at: Option<DateTime<Utc>>,
    pub creado_en: DateTime<Utc>,
    pub actualizado_en: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePlantillaRequest {
    pub nombre: String,
    pub idioma: Option<String>,
    pub activa: Option<bool>,
    pub texto_fijo_body: Option<String>,
    pub tope_extracto: Option<i32>,
    pub parametros: Option<serde_json::Value>,
    pub kapso_template_id: Option<String>,
    pub meta_template_id: Option<String>,
    pub status: Option<String>,
    pub category: Option<String>,
    pub components: Option<serde_json::Value>,
    pub quality_score: Option<String>,
    pub phone_number_id: Option<String>,
    pub waba_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePlantillaRequest {
    pub nombre: Option<String>,
    pub idioma: Option<String>,
    pub activa: Option<bool>,
    pub texto_fijo_body: Option<String>,
    pub tope_extracto: Option<i32>,
    pub parametros: Option<serde_json::Value>,
    pub kapso_template_id: Option<String>,
    pub meta_template_id: Option<String>,
    pub status: Option<String>,
    pub category: Option<String>,
    pub components: Option<serde_json::Value>,
    pub quality_score: Option<String>,
    pub phone_number_id: Option<String>,
    pub waba_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ProbarPlantillaRequest {
    pub telefono: String,
    pub extracto_prueba: Option<String>,
}
