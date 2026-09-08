import re
from typing import List, Dict, Any

# Palabras clave para clasificación temática orientada a seguros
TOPIC_KEYWORDS = {
    "regulación_normativa": [
        "reforma", "decreto", "ley", "diario oficial", "dof", "cnsf", "shcp", 
        "condusef", "regulación", "circular", "iniciativa de ley", "jurisprudencia"
    ],
    "seguridad_publica": [
        "seguridad", "robo", "delincuencia", "transporte de carga", "violencia", 
        "siniestros", "homicidio", "extorsión", "carreteras", "guardia nacional"
    ],
    "economia_finanzas": [
        "inflación", "tasas de interés", "banxico", "tipo de cambio", "pib", 
        "inversión", "crecimiento", "salarios", "mercado bursátil", "política monetaria"
    ],
    "laboral_pensiones": [
        "reforma laboral", "jornada laboral", "pensiones", "imss", "infonavit", 
        "salario mínimo", "sindicatos", "contrato colectivo", "uma"
    ],
    "salud_seguros": [
        "salud", "gastos médicos", "hospitales", "medicamentos", "epidemias", 
        "cofepris", "servicios médicos", "cobertura sanitaria"
    ],
    "movilidad_transporte": [
        "movilidad", "transporte", "parque vehicular", "aduanas", "logística", 
        "puertos", "infraestructura vial", "autotransporte"
    ]
}

def segment_bulletin(pages: List[Dict[str, Any]]) -> List[Dict[str, Any]]:
    """
    Agrupa las páginas del boletín por afinidad temática y jerarquía de títulos
    para no enviar las 40+ páginas en un solo bloque al LLM.
    """
    sections = []
    current_topic = "editorial_general"
    current_pages = []
    current_text = []
    start_page = 1
    section_index = 1

    for page_data in pages:
        page_num = page_data.get("page", 1)
        text = page_data.get("text", "")
        text_lower = text.lower()

        # Detección de cambio de tema preponderante en la página
        detected_topic = None
        highest_score = 0

        for topic, keywords in TOPIC_KEYWORDS.items():
            score = sum(1 for kw in keywords if kw in text_lower)
            if score > highest_score and score >= 2:
                highest_score = score
                detected_topic = topic

        # Si hay cambio de sección relevante y ya acumulamos texto previo
        if detected_topic and detected_topic != current_topic and current_text:
            sections.append({
                "orden": section_index,
                "tema": current_topic,
                "pagina_inicio": start_page,
                "pagina_fin": page_num - 1,
                "contenido": {
                    "tema_detectado": current_topic,
                    "paginas_abarcadas": current_pages,
                    "texto_completo": "\n\n".join(current_text)
                }
            })
            section_index += 1
            current_topic = detected_topic
            current_pages = [page_num]
            current_text = [text]
            start_page = page_num
        else:
            if not current_pages:
                start_page = page_num
                if detected_topic:
                    current_topic = detected_topic
            current_pages.append(page_num)
            current_text.append(text)

    # Última sección remanente
    if current_text:
        sections.append({
            "orden": section_index,
            "tema": current_topic,
            "pagina_inicio": start_page,
            "pagina_fin": pages[-1].get("page", start_page) if pages else start_page,
            "contenido": {
                "tema_detectado": current_topic,
                "paginas_abarcadas": current_pages,
                "texto_completo": "\n\n".join(current_text)
            }
        })

    return sections
