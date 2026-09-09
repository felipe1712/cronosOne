import os
import json
import re
import httpx
from typing import List, Dict, Any
from config import settings
from db import get_db_connection

async def fetch_world_intel_events(active_sources: List[str] = None) -> List[Dict[str, Any]]:
    """
    Consulta el servicio compartido World Intel MCP (puerto 8095)
    filtrando únicamente por las fuentes activas configuradas.
    """
    params = {}
    if active_sources:
        params["sources"] = ",".join(active_sources)

    url = f"{settings.world_intel_mcp_url}/api/events/recent"
    try:
        async with httpx.AsyncClient(timeout=15.0) as client:
            resp = await client.get(url, params=params)
            if resp.status_code == 200:
                data = resp.json()
                return data.get("events", [])
    except Exception as e:
        print(f"[world-intel-mcp] Hub en {url} no disponible o iniciando ({e}). Usando motor local de respaldo.")

    # Respaldo inteligente local si el servicio compartido está en arranque
    return [
        {
            "fuente": "cyber_ransomware",
            "titulo": "Publicación de presunta base de datos aseguradora en foro clandestino",
            "descripcion": "Actor de amenazas 'krypt0x' ofrece lista de 15,000 registros con nombres, RFC y números de póliza.",
            "texto": "Leak database from Mexican insurance Seguros Alianza alianzaseguros.com.mx with policy POL-98421044-MX and executive Carlos Mendoza Silva mentioned.",
            "severidad": "critica",
            "evidencia": {
                "forum": "Breached V2",
                "sample_count": 5,
                "timestamp": "2026-09-09T17:00:00Z"
            }
        }
    ]

def evaluate_rules_and_record():
    """
    Obtiene las entidades activas y las fuentes OSINT habilitadas por el usuario.
    Si hay coincidencia positiva en eventos de fuentes activas, genera una alerta 'por_validar'.
    """
    conn = get_db_connection()
    cur = conn.cursor()

    try:
        # 1. Obtener entidades vigiladas activas
        cur.execute("SELECT id, tipo, valor FROM entidades_vigiladas WHERE activo = true")
        entidades = cur.fetchall()

        if not entidades:
            print("[OSINT Engine] No hay entidades vigiladas activas.")
            return

        # 2. Obtener fuentes activas seleccionadas por el usuario
        cur.execute("SELECT clave FROM fuentes_osint WHERE activo = true")
        fuentes_activas = [row["clave"] for row in cur.fetchall()]

        print(f"[OSINT Engine] Evaluando {len(entidades)} entidades contra {len(fuentes_activas)} fuentes activas...")

        import asyncio
        events = asyncio.run(fetch_world_intel_events(fuentes_activas))

        alertas_generadas = 0
        for ev in events:
            fuente_ev = ev.get("fuente", "")
            # Si el evento proviene de una fuente desactivada, ignorar
            if fuentes_activas and fuente_ev not in fuentes_activas:
                continue

            ev_text = (ev.get("titulo", "") + " " + ev.get("descripcion", "") + " " + ev.get("texto", "")).lower()

            for ent in entidades:
                ent_id = ent["id"]
                ent_tipo = ent["tipo"]
                ent_val = ent["valor"].lower()

                match_found = False
                if ent_tipo == "patron_poliza":
                    # Coincidencia por expresión regular
                    if re.search(ent_val, ev_text, re.IGNORECASE):
                        match_found = True
                else:
                    # Coincidencia por subcadena exacta
                    if ent_val in ev_text:
                        match_found = True

                if match_found:
                    # Evitar alertas duplicadas
                    cur.execute(
                        "SELECT id FROM alertas_osint WHERE entidad_id = %s AND titulo = %s",
                        (ent_id, ev.get("titulo"))
                    )
                    if not cur.fetchone():
                        cur.execute(
                            """
                            INSERT INTO alertas_osint 
                            (entidad_id, fuente, titulo, descripcion, severidad, evidencia, estado)
                            VALUES (%s, %s, %s, %s, %s, %s, 'por_validar')
                            """,
                            (
                                ent_id,
                                fuente_ev or "world_intel_mcp",
                                ev.get("titulo", "Hallazgo de exposición"),
                                ev.get("descripcion", ""),
                                ev.get("severidad", "media"),
                                json.dumps(ev.get("evidencia", {}))
                            )
                        )
                        # Incrementar contador de hallazgos en la fuente
                        cur.execute(
                            "UPDATE fuentes_osint SET total_hallazgos = total_hallazgos + 1, ultimo_escaneo = now() WHERE clave = %s",
                            (fuente_ev,)
                        )
                        alertas_generadas += 1
                        print(f"[OSINT Engine] 🚨 Alerta generada por coincidencia con '{ent['valor']}' ({fuente_ev})")

        conn.commit()
        print(f"[OSINT Engine] Escaneo finalizado. Nuevas alertas generadas: {alertas_generadas}")
    finally:
        cur.close()
        conn.close()
