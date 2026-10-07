import os
import json
from typing import List, Dict, Any, Tuple
import anthropic
from config import settings
from db import get_db_connection

def get_configured_model() -> str:
    """Obtiene el modelo configurado dinámicamente desde PostgreSQL."""
    try:
        conn = get_db_connection()
        cur = conn.cursor()
        cur.execute("SELECT valor FROM configuraciones_sistema WHERE clave = 'CLAUDE_MODEL'")
        row = cur.fetchone()
        cur.close()
        conn.close()
        if row and row.get("valor"):
            return row["valor"].strip()
    except Exception as e:
        print(f"[Config] Error leyendo modelo de BD ({e}), usando default.")
    return settings.claude_model

def extract_text_from_response(response) -> str:
    """Extrae texto concatenado ignorando bloques de pensamiento (ThinkingBlock)."""
    text_blocks = []
    for block in response.content:
        if getattr(block, "type", "") == "text" and hasattr(block, "text"):
            text_blocks.append(block.text)
        elif hasattr(block, "text") and not hasattr(block, "thinking"):
            text_blocks.append(block.text)
    if not text_blocks and response.content:
        for block in response.content:
            if hasattr(block, "text"):
                text_blocks.append(str(block.text))
    return "\n".join(text_blocks).strip()

async def test_claude_model(model_name: str) -> str:
    """Prueba rápida de conectividad con un modelo específico de Claude."""
    if not settings.anthropic_api_key:
        raise ValueError("ANTHROPIC_API_KEY no configurada en workers/.env")
    client = anthropic.Anthropic(api_key=settings.anthropic_api_key)
    try:
        response = client.messages.create(
            model=model_name,
            max_tokens=60,
            messages=[{"role": "user", "content": "Responde con 5 palabras confirmando conexión y modelo."}]
        )
    except TypeError:
        response = client.messages.create(
            model=model_name,
            max_tokens=60,
            messages=[{"role": "user", "content": "Responde con 5 palabras confirmando conexión y modelo."}]
        )
    return extract_text_from_response(response)

DEFAULT_SYSTEM_PROMPT = """Eres el analista jefe de inteligencia estratégica de ExposureIQ, al servicio del Director de Operaciones de una de las aseguradoras más grandes de México.

Tu misión es leer los extractos temáticos del boletín diario de Coparmex (40+ páginas) y generar un "Briefing Ejecutivo Matutino" diseñado para ser leído directamente en WhatsApp en menos de 2 minutos.

Criterios de filtrado y priorización:
1. RELEVANTE PARA ASEGURADORA:
   - Impacto en Siniestralidad y Riesgos (robo a transporte, inseguridad en carreteras, desastres naturales, salud).
   - Marco Regulatorio y Jurídico (reformas legales, CNSF, SHCP, Condusef, reformas laborales/pensiones).
   - Variables Macroeconómicas (inflación médica/general, tasas de interés, tipo de cambio).
2. DESCARTE DE RUIDO:
   - Declaraciones puramente políticas, eventos de relaciones públicas o discursos genéricos sin impacto operativo.

Reglas estrictas de formato para WhatsApp:
- Usa negritas con un solo asterisco: *Título*
- Usa viñetas claras con guiones: - Punto clave
- No uses encabezados Markdown tipo # o ##
- Incluye 3 o 4 puntos clave máximo, cada uno con su impacto operativo para la aseguradora.
- Termina con un bloque breve de "Acción / Atención sugerida".
- Longitud total: Entre 180 y 300 palabras. Debe verse limpio y ejecutivo.
"""

def get_configured_system_prompt() -> str:
    """Obtiene el prompt de sistema configurado dinámicamente desde PostgreSQL."""
    try:
        conn = get_db_connection()
        cur = conn.cursor()
        cur.execute("SELECT valor FROM configuraciones_sistema WHERE clave = 'CLAUDE_SYSTEM_PROMPT'")
        row = cur.fetchone()
        cur.close()
        conn.close()
        if row and row.get("valor") and row["valor"].strip():
            return row["valor"].strip()
    except Exception as e:
        print(f"[Config] Error leyendo CLAUDE_SYSTEM_PROMPT de BD ({e}), usando default.")
    return DEFAULT_SYSTEM_PROMPT

def get_configured_max_tokens(default: int = 1000) -> int:
    """Obtiene el límite máximo de tokens configurado dinámicamente desde PostgreSQL."""
    try:
        conn = get_db_connection()
        cur = conn.cursor()
        cur.execute("SELECT valor FROM configuraciones_sistema WHERE clave = 'CLAUDE_MAX_TOKENS'")
        row = cur.fetchone()
        cur.close()
        conn.close()
        if row and row.get("valor") and str(row["valor"]).strip():
            val = int(str(row["valor"]).strip())
            if 200 <= val <= 8000:
                return val
    except Exception as e:
        print(f"[Config] Error leyendo CLAUDE_MAX_TOKENS de BD ({e}), usando default={default}.")
    return default

async def generate_executive_brief(sections: List[Dict[str, Any]], fecha_str: str) -> Tuple[str, List[str]]:
    """
    Toma las secciones del boletín, construye el resumen consolidado y llama a Claude API.
    Retorna el texto del mensaje y los temas principales detectados.
    """
    # Consolidar extractos más relevantes de cada sección
    compiled_extracts = []
    detected_topics = []

    for sec in sections:
        tema = sec.get("tema", "general")
        if tema not in detected_topics and tema != "editorial_general":
            detected_topics.append(tema)

        texto = extract_section_text(sec)
        # Recortar texto si es muy extenso
        preview = texto[:2500] if len(texto) > 2500 else texto
        compiled_extracts.append(f"--- SECCIÓN: {tema.upper()} (Págs {sec.get('pagina_inicio')}-{sec.get('pagina_fin')}) ---\n{preview}")

    context_prompt = (
        f"Fecha del Boletín: {fecha_str}\n\n"
        f"A continuación tienes los extractos temáticos del boletín Coparmex:\n\n"
        + "\n\n".join(compiled_extracts)
        + "\n\nGenera el briefing matutino para WhatsApp siguiendo estrictamente las instrucciones."
    )

    if not settings.anthropic_api_key:
        print("[LLM] ADVERTENCIA: ANTHROPIC_API_KEY no configurada. Generando brief sintético de respaldo.")
        fallback_brief = (
            f"📋 *BRIEFING EJECUTIVO COPARMEX — {fecha_str}*\n"
            f"_ExposureIQ · Inteligencia Operativa_\n\n"
            f"• *Regulación y Cumplimiento:* Seguimiento a lineamientos regulatorios e iniciativas en discusión.\n"
            f"• *Siniestralidad y Seguridad:* Monitoreo continuo de reportes de transporte de carga y carreteras prioritarias.\n"
            f"• *Entorno Económico:* Presión inflacionaria bajo vigilancia para ajuste de reservas técnicas.\n\n"
            f"📌 *Atención Operativa:* Revisar impacto en comités de siniestros y suscripción de flotillas.\n\n"
            f"_(Síntesis generada en modo de pruebas local)_"
        )
        return fallback_brief, detected_topics

    try:
        client = anthropic.Anthropic(api_key=settings.anthropic_api_key)

        configured_model = get_configured_model()
        models_to_try = [configured_model]
        fallback_candidates = [
            "claude-sonnet-4-5-20250929",
            "claude-haiku-4-5-20251001",
            "claude-sonnet-4-6",
            "claude-sonnet-5",
            "claude-opus-4-5-20251101",
            "claude-opus-4-6",
            "claude-opus-4-7",
            "claude-opus-4-8",
            "claude-opus-5",
            "claude-fable-5-1",
            "claude-fable-5",
        ]
        for m in fallback_candidates:
            if m not in models_to_try:
                models_to_try.append(m)

        active_system_prompt = get_configured_system_prompt()
        active_max_tokens = get_configured_max_tokens(default=1000)
        last_error = None
        for model_name in models_to_try:
            try:
                print(f"[LLM] Solicitando síntesis con modelo: {model_name} (límite: {active_max_tokens} tokens)...")
                try:
                    response = client.messages.create(
                        model=model_name,
                        max_tokens=active_max_tokens,
                        system=active_system_prompt,
                        messages=[
                            {"role": "user", "content": context_prompt}
                        ]
                    )
                except TypeError as te:
                    print(f"[LLM] Reintentando llamada compatible sin parámetro system ({te})...")
                    response = client.messages.create(
                        model=model_name,
                        max_tokens=active_max_tokens,
                        messages=[
                            {"role": "user", "content": f"{active_system_prompt}\n\n---\n\n{context_prompt}"}
                        ]
                    )

                brief_text = extract_text_from_response(response)
                print(f"[LLM] ✅ Síntesis ejecutiva generada exitosamente con '{model_name}'.")
                return brief_text, detected_topics

            except Exception as e:
                err_str = str(e).lower()
                if "404" in err_str or "not_found" in err_str:
                    print(f"[LLM] Modelo '{model_name}' no disponible en tu cuenta (404 Not Found). Probando siguiente...")
                    last_error = e
                    continue
                else:
                    # Si es otro error (ej: saldo/créditos o cuota), guardar y salir
                    last_error = e
                    break

        if last_error:
            raise last_error
    except Exception as e:
        print(f"[LLM] Error al invocar Claude API ({e}). Usando síntesis ejecutiva estructurada.")
        fallback_brief = (
            f"📋 *BRIEFING EJECUTIVO COPARMEX — {fecha_str}*\n"
            f"_ExposureIQ · Inteligencia Operativa_\n\n"
            f"• *Regulación y Cumplimiento:* Seguimiento a lineamientos regulatorios e iniciativas publicadas.\n"
            f"• *Siniestralidad y Seguridad:* Monitoreo continuo de reportes de transporte de carga e incidencias en carreteras prioritarias.\n"
            f"• *Entorno Económico:* Parámetros macroeconómicos e inflación bajo vigilancia para reservas técnicas.\n\n"
            f"📌 *Atención Operativa:* Revisar comités de siniestros y suscripción de flotillas.\n\n"
            f"_(Nota: Claude API retornó '{str(e)[:80]}...'. Se generó síntesis de respaldo editable)_"
        )
        return fallback_brief, detected_topics

def extract_section_text(sec: dict) -> str:
    """Extrae de manera segura el texto completo de una sección, soportando dict, str o JSON string."""
    contenido = sec.get("contenido")
    if not contenido:
        return ""
    if isinstance(contenido, str):
        try:
            parsed = json.loads(contenido)
            if isinstance(parsed, dict):
                return str(parsed.get("texto_completo", "") or parsed.get("text", "") or contenido)
            elif isinstance(parsed, list):
                return "\n".join(str(p) for p in parsed)
            return str(parsed)
        except Exception:
            return str(contenido)
    elif isinstance(contenido, dict):
        return str(contenido.get("texto_completo", "") or contenido.get("text", "") or "")
    return str(contenido)

async def generate_consolidated_daily_brief(documents: List[Dict[str, Any]], fecha_str: str) -> Tuple[str, List[str]]:
    """
    Toma los extractos de múltiples documentos de la fecha (Senado, Coparmex, notas),
    los compila en un contexto único y genera UNA SOLA síntesis ejecutiva consolidada para WhatsApp.
    """
    compiled_corpus = []
    detected_topics = []

    for doc in documents:
        doc_nombre = doc.get("nombre", "Documento sin título")
        doc_origen = doc.get("origen", "general")
        secciones = doc.get("secciones", [])
        
        doc_extracts = []
        for sec in secciones:
            tema = sec.get("tema", "general")
            if tema not in detected_topics and tema not in ["editorial_general", "general"]:
                detected_topics.append(tema)
            
            texto = extract_section_text(sec)
            preview = texto[:2200] if len(texto) > 2200 else texto
            if preview.strip():
                doc_extracts.append(f"[{tema.upper()}]\n{preview}")
        
        if doc_extracts:
            compiled_corpus.append(
                f"=====================================================\n"
                f"DOCUMENTO: {doc_nombre} (Origen: {doc_origen})\n"
                f"=====================================================\n"
                + "\n\n".join(doc_extracts)
            )

    if not compiled_corpus:
        fallback_brief = (
            f"📋 *SÍNTESIS DIARIA CONSOLIDADA — {fecha_str}*\n"
            f"_ExposureIQ · Inteligencia Operativa y Regulatoria_\n\n"
            f"• *Monitoreo Institucional:* Se integraron los documentos del día. No se detectaron alertas operativas críticas en el texto analizado.\n"
            f"• *Seguimiento Regulatorio:* Sin reformas de alto impacto inmediato registradas para esta jornada.\n\n"
            f"📌 *Atención Operativa:* Mantener monitoreo estándar en comités de suscripción."
        )
        return fallback_brief, detected_topics

    context_prompt = (
        f"Fecha de Análisis: {fecha_str}\n"
        f"Se han consolidado {len(documents)} documentos para esta jornada (Senado de la República, boletines sectoriales y notas ejecutivas).\n\n"
        f"A continuación tienes el universo de textos analizados:\n\n"
        + "\n\n".join(compiled_corpus)
        + "\n\nInstrucción: Genera un ÚNICO Briefing Ejecutivo Consolidado del Día para WhatsApp, integrando los puntos más críticos de todos los documentos, eliminando ruido duplicado y aplicando estrictamente las reglas de formato."
    )

    if not settings.anthropic_api_key:
        print("[LLM] ADVERTENCIA: ANTHROPIC_API_KEY no configurada. Generando síntesis consolidada de respaldo.")
        fallback_brief = (
            f"📋 *SÍNTESIS DIARIA CONSOLIDADA — {fecha_str}*\n"
            f"_ExposureIQ · Inteligencia Operativa y Regulatoria_\n\n"
            f"• *Marco Regulatorio (Senado / Diario Oficial):* Seguimiento a iniciativas de ley, marcos jurídicos y resoluciones de comisiones legislativas.\n"
            f"• *Siniestralidad y Seguridad Nacional:* Panorama nacional y reportes de seguridad en carreteras y transporte de carga.\n"
            f"• *Opinión y Análisis Político:* Principales posturas de senadores y columnas editoriales relevantes para la industria aseguradora.\n\n"
            f"📌 *Atención Operativa Sugerida:* Revisar reservas técnicas y seguimiento a comités de riesgos.\n\n"
            f"_(Síntesis consolidada generada con {len(documents)} documentos en modo de pruebas local)_"
        )
        return fallback_brief, detected_topics

    try:
        client = anthropic.Anthropic(api_key=settings.anthropic_api_key)
        configured_model = get_configured_model()
        models_to_try = [
            configured_model,
            "claude-sonnet-4-5-20250929",
            "claude-haiku-4-5-20251001",
            "claude-3-5-sonnet-20241022",
            "claude-3-haiku-20240307"
        ]
        seen = set()
        dedup_models = []
        for m in models_to_try:
            if m and m not in seen:
                seen.add(m)
                dedup_models.append(m)

        active_system_prompt = get_configured_system_prompt()
        active_max_tokens = get_configured_max_tokens(default=1500)
        last_error = None

        # Truncar context_prompt de forma segura si excede 100,000 caracteres (~25k tokens)
        max_prompt_chars = 100000
        safe_prompt = context_prompt if len(context_prompt) <= max_prompt_chars else context_prompt[:max_prompt_chars] + "\n\n[...Extractos adicionales condensados por volumen...]"

        for model_name in dedup_models:
            try:
                print(f"[LLM] Generando síntesis consolidada con: {model_name} (límite: {active_max_tokens} tokens)...")
                try:
                    response = client.messages.create(
                        model=model_name,
                        max_tokens=active_max_tokens,
                        system=active_system_prompt,
                        messages=[{"role": "user", "content": safe_prompt}]
                    )
                except TypeError:
                    # Compatibilidad en caso de SDK sin parámetro system directo
                    response = client.messages.create(
                        model=model_name,
                        max_tokens=active_max_tokens,
                        messages=[{"role": "user", "content": f"{active_system_prompt}\n\n---\n\n{safe_prompt}"}]
                    )

                brief_text = extract_text_from_response(response)
                print(f"[LLM] ✅ Síntesis consolidada generada exitosamente con '{model_name}'.")
                return brief_text, detected_topics
            except Exception as e:
                err_str = str(e).lower()
                print(f"[LLM] Advertencia: Modelo '{model_name}' falló ({e}). Probando alternativa...")
                last_error = e
                continue

        if last_error:
            raise last_error
    except Exception as e:
        print(f"[LLM] Error en Claude API para síntesis consolidada ({e}). Retornando síntesis estructurada.")
        fallback_brief = (
            f"📋 *SÍNTESIS DIARIA CONSOLIDADA — {fecha_str}*\n"
            f"_ExposureIQ · Inteligencia Operativa y Regulatoria_\n\n"
            f"• *Marco Regulatorio:* Iniciativas prioritarias y actividad parlamentaria del día.\n"
            f"• *Siniestralidad y Transporte:* Seguimiento a corredores logísticos y notas de seguridad patrimonial.\n"
            f"• *Impacto Financiero:* Variables del entorno económico bajo vigilancia para el sector asegurador.\n\n"
            f"📌 *Atención Operativa:* Revisar comisiones de riesgos y análisis de siniestros.\n\n"
            f"_(Nota: Claude API retornó '{str(e)[:70]}...'. Se generó síntesis de respaldo editable)_"
        )
        return fallback_brief, detected_topics

