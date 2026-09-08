"use client";

import React, { useState, useEffect } from "react";
import {
  Box,
  Card,
  CardContent,
  Typography,
  Grid,
  Button,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  Paper,
  Chip,
  CircularProgress,
  Alert,
} from "@mui/material";
import WhatsAppIcon from "@mui/icons-material/WhatsApp";
import QrCode2Icon from "@mui/icons-material/QrCode2";
import RefreshIcon from "@mui/icons-material/Refresh";
import CheckCircleOutlineIcon from "@mui/icons-material/CheckCircleOutline";
import ErrorOutlineIcon from "@mui/icons-material/ErrorOutline";
import HourglassEmptyIcon from "@mui/icons-material/HourglassEmpty";
import { ApiService } from "@/lib/api";

export default function WahaPage() {
  const [wahaStatus, setWahaStatus] = useState<any>(null);
  const [wahaQr, setWahaQr] = useState<string | null>(null);
  const [mensajes, setMensajes] = useState<any[]>([]);
  const [loading, setLoading] = useState<boolean>(true);
  const [restarting, setRestarting] = useState<boolean>(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  const loadWahaData = async () => {
    try {
      setLoading(true);
      setError(null);
      const [statusData, qrData, historialData] = await Promise.all([
        ApiService.getWahaStatus().catch(() => ({ status: "DISCONNECTED" })),
        ApiService.getWahaQr().catch(() => ({ qr: null })),
        ApiService.getHistorialMensajes().catch(() => []),
      ]);

      setWahaStatus(statusData);
      setWahaQr(qrData.qr || null);
      setMensajes(historialData);
    } catch (err: any) {
      setError(err.message || "Error al consultar estado de WAHA");
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadWahaData();
    const interval = setInterval(loadWahaData, 15000); // Refresco cada 15s
    return () => clearInterval(interval);
  }, []);

  const handleRestart = async () => {
    try {
      setRestarting(true);
      setError(null);
      await ApiService.restartWaha();
      setSuccess("Comando de reinicio enviado a WAHA. Revisa el estado en unos segundos.");
      setTimeout(loadWahaData, 3000);
    } catch (err: any) {
      setError(err.message || "Error al reiniciar sesión de WAHA");
    } finally {
      setRestarting(false);
    }
  };

  const getEstadoMensajeChip = (estado: string) => {
    switch (estado) {
      case "confirmado":
        return <Chip label="Entregado (WAHA)" color="success" size="small" icon={<CheckCircleOutlineIcon />} />;
      case "entregado_a_n8n":
        return <Chip label="Enviando con n8n..." color="info" size="small" icon={<HourglassEmptyIcon />} />;
      case "pendiente":
        return <Chip label="En cola de salida" color="warning" size="small" />;
      default:
        return <Chip label="Error de envío" color="error" size="small" icon={<ErrorOutlineIcon />} />;
    }
  };

  const isConnected =
    wahaStatus?.status === "CONNECTED" ||
    wahaStatus?.status === "WORKING" ||
    wahaStatus?.status === "STARTING";

  return (
    <Box sx={{ p: 3 }}>
      {/* Encabezado */}
      <Box sx={{ display: "flex", justifyContent: "space-between", alignItems: "center", mb: 3 }}>
        <Box>
          <Typography variant="h5" sx={{ fontWeight: 700, color: "#1e293b" }}>
            Canal WhatsApp & WAHA — Monitoreo de Entrega
          </Typography>
          <Typography variant="body2" sx={{ color: "#64748b" }}>
            Estado de sesión del bot de WhatsApp, vinculación por código QR y registro de despachos ejecutivos.
          </Typography>
        </Box>
        <Button variant="outlined" startIcon={<RefreshIcon />} onClick={loadWahaData} disabled={loading}>
          Actualizar
        </Button>
      </Box>

      {error && (
        <Alert severity="error" sx={{ mb: 3 }} onClose={() => setError(null)}>
          {error}
        </Alert>
      )}

      {success && (
        <Alert severity="success" sx={{ mb: 3 }} onClose={() => setSuccess(null)}>
          {success}
        </Alert>
      )}

      <Grid container spacing={3}>
        {/* Tarjeta de Estado de WAHA y QR */}
        <Grid item xs={12} md={4}>
          <Card sx={{ borderRadius: "12px", boxShadow: "0 2px 6px rgba(0,0,0,0.04)" }}>
            <CardContent>
              <Box sx={{ display: "flex", alignItems: "center", gap: 1, mb: 2 }}>
                <WhatsAppIcon sx={{ color: "#25D366", fontSize: 28 }} />
                <Typography variant="h6" sx={{ fontWeight: 600 }}>
                  Sesión de WhatsApp
                </Typography>
              </Box>

              <Box sx={{ mb: 3, p: 2, backgroundColor: "#f8fafc", borderRadius: "8px" }}>
                <Typography variant="caption" sx={{ color: "#64748b" }}>
                  ESTADO DE CONEXIÓN
                </Typography>
                <Box sx={{ display: "flex", alignItems: "center", gap: 1, mt: 0.5 }}>
                  <Chip
                    label={wahaStatus?.status || "DESCONOCIDO"}
                    color={isConnected ? "success" : "error"}
                    sx={{ fontWeight: 700 }}
                  />
                </Box>
              </Box>

              {/* Si hay código QR para escanear */}
              {wahaQr ? (
                <Box sx={{ textAlign: "center", p: 2, border: "1px dashed #cbd5e1", borderRadius: "8px", mb: 2 }}>
                  <QrCode2Icon sx={{ fontSize: 32, color: "#6366f1" }} />
                  <Typography variant="subtitle2" sx={{ fontWeight: 600, mb: 1 }}>
                    Escanea con WhatsApp en tu celular:
                  </Typography>
                  <img
                    src={wahaQr.startsWith("data:") ? wahaQr : `data:image/png;base64,${wahaQr}`}
                    alt="WAHA QR Code"
                    style={{ maxWidth: "200px", margin: "0 auto", display: "block" }}
                  />
                </Box>
              ) : (
                <Box sx={{ textAlign: "center", p: 2, mb: 2, backgroundColor: isConnected ? "#f0fdf4" : "#fef2f2", borderRadius: "8px" }}>
                  <Typography variant="body2" sx={{ color: isConnected ? "#15803d" : "#991b1b", fontWeight: 500 }}>
                    {isConnected
                      ? "✅ Sesión activa y vinculada. Lista para enviar mensajes."
                      : "⚠️ Sesión desconectada. Pulsa reconectar para regenerar QR."}
                  </Typography>
                </Box>
              )}

              <Button
                variant="outlined"
                fullWidth
                color="secondary"
                disabled={restarting}
                onClick={handleRestart}
                sx={{ fontWeight: 600 }}
              >
                {restarting ? <CircularProgress size={24} /> : "Forzar Reconexión / Reiniciar Sesión"}
              </Button>
            </CardContent>
          </Card>
        </Grid>

        {/* Tarjeta de Log de Mensajes (Cola n8n / WAHA) */}
        <Grid item xs={12} md={8}>
          <Card sx={{ borderRadius: "12px", boxShadow: "0 2px 6px rgba(0,0,0,0.04)" }}>
            <CardContent>
              <Box sx={{ display: "flex", justifyContent: "space-between", alignItems: "center", mb: 2 }}>
                <Typography variant="h6" sx={{ fontWeight: 600 }}>
                  Cola y Registro de Despachos (n8n ➔ WAHA)
                </Typography>
                <Typography variant="caption" sx={{ color: "#64748b" }}>
                  Sondeo automático por cron de n8n
                </Typography>
              </Box>

              {loading ? (
                <Box sx={{ display: "flex", justifyContent: "center", p: 4 }}>
                  <CircularProgress />
                </Box>
              ) : mensajes.length === 0 ? (
                <Box sx={{ textAlign: "center", p: 4, color: "#94a3b8" }}>
                  <Typography>No hay mensajes en cola ni historial de envíos reciente.</Typography>
                </Box>
              ) : (
                <TableContainer component={Paper} elevation={0}>
                  <Table size="small">
                    <TableHead sx={{ backgroundColor: "#f8fafc" }}>
                      <TableRow>
                        <TableCell sx={{ fontWeight: 600 }}>Tipo</TableCell>
                        <TableCell sx={{ fontWeight: 600 }}>Destinatario</TableCell>
                        <TableCell sx={{ fontWeight: 600 }}>Contenido / Mensaje</TableCell>
                        <TableCell sx={{ fontWeight: 600 }}>Estado</TableCell>
                        <TableCell sx={{ fontWeight: 600 }}>Fecha Creación</TableCell>
                      </TableRow>
                    </TableHead>
                    <TableBody>
                      {mensajes.map((m) => (
                        <TableRow key={m.id} hover>
                          <TableCell>
                            <Chip
                              label={m.tipo === "alerta" ? "ALERTA" : "BRIEF"}
                              color={m.tipo === "alerta" ? "error" : "primary"}
                              size="small"
                              sx={{ fontWeight: 700 }}
                            />
                          </TableCell>
                          <TableCell sx={{ fontFamily: "monospace", fontSize: "0.85rem" }}>
                            {m.destinatario}
                          </TableCell>
                          <TableCell sx={{ maxWidth: 260 }}>
                            <Typography
                              variant="body2"
                              sx={{
                                overflow: "hidden",
                                textOverflow: "ellipsis",
                                display: "-webkit-box",
                                WebkitLineClamp: 2,
                                WebkitBoxOrient: "vertical",
                                fontSize: "0.82rem",
                              }}
                            >
                              {m.texto}
                            </Typography>
                          </TableCell>
                          <TableCell>{getEstadoMensajeChip(m.estado)}</TableCell>
                          <TableCell sx={{ fontSize: "0.8rem", whiteSpace: "nowrap" }}>
                            {new Date(m.creado_en).toLocaleString()}
                          </TableCell>
                        </TableRow>
                      ))}
                    </TableBody>
                  </Table>
                </TableContainer>
              )}
            </CardContent>
          </Card>
        </Grid>
      </Grid>
    </Box>
  );
}
