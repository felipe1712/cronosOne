"use client";

import React, { useEffect, useState } from "react";
import { useParams, useRouter } from "next/navigation";
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
  Tooltip,
  Paper,
} from "@mui/material";
import { ApiService } from "@/lib/api";

export default function HistoricalBriefPage() {
  const params = useParams();
  const router = useRouter();

  const fecha = params?.fecha as string;
  const seccion = params?.seccion as string;

  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);
  const [item, setItem] = useState<any | null>(null);
  const [copied, setCopied] = useState<boolean>(false);

  useEffect(() => {
    if (!fecha || !seccion) return;
    setLoading(true);
    setError(null);
    ApiService.getPublicBriefByDate(fecha, seccion)
      .then((res) => {
        if (res && res.item) {
          setItem(res.item);
        } else {
          setError("No se encontró la edición histórica para la fecha solicitada.");
        }
      })
      .catch((err) => {
        setError(err.message || "Error consultando edición histórica.");
      })
      .finally(() => setLoading(false));
  }, [fecha, seccion]);

  const copyUrl = () => {
    navigator.clipboard.writeText(window.location.href);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
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
      <Box sx={{ display: "flex", flexDirection: "column", alignItems: "center", justifyContent: "center", py: 10 }}>
        <CircularProgress size={45} sx={{ color: "#0284c7" }} />
        <Typography variant="body2" sx={{ mt: 2, color: "#64748b" }}>
          Recuperando edición archivada del {fecha}...
        </Typography>
      </Box>
    );
  }

  if (error || !item) {
    return (
      <Box sx={{ maxWidth: 700, mx: "auto", py: 6 }}>
        <Alert severity="warning" sx={{ borderRadius: "12px" }}>
          {error || "Edición no localizada en el histórico."}
        </Alert>
        <Box sx={{ mt: 3, textAlign: "center" }}>
          <Button variant="contained" onClick={() => router.push(`/brief/${seccion}`)}>
            Ir a la edición en vivo de {seccion}
          </Button>
        </Box>
      </Box>
    );
  }

  return (
    <Box sx={{ maxWidth: 880, mx: "auto" }}>
      {/* Historical Navigation Bar */}
      <Paper
        elevation={0}
        sx={{
          p: 2,
          mb: 4,
          borderRadius: "14px",
          backgroundColor: "#fffbeb",
          border: "1px solid #fde68a",
          display: "flex",
          flexWrap: "wrap",
          alignItems: "center",
          justifyContent: "space-between",
          gap: 2,
        }}
      >
        <Box sx={{ display: "flex", alignItems: "center", gap: 1.5 }}>
          <span className="material-symbols-outlined" style={{ color: "#d97706", fontSize: "22px" }}>
            history
          </span>
          <Box>
            <Typography variant="subtitle2" sx={{ fontWeight: 800, color: "#92400e" }}>
              Edición Histórica Archivada
            </Typography>
            <Typography variant="caption" sx={{ color: "#b45309" }}>
              Estás visualizando el registro inmutable del {formatDateDisplay(fecha)}
            </Typography>
          </Box>
        </Box>

        <Button
          variant="contained"
          size="small"
          onClick={() => router.push(`/brief/${seccion}`)}
          sx={{
            backgroundColor: "#0284c7",
            color: "#ffffff",
            fontWeight: 700,
            borderRadius: "8px",
            textTransform: "none",
            "&:hover": { backgroundColor: "#0369a1" },
          }}
        >
          Ver edición más reciente
        </Button>
      </Paper>

      {/* Main Historical Card */}
      <Card
        elevation={0}
        sx={{
          borderRadius: "16px",
          border: "1px solid #e2e8f0",
          backgroundColor: "#ffffff",
          p: { xs: 2.5, md: 4 },
        }}
      >
        <CardContent sx={{ p: 0 }}>
          <Box sx={{ display: "flex", justifyContent: "space-between", alignItems: "center", mb: 2 }}>
            <Box sx={{ display: "flex", alignItems: "center", gap: 1 }}>
              <Chip
                label={item.section_title}
                sx={{
                  backgroundColor: item.section_color || "#0284c7",
                  color: "#ffffff",
                  fontWeight: 700,
                }}
              />
              <Typography variant="subtitle2" sx={{ fontWeight: 700, color: "#64748b", textTransform: "capitalize" }}>
                {formatDateDisplay(item.edition_date)}
              </Typography>
            </Box>

            <Tooltip title={copied ? "¡Enlace copiado!" : "Copiar URL histórica"}>
              <IconButton size="small" onClick={copyUrl}>
                <span className="material-symbols-outlined" style={{ fontSize: "18px", color: copied ? "#16a34a" : "#94a3b8" }}>
                  {copied ? "check" : "share"}
                </span>
              </IconButton>
            </Tooltip>
          </Box>

          <Typography variant="h4" sx={{ fontWeight: 900, color: "#0f172a", mb: 2.5, lineHeight: 1.25 }}>
            {item.title}
          </Typography>

          {item.summary && (
            <Box
              sx={{
                backgroundColor: "#f8fafc",
                borderLeft: `4px solid ${item.section_color || "#0284c7"}`,
                p: 2.5,
                borderRadius: "0 10px 10px 0",
                mb: 3.5,
              }}
            >
              <Typography variant="caption" sx={{ fontWeight: 700, color: item.section_color || "#0284c7", display: "block", mb: 0.5 }}>
                RESUMEN EJECUTIVO
              </Typography>
              <Typography variant="body1" sx={{ color: "#334155", lineHeight: 1.6 }}>
                {item.summary}
              </Typography>
            </Box>
          )}

          <Divider sx={{ my: 3 }} />

          <Box
            sx={{
              typography: "body1",
              color: "#1e293b",
              lineHeight: 1.85,
              whiteSpace: "pre-line",
            }}
          >
            {item.full_content}
          </Box>
        </CardContent>
      </Card>
    </Box>
  );
}
