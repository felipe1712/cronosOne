FROM python:3.11-slim

WORKDIR /app

# Dependencias del sistema para compilación y bibliotecas compartidas requeridas por Firefox/Camoufox
RUN apt-get update && apt-get install -y --no-install-recommends \
    gcc \
    libpq-dev \
    curl \
    ca-certificates \
    libgtk-3-0 \
    libdbus-glib-1-2 \
    libxt6 \
    libasound2 \
    libx11-xcb1 \
    libxcomposite1 \
    libxcursor1 \
    libxdamage1 \
    libxfixes3 \
    libxi6 \
    libxrandr2 \
    libxrender1 \
    libxtst6 \
    libgl1-mesa-dri \
    libgbm1 \
    libegl1 \
    fonts-liberation \
    fontconfig \
    && rm -rf /var/lib/apt/lists/*

COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# Pre-descarga de binarios de Camoufox durante el build para evitar descargas en tiempo de ejecución
RUN python -m camoufox fetch || true

COPY . .

EXPOSE 8001
CMD ["uvicorn", "main:app", "--host", "0.0.0.0", "--port", "8001"]
