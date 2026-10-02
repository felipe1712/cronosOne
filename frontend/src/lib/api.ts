/**
 * Cliente HTTP para comunicarse con la API Rust de ExposureIQ
 */

const API_BASE = process.env.NEXT_PUBLIC_API_URL || '/api';

export function getToken(): string | null {
  if (typeof window === 'undefined') return null;
  return localStorage.getItem('exposureiq_token');
}

export function setToken(token: string) {
  if (typeof window !== 'undefined') {
    localStorage.setItem('exposureiq_token', token);
  }
}

export function removeToken() {
  if (typeof window !== 'undefined') {
    localStorage.removeItem('exposureiq_token');
    localStorage.removeItem('exposureiq_user');
  }
}

export async function apiFetch<T>(endpoint: string, options: RequestInit = {}): Promise<T> {
  const token = getToken();
  const headers = new Headers(options.headers || {});

  if (token && !headers.has('Authorization')) {
    headers.set('Authorization', `Bearer ${token}`);
  }

  if (!headers.has('Content-Type') && !(options.body instanceof FormData)) {
    headers.set('Content-Type', 'application/json');
  }

  const response = await fetch(`${API_BASE}${endpoint}`, {
    ...options,
    headers,
  });

  if (!response.ok) {
    if (response.status === 401) {
      removeToken();
      if (typeof window !== 'undefined' && !window.location.pathname.startsWith('/authentication')) {
        window.location.href = '/authentication/sign-in';
      }
      throw new Error('Sesión no autorizada o credenciales inválidas. Inicie sesión.');
    }

    let errorMsg = `Error HTTP ${response.status}`;
    try {
      const errJson = await response.json();
      if (errJson.error) errorMsg = errJson.error;
    } catch {
      // Ignorar fallback
    }
    throw new Error(errorMsg);
  }

  return response.json();
}

// ============================================================================
// Tipos y Modelos
// ============================================================================

export interface UpdateConfiguracionPayload {
  claude_model?: string;
  system_prompt?: string;
  whatsapp_provider?: string;
  kapso_api_key?: string;
  kapso_phone_number_id?: string;
  director_whatsapp_phone?: string;
  whatsapp_template_name?: string;
  whatsapp_template_language?: string;
}

export interface GrupoDistribucion {
  id: string;
  nombre: string;
  descripcion?: string;
  color: string;
  activo: boolean;
  creado_en?: string;
  total_miembros?: number;
}

export interface CreateGrupoPayload {
  nombre: string;
  descripcion?: string;
  color?: string;
  activo?: boolean;
}

export interface UpdateGrupoPayload {
  nombre: string;
  descripcion?: string;
  color?: string;
  activo: boolean;
}

export interface GrupoResumen {
  id: string;
  nombre: string;
  color: string;
}

export interface Destinatario {
  id: string;
  nombre: string;
  telefono: string;
  cargo?: string;
  activo: boolean;
  notas?: string;
  creado_en?: string;
  actualizado_en?: string;
  grupos?: GrupoResumen[];
  plantilla_enviada?: boolean;
  plantilla_error?: string;
  kapso_message_id?: string;
}

export interface CreateDestinatarioPayload {
  nombre: string;
  telefono: string;
  cargo?: string;
  notas?: string;
  activo?: boolean;
  grupo_ids?: string[];
  enviar_plantilla?: boolean;
  template_name?: string;
  template_language?: string;
  template_variables?: string[];
}

export interface UpdateDestinatarioPayload {
  nombre: string;
  telefono: string;
  cargo?: string;
  notas?: string;
  activo: boolean;
  grupo_ids?: string[];
}

export interface SintesisDiariaResumen {
  fecha: string;
  total_documentos: number;
  total_senado: number;
  total_manual: number;
  total_paginas: number;
  sintesis_id?: string;
  estado_sintesis?: string;
  texto_preview?: string;
  modelo_usado?: string;
  actualizado_en?: string;
}

export interface BoletinEnWorkspace {
  id: string;
  fecha_boletin: string;
  nombre_archivo: string;
  ruta_archivo: string;
  estado: string;
  origen: string;
  incluido_en_sintesis: boolean;
  total_paginas?: number;
  error_mensaje?: string;
  creado_en: string;
}

export interface SintesisDiaria {
  id: string;
  fecha: string;
  texto: string;
  temas?: string[];
  documentos_ids?: string[];
  estado: string;
  modelo_usado?: string;
  tokens_usados?: number;
  aprobado_por?: string;
  creado_en: string;
  actualizado_en: string;
}

export interface WorkspaceFechaResponse {
  fecha: string;
  documentos: BoletinEnWorkspace[];
  sintesis?: SintesisDiaria;
  total_documentos: number;
  documentos_incluidos: number;
}

export interface SeccionSenado {
  id: string;
  nombre: string;
  archivo: string;
  orden: number;
}

// ============================================================================
// Servicios de API
// ============================================================================

export const ApiService = {
  // Autenticación
  login: (correo: string, password: string) =>
    apiFetch<{ token: string; usuario: any }>('/auth/login', {
      method: 'POST',
      body: JSON.stringify({ correo, password }),
    }),

  getMe: () => apiFetch<any>('/auth/me'),

  // Boletines Coparmex
  getBoletines: (limit = 20, offset = 0) =>
    apiFetch<any[]>(`/boletines?limit=${limit}&offset=${offset}`),

  getBoletinDetail: (id: string) => apiFetch<any>(`/boletines/${id}`),

  uploadBoletin: (formData: FormData) =>
    apiFetch<any>('/boletines/upload', {
      method: 'POST',
      body: formData,
    }),

  procesarBoletin: (id: string) =>
    apiFetch<any>(`/boletines/${id}/procesar`, {
      method: 'POST',
    }),

  actualizarSintesis: (id: string, texto: string) =>
    apiFetch<any>(`/boletines/${id}/sintesis`, {
      method: 'PUT',
      body: JSON.stringify({ texto }),
    }),

  aprobarBoletin: (id: string, texto: string) =>
    apiFetch<any>(`/boletines/${id}/aprobar`, {
      method: 'POST',
      body: JSON.stringify({ texto }),
    }),

  // Mensajería y Despacho WhatsApp
  getHistorialMensajes: () => apiFetch<any[]>('/mensajes/historial'),

  enviarMensajeCola: (id: string, payload: { telefono?: string; api_key?: string; phone_number_id?: string } = {}) =>
    apiFetch<any>(`/mensajes/${id}/enviar`, {
      method: 'POST',
      body: JSON.stringify(payload),
    }),

  despacharColaPendiente: (payload: { telefono?: string; api_key?: string; phone_number_id?: string } = {}) =>
    apiFetch<any>('/mensajes/despachar-cola', {
      method: 'POST',
      body: JSON.stringify(payload),
    }),

  getWahaStatus: () => apiFetch<any>('/waha/status'),

  getWahaQr: () => apiFetch<any>('/waha/qr'),

  restartWaha: () =>
    apiFetch<any>('/waha/restart', {
      method: 'POST',
    }),

  // OSINT y Exposición Digital
  getAlertas: (estado?: string, severidad?: string) => {
    const params = new URLSearchParams();
    if (estado) params.append('estado', estado);
    if (severidad) params.append('severidad', severidad);
    return apiFetch<any[]>(`/osint/alertas?${params.toString()}`);
  },

  validarAlerta: (id: string, accion: 'validar' | 'descartar', notas?: string) =>
    apiFetch<any>(`/osint/alertas/${id}/validar`, {
      method: 'POST',
      body: JSON.stringify({ accion, notas }),
    }),

  getEntidades: () => apiFetch<any[]>('/osint/entidades'),

  createEntidad: (tipo: string, valor: string, descripcion?: string) =>
    apiFetch<any>('/osint/entidades', {
      method: 'POST',
      body: JSON.stringify({ tipo, valor, descripcion }),
    }),

  toggleEntidad: (id: string) =>
    apiFetch<any>(`/osint/entidades/${id}/toggle`, {
      method: 'PATCH',
    }),

  deleteEntidad: (id: string) =>
    apiFetch<any>(`/osint/entidades/${id}`, {
      method: 'DELETE',
    }),

  // Fuentes OSINT (world-intel-mcp)
  getFuentesOsint: () => apiFetch<any[]>('/osint/fuentes'),

  toggleFuenteOsint: (id: string) =>
    apiFetch<any>(`/osint/fuentes/${id}/toggle`, {
      method: 'PATCH',
    }),

  runOsintScan: () =>
    apiFetch<any>('/osint/scan', {
      method: 'POST',
    }),

  // Configuración del Sistema & Modelos de IA
  getConfiguraciones: () => apiFetch<any>('/configuracion'),

  updateConfiguracion: (payload: UpdateConfiguracionPayload | string) => {
    const body = typeof payload === 'string' ? { claude_model: payload } : payload;
    return apiFetch<any>('/configuracion', {
      method: 'PUT',
      body: JSON.stringify(body),
    });
  },

  testClaude: (model?: string) =>
    apiFetch<any>('/configuracion/test-claude', {
      method: 'POST',
      body: JSON.stringify({ model }),
    }),

  testWhatsapp: (payload: { phone?: string; message?: string; api_key?: string; phone_number_id?: string } = {}) =>
    apiFetch<any>('/configuracion/test-whatsapp', {
      method: 'POST',
      body: JSON.stringify(payload),
    }),

  testTemplate: (payload: { phone?: string; template_name?: string; template_language?: string; variables?: string[]; api_key?: string; phone_number_id?: string } = {}) =>
    apiFetch<any>('/configuracion/test-template', {
      method: 'POST',
      body: JSON.stringify(payload),
    }),

  // Lista de Distribución WhatsApp (Destinatarios)
  getDestinatarios: () => apiFetch<Destinatario[]>('/configuracion/destinatarios'),

  createDestinatario: (payload: CreateDestinatarioPayload) =>
    apiFetch<Destinatario>('/configuracion/destinatarios', {
      method: 'POST',
      body: JSON.stringify(payload),
    }),

  updateDestinatario: (id: string, payload: UpdateDestinatarioPayload) =>
    apiFetch<Destinatario>(`/configuracion/destinatarios/${id}`, {
      method: 'PUT',
      body: JSON.stringify(payload),
    }),

  toggleDestinatario: (id: string) =>
    apiFetch<Destinatario>(`/configuracion/destinatarios/${id}/toggle`, {
      method: 'PATCH',
    }),

  deleteDestinatario: (id: string) =>
    apiFetch<{ mensaje: string }>(`/configuracion/destinatarios/${id}`, {
      method: 'DELETE',
    }),

  enviarPlantillaDestinatario: (id: string, payload: { template_name?: string; template_language?: string; variables?: string[]; api_key?: string; phone_number_id?: string } = {}) =>
    apiFetch<any>(`/configuracion/destinatarios/${id}/enviar-plantilla`, {
      method: 'POST',
      body: JSON.stringify(payload),
    }),

  // Grupos / Listas de Distribución
  getGrupos: () => apiFetch<GrupoDistribucion[]>('/configuracion/grupos'),

  createGrupo: (payload: CreateGrupoPayload) =>
    apiFetch<GrupoDistribucion>('/configuracion/grupos', {
      method: 'POST',
      body: JSON.stringify(payload),
    }),

  updateGrupo: (id: string, payload: UpdateGrupoPayload) =>
    apiFetch<GrupoDistribucion>(`/configuracion/grupos/${id}`, {
      method: 'PUT',
      body: JSON.stringify(payload),
    }),

  deleteGrupo: (id: string) =>
    apiFetch<{ mensaje: string }>(`/configuracion/grupos/${id}`, {
      method: 'DELETE',
    }),

  // Scraper Automatizado Senado de la República
  ejecutarScraperSenado: (fecha?: string) =>
    apiFetch<{ status: string; mensaje: string; fecha?: string }>('/scraper/senado/ejecutar', {
      method: 'POST',
      body: JSON.stringify({ fecha }),
    }),

  ejecutarScraperSenadoConSecciones: (fecha?: string, secciones?: string[]) =>
    apiFetch<{ status: string; mensaje: string; fecha?: string }>('/scraper/senado/ejecutar', {
      method: 'POST',
      body: JSON.stringify({ fecha, secciones }),
    }),

  getScraperSenadoStatus: () =>
    apiFetch<{
      en_progreso: boolean;
      fecha_objetivo?: string;
      navegador_usado?: string;
      archivos_descargados: Array<{ seccion: string; archivo: string; bytes: number; hash: string }>;
      archivos_procesados: Array<{ boletin_id: string; seccion: string; estado: string }>;
      errores: string[];
      ultimo_inicio?: string;
      ultimo_fin?: string;
      mensaje: string;
    }>('/scraper/senado/status'),

  getSeccionesSenado: () =>
    apiFetch<SeccionSenado[]>('/scraper/senado/secciones'),

  // Síntesis Diarias Consolidadas (Monitoreo Ejecutivo por Fecha)
  getFechasSintesis: (limit = 30, offset = 0) =>
    apiFetch<SintesisDiariaResumen[]>(`/sintesis-diarias?limit=${limit}&offset=${offset}`),

  getWorkspaceFecha: (fecha: string) =>
    apiFetch<WorkspaceFechaResponse>(`/sintesis-diarias/${fecha}`),

  uploadMultipleBoletines: (formData: FormData) =>
    apiFetch<{ mensaje: string; fecha: string; documentos: any[] }>('/boletines/upload-multiple', {
      method: 'POST',
      body: formData,
    }),

  toggleDocumentoSeleccion: (id: string, incluido: boolean) =>
    apiFetch<{ id: string; incluido_en_sintesis: boolean }>(`/boletines/${id}/seleccion`, {
      method: 'PATCH',
      body: JSON.stringify({ incluido }),
    }),

  consolidarSintesisFecha: (fecha: string, documentos_ids?: string[]) =>
    apiFetch<any>(`/sintesis-diarias/${fecha}/consolidar`, {
      method: 'POST',
      body: JSON.stringify({ documentos_ids }),
    }),

  actualizarSintesisDiaria: (fecha: string, texto: string) =>
    apiFetch<SintesisDiaria>(`/sintesis-diarias/${fecha}`, {
      method: 'PUT',
      body: JSON.stringify({ texto }),
    }),

  aprobarSintesisDiaria: (fecha: string, texto: string, enviar_whatsapp = true) =>
    apiFetch<{ mensaje: string; sintesis: SintesisDiaria; destinatarios_notificados: number }>(
      `/sintesis-diarias/${fecha}/aprobar`,
      {
        method: 'POST',
        body: JSON.stringify({ texto, enviar_whatsapp }),
      }
    ),
};

