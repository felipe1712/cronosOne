import os
import json
import re
import httpx
from typing import List, Dict, Any
from config import settings
from db import get_db_connection

async def fetch_world_intel_events() -> List[Dict[str, Any]]:
    """
    Consulta world-intel-mcp para obtener hallazgos recientes en redes,
    paste sites, foros y marketplaces de dark web.
    """
    try:
        async with httpx.AsyncClient(timeout=30.0) as client:
            resp = await client.get(f"{settings.world_intel_mcp_url}/events/recent")
            if resp.status_code == 200:
                return resp.json().get("events", [])
    except Exception as e:
        print(f"[world-intel-mcp] Servicio no disponible en {settings.world_intel_mcp_url} ({e}).")

    # Muestra simulada representativa para pruebas
    return [
        {
            "fuente": "darkweb_forum_breached",
            "titulo": "Posible filtración de base de datos clientes Seguros Alianza",
            "descripcion": "Actor de amenazas 'krypt0x' ofrece lista de 15,000 registros con nombres, RFC y números de póliza.",
            "texto": "Leak database from Mexican insurance Seguros Alianza alianzaseguros.com.mx with policy POL-98421044-MX and executive Carlos Mendoza Silva mentioned.",
            "severidad": "critica",
            "evidencia": {
                "thread_url": "http://breached23...onion/viewtopic.php?id=9021",
                "sample_count": 5,
                "timestamp": "2026-09-08T15:00:00Z"
            }
        }
    ]

def evaluate_rules_and_record():
    """
    Obtiene las entidades activas de la base de datos y evalúa los eventos de inteligencia.
    Si hay coincidencia positiva, inserta una alerta en estado 'por_validar'.
    """
    conn = get_db_connection()
    cur = conn.cursor()

    try:
        cur.execute("SELECT id, tipo, valor FROM entidades_vigiladas WHERE activo = true")
        entidades = cur.fetchall()

        if not entidades:
            print("[OSINT Engine] No hay entidades vigiladas activas.")
            return

        import asyncio
        events = asyncio.run(fetch_world_intel_events())

        alertas_generadas = 0
        for ev in events:
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
                    # Verificar si la alerta ya existe para evitar duplicados
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
                                ev.get("fuente", "osint_feed"),
                                ev.get("titulo", "Hallazgo de exposición"),
                                ev.get("descripcion", ""),
                                ev.get("severidad", "media"),
                                json.dumps(ev.get("evidencia", {}))
                            )
                        )
                        alertas_generadas += 1
                        print(f"[OSINT Engine] 🚨 Alerta encolada por coincidencia con '{ent['valor']}' (por validar)")

        conn.commit()
        print(f"[OSINT Engine] Escaneo finalizado. Nuevas alertas generadas: {alertas_generadas}")
    finally:
        cur.close()
        conn.close()
