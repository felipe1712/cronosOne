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
2. **Plantilla Oficial Activa (`brief_informativo_cp_v2`, idioma `es_MX`)**:
   - Administrada en la tabla `whatsapp_plantillas` (pestaña **"Plantillas WhatsApp"** en `/configuracion`).
   - Utiliza **parámetros nombrados (`parameter_name`)**:
     - Componente `header`: `parameter_name: "date"` (ej. `"11 de Noviembre"`).
     - Componente `body`: `parameter_name: "primeras_planas"` (contenido de la tabla `extractos`).
3. **Tabla `extractos` y Límite de Caracteres**:
   - El cuerpo total de la plantilla en Meta no puede superar los **1,024 caracteres** (tope por defecto del extracto: `900` caracteres).
   - Se conserva el formato exacto del editor: saltos de línea (`\n`), emojis UTF-8 y negritas (`*texto*`).

---

## 3. Reglas del Portal Público y Menú
1. **Aislamiento de `Primeras Planas`**: La síntesis diaria consolidada se publica exclusivamente en `/brief/primeras_planas` (`section_key = 'primeras_planas'`). Nunca debe sobrescribir las secciones de `politica`, `economia` ni `seguridad`.
2. **Módulo OSINT**: La opción "Inteligencia & OSINT" en `LeftSidebarMenu.tsx` debe permanecer oculta/comentada hasta nuevo aviso.
