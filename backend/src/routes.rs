use axum::{
    middleware,
    routing::{delete, get, patch, post, put},
    Router,
};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use crate::{
    config::Config,
    db::DbPool,
    handlers::{auth, boletines, configuracion, mensajes, osint, scraper, waha},
    middleware::auth::require_auth,
};

pub fn create_router(pool: DbPool, config: Arc<Config>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Rutas protegidas por JWT
    let protected_routes = Router::new()
        // Auth
        .route("/api/auth/me", get(auth::me))
        // Boletines Coparmex
        .route("/api/boletines", get(boletines::list_boletines))
        .route("/api/boletines/upload", post(boletines::upload_boletin))
        .route("/api/boletines/:id", get(boletines::get_boletin))
        .route("/api/boletines/:id/procesar", post(boletines::procesar_boletin))
        .route("/api/boletines/:id/sintesis", put(boletines::actualizar_sintesis))
        .route("/api/boletines/:id/aprobar", post(boletines::aprobar_boletin))
        // Mensajería (historial y despacho en panel)
        .route("/api/mensajes/historial", get(mensajes::list_historial_mensajes))
        .route("/api/mensajes/:id/enviar", post(mensajes::enviar_mensaje_directo))
        .route("/api/mensajes/despachar-cola", post(mensajes::despachar_cola))
        // WAHA
        .route("/api/waha/status", get(waha::get_waha_status))
        .route("/api/waha/qr", get(waha::get_waha_qr))
        .route("/api/waha/restart", post(waha::restart_waha_session))
        // OSINT & Exposición (world-intel-mcp)
        .route("/api/osint/alertas", get(osint::list_alertas))
        .route("/api/osint/alertas/:id/validar", post(osint::validar_alerta))
        .route("/api/osint/entidades", get(osint::list_entidades))
        .route("/api/osint/entidades", post(osint::create_entidad))
        .route("/api/osint/entidades/:id", delete(osint::delete_entidad))
        .route("/api/osint/entidades/:id/toggle", patch(osint::toggle_entidad))
        .route("/api/osint/fuentes", get(osint::list_fuentes_osint))
        .route("/api/osint/fuentes/:id/toggle", patch(osint::toggle_fuente_osint))
        .route("/api/osint/scan", post(osint::run_osint_scan))
        // Configuración General, Modelos de IA y Canal WhatsApp (Kapso)
        .route("/api/configuracion", get(configuracion::get_configuraciones))
        .route("/api/configuracion", put(configuracion::update_configuracion))
        .route("/api/configuracion/test-claude", post(configuracion::test_claude))
        .route("/api/configuracion/test-whatsapp", post(configuracion::test_whatsapp))
        // Lista de Distribución WhatsApp (Destinatarios)
        .route("/api/configuracion/destinatarios", get(configuracion::list_destinatarios))
        .route("/api/configuracion/destinatarios", post(configuracion::create_destinatario))
        .route("/api/configuracion/destinatarios/:id", put(configuracion::update_destinatario))
        .route("/api/configuracion/destinatarios/:id", delete(configuracion::delete_destinatario))
        .route("/api/configuracion/destinatarios/:id/toggle", patch(configuracion::toggle_destinatario))
        // Grupos / Listas de Distribución
        .route("/api/configuracion/grupos", get(configuracion::list_grupos))
        .route("/api/configuracion/grupos", post(configuracion::create_grupo))
        .route("/api/configuracion/grupos/:id", put(configuracion::update_grupo))
        .route("/api/configuracion/grupos/:id", delete(configuracion::delete_grupo))
        // Scraper Automatizado Senado de la República
        .route("/api/scraper/senado/ejecutar", post(scraper::ejecutar_scraper_senado))
        .route("/api/scraper/senado/status", get(scraper::get_scraper_senado_status))
        .layer(middleware::from_fn_with_state(config.clone(), require_auth));

    // Rutas públicas (consumidas por frontend login y cron n8n)
    let public_routes = Router::new()
        .route("/health", get(|| async { "ExposureIQ Backend OK" }))
        .route("/api/auth/login", post(auth::login))
        // Endpoint prioritario de sondeo (polling) para cron de n8n
        .route("/api/mensajes/pendientes", get(mensajes::get_mensajes_pendientes))
        // Webhook para que n8n confirme la entrega
        .route("/api/mensajes/:id/confirmar", post(mensajes::confirmar_mensaje))
        // Webhook para que n8n sincronice estado de sesión y QR de WhatsApp
        .route("/api/webhooks/whatsapp/session", post(waha::webhook_session_update));

    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state((pool, config))
}
