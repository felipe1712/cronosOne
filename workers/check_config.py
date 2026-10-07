#!/usr/bin/env python3
"""
Herramienta Forense de Diagnóstico de Configuración en PostgreSQL
Ejecutar en el servidor con:
  cd /opt/cronosOne/workers
  venv/bin/python check_config.py
"""

import sys
import os
from pathlib import Path
from dotenv import load_dotenv

# Cargar .env de workers
env_path = Path(__file__).resolve().parent / ".env"
load_dotenv(dotenv_path=env_path)

def main():
    print("=" * 70)
    print("🔍 DIAGNÓSTICO FORENSE DE CONFIGURACIÓN Y BASE DE DATOS")
    print("=" * 70)

    try:
        from config import settings
        from db import get_db_connection
    except Exception as e:
        print(f"❌ Error importando módulos de workers: {e}")
        sys.exit(1)

    print(f"📌 DATABASE_URL configurada: {settings.database_url.split('@')[-1] if '@' in settings.database_url else settings.database_url}")

    try:
        conn = get_db_connection()
        cur = conn.cursor()
        print("✅ Conexión a PostgreSQL establecida con éxito.")
    except Exception as e:
        print(f"❌ Fallo al conectar a PostgreSQL: {e}")
        sys.exit(1)

    # 1. Verificar tabla configuraciones_sistema
    print("\n" + "-" * 70)
    print("📋 TABLA: configuraciones_sistema")
    print("-" * 70)
    try:
        cur.execute("""
            SELECT table_name 
            FROM information_schema.tables 
            WHERE table_schema = 'public' AND table_name = 'configuraciones_sistema';
        """)
        if not cur.fetchone():
            print("⚠️ La tabla 'configuraciones_sistema' NO existe en la base de datos.")
        else:
            cur.execute("SELECT clave, valor, actualizado_en FROM configuraciones_sistema ORDER BY clave;")
            rows = cur.fetchall()
            print(f"Total de registros encontrados: {len(rows)}")
            if rows:
                for r in rows:
                    val = r['valor']
                    val_preview = (val[:80] + '...') if len(val) > 80 else val
                    # ocultar parte de la api key si existe
                    if 'KEY' in r['clave'] and len(val) > 8:
                        val_preview = val[:4] + '****' + val[-4:]
                    print(f"  • [{r['clave']}]: {val_preview} (actualizado: {r['actualizado_en']})")
            else:
                print("  ℹ️ La tabla está VACÍA (0 registros).")
    except Exception as e:
        print(f"❌ Error consultando configuraciones_sistema: {e}")

    # 2. Verificar tabla lista_distribucion
    print("\n" + "-" * 70)
    print("📱 TABLA: lista_distribucion (Destinatarios WhatsApp)")
    print("-" * 70)
    try:
        cur.execute("""
            SELECT table_name 
            FROM information_schema.tables 
            WHERE table_schema = 'public' AND table_name = 'lista_distribucion';
        """)
        if not cur.fetchone():
            print("⚠️ La tabla 'lista_distribucion' NO existe en la base de datos.")
        else:
            cur.execute("SELECT id, nombre, telefono, cargo, activo FROM lista_distribucion ORDER BY creado_en ASC;")
            rows = cur.fetchall()
            print(f"Total de destinatarios encontrados: {len(rows)}")
            if rows:
                for r in rows:
                    estado = "Activo" if r['activo'] else "Inactivo"
                    cargo = f"({r['cargo']})" if r.get('cargo') else ""
                    print(f"  • {r['nombre']} {cargo} -> Tel: {r['telefono']} [{estado}]")
            else:
                print("  ℹ️ No hay destinatarios registrados.")
    except Exception as e:
        print(f"❌ Error consultando lista_distribucion: {e}")

    # 3. Verificar tabla grupos_distribucion
    print("\n" + "-" * 70)
    print("👥 TABLA: grupos_distribucion (Listas de Envío)")
    print("-" * 70)
    try:
        cur.execute("""
            SELECT table_name 
            FROM information_schema.tables 
            WHERE table_schema = 'public' AND table_name = 'grupos_distribucion';
        """)
        if not cur.fetchone():
            print("⚠️ La tabla 'grupos_distribucion' NO existe en la base de datos.")
        else:
            cur.execute("SELECT id, nombre, color, activo FROM grupos_distribucion ORDER BY nombre ASC;")
            rows = cur.fetchall()
            print(f"Total de grupos encontrados: {len(rows)}")
            if rows:
                for r in rows:
                    print(f"  • [{r['color']}] {r['nombre']}")
            else:
                print("  ℹ️ No hay grupos registrados.")
    except Exception as e:
        print(f"❌ Error consultando grupos_distribucion: {e}")

    # 4. Verificar tablas existentes en la BD
    print("\n" + "-" * 70)
    print("🗄️ TODAS LAS TABLAS EXISTENTES EN EXPOSUREIQ_DB:")
    print("-" * 70)
    try:
        cur.execute("""
            SELECT table_name 
            FROM information_schema.tables 
            WHERE table_schema = 'public' 
            ORDER BY table_name;
        """)
        tables = [t['table_name'] for t in cur.fetchall()]
        print(f"Tablas ({len(tables)}): {', '.join(tables)}")
    except Exception as e:
        print(f"❌ Error listando tablas: {e}")

    # 5. Documentos en boletines y fechas en sintesis_diarias
    print("\n" + "-" * 70)
    print("📑 DOCUMENTOS Y SÍNTESIS REGISTRADAS:")
    print("-" * 70)
    try:
        cur.execute("SELECT COUNT(*), COUNT(DISTINCT fecha_boletin) FROM boletines;")
        doc_count = cur.fetchone()
        print(f"Total documentos en 'boletines': {doc_count['count']} en {doc_count['count_1']} fechas distintas.")

        cur.execute("SELECT fecha, estado, modelo_usado FROM sintesis_diarias ORDER BY fecha DESC LIMIT 5;")
        sintesis = cur.fetchall()
        print(f"Últimas síntesis diarias registradas ({len(sintesis)}):")
        for s in sintesis:
            print(f"  • {s['fecha']}: {s['estado']} (Modelo: {s['modelo_usado']})")
    except Exception as e:
        print(f"⚠️ Nota sobre documentos: {e}")

    cur.close()
    conn.close()

    # 6. Probar Backend HTTP
    print("\n" + "-" * 70)
    print("🔌 PRUEBA DE CONEXIÓN CON BACKEND RUST (PUERTO 8090):")
    print("-" * 70)
    try:
        import httpx
        res = httpx.get("http://127.0.0.1:8090/health", timeout=3.0)
        print(f"  GET /health: Status {res.status_code} -> {res.text.strip()}")
    except Exception as e:
        print(f"  ⚠️ No se pudo contactar el backend en 127.0.0.1:8090: {e}")

    print("\n" + "=" * 70)
    print("🏁 DIAGNÓSTICO FINALIZADO")
    print("=" * 70)

if __name__ == "__main__":
    main()
