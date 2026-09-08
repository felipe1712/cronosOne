/**
 * Cliente HTTP para comunicarse con la API Rust de ExposureIQ
 */

const API_BASE = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080/api';

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

  // Mensajería y WAHA
  getHistorialMensajes: () => apiFetch<any[]>('/mensajes/historial'),

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
};
