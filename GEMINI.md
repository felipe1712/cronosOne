# Memoria y Reglas Críticas del Proyecto CronosOne (ExposureIQ)

## 1. Arquitectura de Despliegue en el Servidor (`srv1483402` — `monitoreo.causer.com.mx`)

> **REGLA DE ORO DE DESPLIEGUE:** En el servidor de producción (`/opt/cronosOne`), el tráfico web público (enrutado por Nginx) es atendido por **servicios nativos de Systemd (`systemctl`)**, **NO por los contenedores de Docker**.

### Mapa Real de Servicios en Producción:
- **Frontend (`exposureiq-frontend.service`)**:
  - Directorio: `/opt/cronosOne/frontend`
  - Puerto interno: `3010` (`npm start -- -p 3010`)
  - Comando de actualización: `cd /opt/cronosOne/frontend && npm run build && systemctl restart exposureiq-frontend`
- **Backend (`exposureiq-backend.service`)**:
  - Directorio: `/opt/cronosOne/backend`
  - Puerto interno: `8090` (`target/release/exposureiq-backend`)
  - Comando de actualización: `cd /opt/cronosOne/backend && cargo build --release && systemctl restart exposureiq-backend`
- **Workers Python (`exposureiq-workers.service`)**:
  - Directorio: `/opt/cronosOne/workers`
  - Puerto interno: `8091` (`venv/bin/uvicorn main:app --host 127.0.0.1 --port 8091`)
  - Comando de actualización: `systemctl restart exposureiq-workers`
- **Surya OCR (`exposureiq-surya.service`)**:
  - Puerto interno: `5000`

*(Nota: Aunque en el servidor puedan existir contenedores de Docker corriendo en los puertos `3100`, `8180` y `8181`, Nginx apunta a los servicios de Systemd. Reconstruir solo Docker NO actualiza el sitio en producción).*

### Protocolo Obligatorio antes de dar instrucciones de despliegue:
1. **Verificar Git Local**: Confirmar siempre con `git status` que todos los cambios estén en `git commit` y subidos con `git push origin expo-hls-ocr` antes de indicar un `git pull` en el servidor.
2. **Verificar Compilación**: Asegurar que no falte ningún `import` (especialmente iconos de `@mui/icons-material` en archivos `.tsx`) y que `cargo check` pase sin errores.
3. **Conflicto de `Cargo.lock` en el servidor**: Al compilar con `cargo build --release` en el servidor se modifica/crea `backend/Cargo.lock`. Antes de `git pull`, indicar `rm -f backend/Cargo.lock` o `git checkout -- backend/Cargo.lock`.
4. **Comandos estándar de despliegue en el servidor**:
   ```bash
   cd /opt/cronosOne
   rm -f backend/Cargo.lock
   git pull origin expo-hls-ocr

   # Actualizar Backend
   cd /opt/cronosOne/backend && cargo build --release && systemctl restart exposureiq-backend

   # Actualizar Frontend
   cd /opt/cronosOne/frontend && npm run build && systemctl restart exposureiq-frontend

   # Reiniciar Workers
   systemctl restart exposureiq-workers
   ```

---

## 2. Reglas de WhatsApp Cloud API (Kapso / Meta)

1. **Nunca enviar síntesis ni pruebas como texto libre (`type: "text"`)**:
   - Meta rechaza cualquier mensaje que no sea plantilla fuera de la ventana de 24 horas (`Cannot send non-template messages outside the 24-hour window`).
   - Todo envío de boletín, síntesis diaria o prueba debe ir como `"type": "template"` usando `send_brief_template`.
2. **Plantillas Oficiales en WABA (`1280554003783822`) y Sincronización con Kapso**:
   - Administradas en la tabla `whatsapp_plantillas` (seleccionables mediante ComboBox en las pestañas **"Canal WhatsApp"** y **"Plantillas WhatsApp"** en `/configuracion`).
   - Sincronización:
     1. `POST https://api.kapso.ai/platform/v1/whatsapp_templates/sync`
     2. `GET https://api.kapso.ai/meta/whatsapp/v24.0/1280554003783822/message_templates` (WABA ID `1280554003783822`, Phone Number ID `759195320601997`).
   - Guarda localmente `meta_template_id`, `kapso_template_id`, `status` (`APPROVED`), `category`, `components`, `quality_score`, `phone_number_id`, `waba_id` y `synced_at`.
   - Si una plantilla tiene **0 variables** (ej. `brief_informativo`, Meta ID `784203120908608`), **se omite por completo `components`** en el payload. Si tiene variables (ej. `brief_informativo_cp_v2`), envía `header` (`date`) y `body` (`primeras_planas`).
3. **Tabla `extractos`, Límite de Caracteres y Regla `#132018` de Meta**:
   - El cuerpo total de la plantilla en Meta no puede superar los **1,024 caracteres** (tope por defecto del extracto: `900` caracteres).
   - **Regla crítica `#132018` de Meta Graph API**: Los valores dentro de `template.components[].parameters[].text` **NO pueden contener saltos de línea (`\n`), tabuladores (`\t`) ni más de 4 espacios consecutivos** (`Param text cannot have new-line/tab characters or more than 4 consecutive spaces`).
   - Aunque el editor en la UI permita editar con saltos de línea para comodidad visual, `KapsoClient::sanitize_template_param` aplana automáticamente los saltos de línea a separadores en línea (`  ` o ` • `), conserva los emojis UTF-8 y negritas (`*texto*`), y colapsa espacios múltiples antes de enviar a Meta.

---

## 3. Reglas del Portal Público y Menú
1. **Segmentación en 4 Secciones del Portal Público (`/brief/[section_key]`)**: Cada edición diaria segmenta su contenido especializado sin duplicar texto en las 4 secciones activas: `primeras_planas`, `politica`, `economia` y `seguridad` (tanto al consolidar con Claude en `workers/llm_synthesis.py` como mediante auto-segmentación temática en `backend/src/handlers/cms.rs`).
2. **Menú Lateral (`LeftSidebarMenu.tsx`)**: Las opciones `"Inteligencia & OSINT"` (`/osint`) y `"Canal WhatsApp"` (`/waha`) deben permanecer ocultas/comentadas. Toda la telemetría, salud del webhook y configuración de Kapso vive en `/configuracion` (pestaña **"Salud & Telemetría Kapso"** y **"Plantillas WhatsApp"**).
