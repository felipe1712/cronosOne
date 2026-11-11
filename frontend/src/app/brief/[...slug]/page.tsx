"use client";

import React, { useEffect, useState } from "react";
import { useParams, useSearchParams, useRouter } from "next/navigation";
import Link from "next/link";
import {
  Box,
  Card,
  CardContent,
  Typography,
  Chip,
  Button,
  IconButton,
  Divider,
  CircularProgress,
  Alert,
  Collapse,
  Paper,
  Tooltip,
} from "@mui/material";
import { ApiService } from "@/lib/api";

interface BriefItem {
  id: string;
  edition_date: string;
  section_key: string;
  section_title: string;
  section_color: string;
  section_icon: string;
  title: string;
  summary: string;
  full_content: string;
  public_slug: string;
  published_at?: string;
}

interface WeekData {
  start_date: string;
  end_date: string;
  label: string;
  prev_week: string;
  next_week: string;
  is_current: boolean;
  entries: BriefItem[];
}

export default function UnifiedBriefSlugPage() {
  const rawParams = useParams();
  const searchParams = useSearchParams();
  const router = useRouter();

  // Parse dynamic slug segments
  const rawSlug = rawParams?.slug;
  const slugArray: string[] = Array.isArray(rawSlug)
    ? rawSlug
    : typeof rawSlug === "string"
    ? [rawSlug]
    : ["politica"];

  const isDateRegex = /^\d{4}-\d{2}-\d{2}$/;

  let mode: "feed" | "historical" = "feed";
  let targetSection = "politica";
  let targetFecha: string | undefined = searchParams.get("fecha") || undefined;

  if (slugArray.length >= 2) {
    if (isDateRegex.test(slugArray[0])) {
      // Formato /brief/2026-10-12/politica
      targetFecha = slugArray[0];
      targetSection = slugArray[1];
      mode = "historical";
    } else if (isDateRegex.test(slugArray[1])) {
      // Formato alternativo /brief/politica/2026-10-12
      targetSection = slugArray[0];
      targetFecha = slugArray[1];
      mode = "historical";
    } else {
      targetSection = slugArray[0];
    }
  } else if (slugArray.length === 1) {
    targetSection = slugArray[0];
    mode = "feed";
  }

  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);

  // Estados para modo Feed Semanal
  const [latestItem, setLatestItem] = useState<BriefItem | null>(null);
  const [weekData, setWeekData] = useState<WeekData | null>(null);
  const [expandedIds, setExpandedIds] = useState<Record<string, boolean>>({});
  const [copiedId, setCopiedId] = useState<string | null>(null);

  // Estados para modo Histórico único
  const [historicalItem, setHistoricalItem] = useState<BriefItem | null>(null);
  const [copiedHistorical, setCopiedHistorical] = useState<boolean>(false);

  useEffect(() => {
    setLoading(true);
    setError(null);

    if (mode === "historical" && targetFecha) {
      ApiService.getPublicBriefByDate(targetFecha, targetSection)
        .then((res) => {
          if (res && res.item) {
            setHistoricalItem(res.item);
          } else {
            setError(`No se encontró la síntesis histórica para la fecha ${targetFecha}.`);
          }
        })
        .catch((err) => {
          setError(err.message || "Error consultando edición histórica.");
        })
        .finally(() => setLoading(false));
    } else {
      ApiService.getPublicBrief(targetSection, { fecha: targetFecha })
        .then((res) => {
          if (res && res.item) {
            setLatestItem(res.item);
            setWeekData(res.week);
            if (res.week?.entries?.length > 0) {
              setExpandedIds({ [res.week.entries[0].id]: true });
            }
          } else {
            setError("No se encontró información publicada para esta sección.");
          }
        })
        .catch((err) => {
          setError(err.message || "Error al cargar el feed semanal.");
        })
        .finally(() => setLoading(false));
    }
  }, [mode, targetSection, targetFecha]);

  const toggleExpand = (id: string) => {
    setExpandedIds((prev) => ({ ...prev, [id]: !prev[id] }));
  };

  const copyPermalink = (dateStr: string, id: string) => {
    const url = `${window.location.origin}/brief/${dateStr}/${targetSection}`;
    navigator.clipboard.writeText(url);
    setCopiedId(id);
    setTimeout(() => setCopiedId(null), 2500);
  };

  const copyHistoricalPermalink = () => {
    navigator.clipboard.writeText(window.location.href);
    setCopiedHistorical(true);
    setTimeout(() => setCopiedHistorical(false), 2500);
  };

  const handleNavigateWeek = (dateStr: string) => {
    router.push(`/brief/${targetSection}?fecha=${dateStr}`);
  };

  const formatDateDisplay = (dateStr: string) => {
    try {
      const [y, m, d] = dateStr.split("-").map(Number);
      const dt = new Date(y, m - 1, d);
      return new Intl.DateTimeFormat("es-MX", {
        weekday: "long",
        day: "numeric",
        month: "long",
        year: "numeric",
      }).format(dt);
    } catch {
      return dateStr;
    }
  };

  if (loading) {
    return (
      <Box sx={{ display: "flex", flexDirection: "column", alignItems: "center", justifyContent: "center", py: 12 }}>
        <CircularProgress size={45} thickness={4} sx={{ color: "#0284c7" }} />
        <Typography variant="body2" sx={{ mt: 2, color: "#64748b", fontWeight: 500 }}>
          Cargando síntesis ejecutiva...
        </Typography>
      </Box>
    );
  }

  if (error) {
    return (
      <Box sx={{ maxWidth: 700, mx: "auto", py: 6 }}>
        <Alert severity="info" sx={{ borderRadius: "12px", border: "1px solid #bae6fd" }}>
          {error}
        </Alert>
        <Box sx={{ mt: 3, textAlign: "center" }}>
          <Button variant="outlined" onClick={() => router.push("/brief/politica")} sx={{ borderRadius: "8px" }}>
            Ver Canal de Política y Gobierno
          </Button>
        </Box>
      </Box>
    );
  }

  // =========================================================================
  // VISTA 1: EDICIÓN HISTÓRICA INDIVIDUAL (/brief/YYYY-MM-DD/seccion)
  // =========================================================================
  if (mode === "historical" && historicalItem) {
    return (
      <Box sx={{ maxWidth: 840, mx: "auto", py: 2 }}>
        <Box sx={{ mb: 3 }}>
          <Button
            variant="text"
            startIcon={<span className="material-symbols-outlined">arrow_back</span>}
            onClick={() => router.push(`/brief/${historicalItem.section_key}`)}
            sx={{ color: "#475569", fontWeight: 600, textTransform: "none", mb: 2 }}
          >
            Volver al feed semanal de {historicalItem.section_title}
          </Button>

          <Box sx={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start", flexWrap: "wrap", gap: 2 }}>
            <Box>
              <Box sx={{ display: "flex", alignItems: "center", gap: 1.5, mb: 1 }}>
                <Chip
                  label={historicalItem.section_title}
                  sx={{
                    backgroundColor: historicalItem.section_color || "#0284c7",
                    color: "#ffffff",
                    fontWeight: 700,
                  }}
                />
                <Chip label="EDICIÓN HISTÓRICA DE ARCHIVO" size="small" variant="outlined" sx={{ color: "#64748b" }} />
              </Box>
              <Typography variant="h4" sx={{ fontWeight: 900, color: "#0f172a", letterSpacing: -0.5 }}>
                {historicalItem.title}
              </Typography>
              <Typography variant="subtitle1" sx={{ color: "#475569", textTransform: "capitalize", mt: 0.5 }}>
                📅 {formatDateDisplay(historicalItem.edition_date)}
              </Typography>
            </Box>

            <Tooltip title={copiedHistorical ? "¡Enlace permanente copiado!" : "Copiar enlace permanente"}>
              <Button
                variant="outlined"
                size="small"
                startIcon={<span className="material-symbols-outlined">{copiedHistorical ? "check" : "share"}</span>}
                onClick={copyHistoricalPermalink}
                sx={{ borderRadius: "8px", textTransform: "none", borderColor: "#cbd5e1", color: "#334155" }}
              >
                {copiedHistorical ? "Copiado" : "Compartir URL"}
              </Button>
            </Tooltip>
          </Box>
        </Box>

        <Card
          elevation={0}
          sx={{
            borderRadius: "16px",
            border: `2px solid ${historicalItem.section_color || "#0284c7"}`,
            backgroundColor: "#ffffff",
            overflow: "hidden",
            boxShadow: "0 10px 30px rgba(15, 23, 42, 0.05)",
          }}
        >
          <CardContent sx={{ p: { xs: 3, md: 4 } }}>
            {historicalItem.summary && (
              <Box
                sx={{
                  backgroundColor: "#f8fafc",
                  borderLeft: `4px solid ${historicalItem.section_color || "#0284c7"}`,
                  p: 2.5,
                  borderRadius: "0 10px 10px 0",
                  mb: 3,
                }}
              >
                <Typography variant="caption" sx={{ fontWeight: 800, color: historicalItem.section_color || "#0284c7", display: "block", mb: 0.5 }}>
                  RESUMEN EJECUTIVO
                </Typography>
                <Typography variant="body1" sx={{ color: "#334155", lineHeight: 1.6, fontWeight: 500 }}>
                  {historicalItem.summary}
                </Typography>
              </Box>
            )}

            <Divider sx={{ my: 3 }} />

            <Box
              sx={{
                typography: "body1",
                color: "#1e293b",
                lineHeight: 1.9,
                "& p": { mb: 2.5 },
                "& ul": { pl: 3, mb: 2.5 },
                "& li": { mb: 1 },
                "& strong": { color: "#0f172a" },
                whiteSpace: "pre-line",
              }}
            >
              {historicalItem.full_content}
            </Box>
          </CardContent>
        </Card>
      </Box>
    );
  }

  // =========================================================================
  // VISTA 2: FEED SEMANAL POR CANAL (/brief/politica)
  // =========================================================================
  if (!latestItem) {
    return null;
  }

  const entries = weekData?.entries || [latestItem];

  return (
    <Box sx={{ maxWidth: 880, mx: "auto" }}>
      {/* Barra de Navegación Semanal */}
      {weekData && (
        <Paper
          elevation={0}
          sx={{
            p: 2,
            mb: 4,
            borderRadius: "14px",
            backgroundColor: "#ffffff",
            border: "1px solid #e2e8f0",
            display: "flex",
            flexWrap: "wrap",
            alignItems: "center",
            justifyContent: "space-between",
            gap: 2,
          }}
        >
          <Button
            variant="text"
            size="small"
            startIcon={<span className="material-symbols-outlined">arrow_back</span>}
            onClick={() => handleNavigateWeek(weekData.prev_week)}
            sx={{ color: "#475569", fontWeight: 600, textTransform: "none" }}
          >
            Semana anterior
          </Button>

          <Box sx={{ textAlign: "center" }}>
            <Box sx={{ display: "flex", alignItems: "center", justifyContent: "center", gap: 1 }}>
              <span className="material-symbols-outlined" style={{ color: latestItem.section_color || "#0284c7", fontSize: "20px" }}>
                calendar_today
              </span>
              <Typography variant="subtitle1" sx={{ fontWeight: 800, color: "#0f172a" }}>
                {weekData.label}
              </Typography>
            </Box>
            <Typography variant="caption" sx={{ color: "#64748b" }}>
              Mostrando {entries.length} {entries.length === 1 ? "síntesis" : "síntesis"} ordenadas de más reciente a más antigua
            </Typography>
          </Box>

          <Box sx={{ display: "flex", alignItems: "center", gap: 1 }}>
            {!weekData.is_current && (
              <Button
                variant="outlined"
                size="small"
                onClick={() => router.push(`/brief/${targetSection}`)}
                sx={{
                  borderRadius: "8px",
                  fontSize: "0.75rem",
                  borderColor: "#cbd5e1",
                  color: "#0f172a",
                  textTransform: "none",
                }}
              >
                Semana en curso
              </Button>
            )}
            <Button
              variant="text"
              size="small"
              endIcon={<span className="material-symbols-outlined">arrow_forward</span>}
              disabled={weekData.is_current}
              onClick={() => handleNavigateWeek(weekData.next_week)}
              sx={{ color: "#475569", fontWeight: 600, textTransform: "none" }}
            >
              Semana siguiente
            </Button>
          </Box>
        </Paper>
      )}

      {/* Encabezado del Canal */}
      <Box sx={{ mb: 4, display: "flex", alignItems: "center", justifyContent: "space-between", flexWrap: "wrap", gap: 2 }}>
        <Box>
          <Box sx={{ display: "flex", alignItems: "center", gap: 1.5, mb: 0.5 }}>
            <Chip
              label={latestItem.section_title}
              sx={{
                backgroundColor: latestItem.section_color || "#0284c7",
                color: "#ffffff",
                fontWeight: 700,
                fontSize: "0.85rem",
              }}
            />
            {weekData?.is_current && (
              <Chip
                label="Edición más reciente"
                size="small"
                variant="outlined"
                sx={{ borderColor: "#22c55e", color: "#15803d", fontWeight: 600 }}
              />
            )}
          </Box>
          <Typography variant="h4" sx={{ fontWeight: 900, color: "#0f172a", letterSpacing: -0.5 }}>
            {latestItem.section_title}
          </Typography>
        </Box>
      </Box>

      {/* Feed de Ediciones de la Semana (Más Reciente Primero) */}
      <Box sx={{ display: "flex", flexDirection: "column", gap: 3.5 }}>
        {entries.map((item, index) => {
          const isExpanded = expandedIds[item.id] ?? (index === 0);
          const isCopied = copiedId === item.id;

          return (
            <Card
              key={item.id}
              elevation={0}
              sx={{
                borderRadius: "16px",
                border: index === 0 ? `2px solid ${item.section_color || "#0284c7"}` : "1px solid #e2e8f0",
                backgroundColor: "#ffffff",
                overflow: "hidden",
                transition: "all 0.2s ease",
                "&:hover": {
                  boxShadow: "0 8px 24px rgba(15, 23, 42, 0.05)",
                },
              }}
            >
              <CardContent sx={{ p: { xs: 2.5, md: 3.5 } }}>
                {/* Meta Bar */}
                <Box sx={{ display: "flex", justifyContent: "space-between", alignItems: "center", mb: 2 }}>
                  <Box sx={{ display: "flex", alignItems: "center", gap: 1 }}>
                    <span
                      className="material-symbols-outlined"
                      style={{ fontSize: "18px", color: item.section_color || "#0284c7" }}
                    >
                      event
                    </span>
                    <Typography variant="subtitle2" sx={{ fontWeight: 700, color: "#334155", textTransform: "capitalize" }}>
                      {formatDateDisplay(item.edition_date)}
                    </Typography>
                    {index === 0 && weekData?.is_current && (
                      <Chip label="ÚLTIMA" size="small" sx={{ height: 20, fontSize: "0.65rem", fontWeight: 800, backgroundColor: "#dbeafe", color: "#1d4ed8" }} />
                    )}
                  </Box>

                  <Tooltip title={isCopied ? "¡Enlace copiado!" : "Copiar enlace permanente"}>
                    <IconButton size="small" onClick={() => copyPermalink(item.edition_date, item.id)}>
                      <span className="material-symbols-outlined" style={{ fontSize: "18px", color: isCopied ? "#16a34a" : "#94a3b8" }}>
                        {isCopied ? "check" : "share"}
                      </span>
                    </IconButton>
                  </Tooltip>
                </Box>

                {/* Entry Title */}
                <Typography variant="h5" sx={{ fontWeight: 800, color: "#0f172a", mb: 2, lineHeight: 1.3 }}>
                  {item.title}
                </Typography>

                {/* Key Takeaways Summary Box */}
                {item.summary && (
                  <Box
                    sx={{
                      backgroundColor: "#f8fafc",
                      borderLeft: `4px solid ${item.section_color || "#0284c7"}`,
                      p: 2,
                      borderRadius: "0 8px 8px 0",
                      mb: 2.5,
                    }}
                  >
                    <Typography variant="caption" sx={{ fontWeight: 700, color: item.section_color || "#0284c7", display: "block", mb: 0.5, letterSpacing: 0.5 }}>
                      RESUMEN EJECUTIVO
                    </Typography>
                    <Typography variant="body2" sx={{ color: "#334155", lineHeight: 1.6 }}>
                      {item.summary}
                    </Typography>
                  </Box>
                )}

                {/* Collapsible Full Content */}
                <Collapse in={isExpanded} timeout="auto" unmountOnExit>
                  <Divider sx={{ my: 2.5 }} />
                  <Box
                    sx={{
                      typography: "body1",
                      color: "#1e293b",
                      lineHeight: 1.8,
                      "& p": { mb: 2 },
                      "& ul": { pl: 3, mb: 2 },
                      "& li": { mb: 1 },
                      "& strong": { color: "#0f172a" },
                      whiteSpace: "pre-line",
                    }}
                  >
                    {item.full_content}
                  </Box>
                </Collapse>

                {/* Toggle Button */}
                <Box sx={{ mt: 2, display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                  <Button
                    variant="text"
                    onClick={() => toggleExpand(item.id)}
                    endIcon={
                      <span className="material-symbols-outlined">
                        {isExpanded ? "expand_less" : "expand_more"}
                      </span>
                    }
                    sx={{
                      color: item.section_color || "#0284c7",
                      fontWeight: 700,
                      textTransform: "none",
                      px: 0,
                    }}
                  >
                    {isExpanded ? "Ocultar desglose completo" : "Leer análisis completo del día"}
                  </Button>

                  <Link
                    href={`/brief/${item.edition_date}/${item.section_key}`}
                    style={{
                      fontSize: "0.8rem",
                      color: "#64748b",
                      textDecoration: "none",
                      display: "flex",
                      alignItems: "center",
                      gap: 4,
                    }}
                  >
                    URL Histórica
                    <span className="material-symbols-outlined" style={{ fontSize: "14px" }}>
                      open_in_new
                    </span>
                  </Link>
                </Box>
              </CardContent>
            </Card>
          );
        })}
      </Box>
    </Box>
  );
}
