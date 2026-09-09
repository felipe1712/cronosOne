# Guía de Despliegue en Producción — ExposureIQ
## Subdominio: `monitoreo.causer.com.mx`
### Servidor compartido con SentinelIQ (Ubuntu / Debian Linux)

Esta guía detalla paso a paso cómo montar **ExposureIQ** en el mismo servidor donde ya opera **SentinelIQ**, reutilizando de forma segura los servicios compartidos (**Surya OCR**, **WAHA**, **PostgreSQL** y **n8n**) manteniendo **aislamiento absoluto** a nivel de base de datos, backend, frontend y dominio.

---

## 1. Mapa de Puertos y Servicios en el Servidor

| Servicio | Puerto / Host | Proyecto | Comentarios |
| :--- | :--- | :--- | :--- |
| **Nginx (Reverse Proxy)** | 80 / 443 | Global | Enruta `monitoreo.causer.com.mx` con SSL Let's Encrypt |
| **PostgreSQL (Motor)** | 5432 | Compartido | Base de datos separada: `exposureiq_db` |
| **Surya OCR** | 5000 (o socket) | Compartido | API interna consumida por los workers de ExposureIQ |
| **WAHA (WhatsApp API)** | 3000 | Compartido | Instancia de WhatsApp ya vinculada |
| **n8n (Orquestador)** | 5678 | Compartido | Flujo independiente importado para sondeo de cola |
| **ExposureIQ Backend (Rust)** | **8090** | ExposureIQ | API Axum escuchando en localhost:8090 |
| **ExposureIQ Workers (Python)** | **8091** | ExposureIQ | Procesamiento en background en localhost:8091 |
| **ExposureIQ Frontend (Next.js)**| **3010** | ExposureIQ | Aplicación web en localhost:3010 |

---

## 2. Paso 1: Configuración de DNS
En tu panel de DNS (Cloudflare, GoDaddy, Namecheap, etc.):
- Añade un registro **tipo A**:
  - **Nombre / Host:** `monitoreo`
  - **Valor / IP:** `<IP_PUBLICA_DE_TU_SERVIDOR>`
  - **TTL:** Automático / 300

---

## 3. Paso 2: Base de Datos PostgreSQL Separada

Ejecuta estos comandos directamente en la terminal de tu servidor (no necesitas entrar al prompt interactivo de psql):

```bash
# 1. Crear la base de datos exclusiva para ExposureIQ
sudo -u postgres psql -c "CREATE DATABASE exposureiq_db;"

# 2. Aplicar migraciones iniciales, semillas y tabla de sesión WhatsApp
cd /opt/CronosOne/backend/migrations
sudo -u postgres psql -d exposureiq_db -f 20260908000001_init_schema.sql
sudo -u postgres psql -d exposureiq_db -f 20260908000002_seed_data.sql
sudo -u postgres psql -d exposureiq_db -f 20260908000003_whatsapp_sesion.sql
```

> **Nota:** La migración `01_init_schema.sql` ya incluye la habilitación automática de la extensión `pgcrypto`. La migración `03_whatsapp_sesion.sql` crea la tabla que almacena el QR y estado recibidos vía webhook desde n8n.

---

## 4. Paso 3: Despliegue de ExposureIQ en el Servidor

Clona o copia la carpeta del proyecto a `/opt/CronosOne` (o el directorio que uses para tus proyectos):

### 4.1. Compilación y Configuración del Backend (Rust)
```bash
cd /opt/CronosOne/backend
cp .env.example .env
nano .env
```
Ajusta las variables en `.env`:
```ini
HOST=127.0.0.1
PORT=8090
DATABASE_URL=postgres://postgres:<TU_PASSWORD_POSTGRES>@127.0.0.1:5432/exposureiq_db?sslmode=disable
JWT_SECRET=<GENERA_UNA_CLAVE_ALEATORIA_SEGURA>
UPLOAD_DIR=/opt/CronosOne/backend/uploads
WORKER_BASE_URL=http://127.0.0.1:8091
WAHA_BASE_URL=http://127.0.0.1:3000
WAHA_SESSION=default
DIRECTOR_WHATSAPP_PHONE=5215512345678
```

Compila el binario optimizado en release:
```bash
cargo build --release
```

### 4.2. Configuración de los Workers (Python)
```bash
cd /opt/CronosOne/workers
cp .env.example .env
nano .env
```
Ajusta en `.env`:
```ini
HOST=127.0.0.1
PORT=8001
DATABASE_URL=postgres://postgres:<TU_PASSWORD_POSTGRES>@127.0.0.1:5432/exposureiq_db
ANTHROPIC_API_KEY=<TU_CLAVE_CLAUDE_API>
CLAUDE_MODEL=claude-3-5-sonnet-20241022
SURYA_OCR_URL=http://127.0.0.1:5000/ocr
DIRECTOR_WHATSAPP_PHONE=5215512345678
```

Instala dependencias en entorno virtual:
```bash
python3 -m venv venv
source venv/bin/activate
pip install --upgrade pip
pip install -r requirements.txt
```

### 4.3. Configuración y Compilación del Frontend (Next.js)
```bash
cd /opt/CronosOne/frontend
echo "NEXT_PUBLIC_API_URL=https://monitoreo.causer.com.mx/api" > .env.local
npm install --legacy-peer-deps
npm run build
```

---

## 5. Paso 4: Configuración de Servicios en Segundo Plano (Systemd)

Crea 3 servicios para que arranquen automáticamente y se reinicien ante fallos.

### 5.1. Backend (`/etc/systemd/system/exposureiq-backend.service`)
```ini
[Unit]
Description=ExposureIQ Backend API (Rust Axum)
After=network.target postgresql.service

[Service]
Type=simple
User=root
WorkingDirectory=/opt/CronosOne/backend
ExecStart=/opt/CronosOne/backend/target/release/exposureiq-backend
Restart=always
RestartSec=5
EnvironmentFile=/opt/CronosOne/backend/.env

[Install]
WantedBy=multi-user.target
```

### 5.2. Workers (`/etc/systemd/system/exposureiq-workers.service`)
```ini
[Unit]
Description=ExposureIQ Python Workers (Surya OCR & Claude LLM)
After=network.target postgresql.service

[Service]
Type=simple
User=root
WorkingDirectory=/opt/CronosOne/workers
ExecStart=/opt/CronosOne/workers/venv/bin/uvicorn main:app --host 127.0.0.1 --port 8091
Restart=always
RestartSec=5
EnvironmentFile=/opt/CronosOne/workers/.env

[Install]
WantedBy=multi-user.target
```

### 5.3. Frontend (`/etc/systemd/system/exposureiq-frontend.service`)
```ini
[Unit]
Description=ExposureIQ Frontend (Next.js 15)
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=/opt/CronosOne/frontend
ExecStart=/usr/bin/npm start -- -p 3010
Restart=always
RestartSec=5
Environment=PORT=3010
Environment=NODE_ENV=production

[Install]
WantedBy=multi-user.target
```

Activa y arranca los servicios:
```bash
sudo systemctl daemon-reload
sudo systemctl enable --now exposureiq-backend exposureiq-workers exposureiq-frontend
sudo systemctl status exposureiq-backend exposureiq-workers exposureiq-frontend
```

---

## 6. Paso 5: Configuración de Nginx Reverse Proxy y Certificado SSL

Crea la configuración para el subdominio:

```bash
sudo nano /etc/nginx/sites-available/monitoreo.causer.com.mx
```

Pega la siguiente configuración:

```nginx
server {
    listen 80;
    server_name monitoreo.causer.com.mx;

    # Límite de carga para boletines PDF pesados (hasta 100MB)
    client_max_body_size 100M;

    # Enrutar API hacia el Backend en Rust
    location /api/ {
        proxy_pass http://127.0.0.1:8090/api/;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_read_timeout 300s;
        proxy_connect_timeout 300s;
    }

    # Enrutar todo lo demás al Frontend Next.js
    location / {
        proxy_pass http://127.0.0.1:3010;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

Habilita el sitio y recarga Nginx:
```bash
sudo ln -s /etc/nginx/sites-available/monitoreo.causer.com.mx /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl reload nginx
```

Genera el certificado SSL gratuito con Let's Encrypt / Certbot:
```bash
sudo certbot --nginx -d monitoreo.causer.com.mx
```

---

## 7. Paso 6: Configurar el Flujo en n8n

1. Abre tu instancia de **n8n** en el navegador (`https://n8n...` o `http://localhost:5678`).
2. Ve a **Workflows** ➔ pulsa los 3 puntos superiores ➔ **Import from File**.
3. Selecciona el archivo:
   `/opt/CronosOne/n8n/exposureiq_waha_polling_workflow.json`
4. Revisa los parámetros:
   - **Nodo Consultar Mensajes Pendientes**: URL `http://127.0.0.1:8080/api/mensajes/pendientes?limit=1&auto_lock=true`
   - **Nodo Enviar WhatsApp vía WAHA**: URL `http://127.0.0.1:3000/api/sendText`
   - **Nodo Confirmar Entrega**: URL `http://127.0.0.1:8080/api/mensajes/{{ $json.id }}/confirmar`
5. Activa el interruptor **Active** del workflow en n8n.

### Sincronización del Estado de WhatsApp y QR desde n8n:
En tu flujo de gestión de WAHA dentro de n8n, cuando recibas el evento de estado o el código QR de WAHA, simplemente haz una petición HTTP POST a:
```
POST http://127.0.0.1:8080/api/webhooks/whatsapp/session
Content-Type: application/json

{
  "session": "default",
  "status": "SCAN_QR_CODE",      // o "CONNECTED", "WORKING", "STOPPED"
  "qr": "data:image/png;base64,...", // o string del QR para mostrarlo en el panel
  "detalles": {}
}
```
Esto guardará inmediatamente el estado y la imagen QR en la base de datos de ExposureIQ para que se muestre en tiempo real en la pantalla **/waha** del panel web.

---

## 8. Verificación Final de Funcionamiento

1. Abre en tu navegador: **`https://monitoreo.causer.com.mx`**
2. Inicia sesión con las credenciales creadas:
   - **Correo:** `admin@exposureiq.internal`
   - **Contraseña:** `Admin1234!`
3. Ve a **Boletines Coparmex**, sube un archivo PDF de prueba y observa:
   - El estado pasa de `en_ocr` ➔ `ocr_completo` ➔ `sintesis_lista`.
   - Se genera el resumen para WhatsApp y queda en cola.
4. En menos de 3 minutos, el cron de n8n detecta el mensaje en `GET /api/mensajes/pendientes`, WAHA lo envía al WhatsApp del Director y la tabla de **Canal WhatsApp** lo muestra como `Entregado (WAHA)`.
5. Ve a **Inteligencia & OSINT**, simula o crea una alerta y pulsa **Validar** para comprobar el despacho prioritario de ciberseguridad.
