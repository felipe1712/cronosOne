import os
from typing import List, Dict, Any, Tuple
import anthropic
from config import settings

SYSTEM_PROMPT = """Eres el analista jefe de inteligencia estratégica de ExposureIQ, al servicio del Director de Operaciones de una de las aseguradoras más grandes de México.

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

        texto = sec.get("contenido", {}).get("texto_completo", "")
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

        try:
            response = client.messages.create(
                model=settings.claude_model,
                max_tokens=1000,
                system=SYSTEM_PROMPT,
                messages=[
                    {"role": "user", "content": context_prompt}
                ]
            )
        except TypeError as te:
            print(f"[LLM] Reintentando llamada compatible a Claude sin parámetro system ({te})...")
            response = client.messages.create(
                model=settings.claude_model,
                max_tokens=1000,
                messages=[
                    {"role": "user", "content": f"{SYSTEM_PROMPT}\n\n---\n\n{context_prompt}"}
                ]
            )

        brief_text = response.content[0].text
        return brief_text, detected_topics
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
