"use client";

import React, { useState, useEffect } from "react";
import Link from "next/link";
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
  Divider,
} from "@mui/material";
import WhatsAppIcon from "@mui/icons-material/WhatsApp";
import RefreshIcon from "@mui/icons-material/Refresh";
import CheckCircleOutlineIcon from "@mui/icons-material/CheckCircleOutline";
import ErrorOutlineIcon from "@mui/icons-material/ErrorOutline";
import HourglassEmptyIcon from "@mui/icons-material/HourglassEmpty";
import SendIcon from "@mui/icons-material/Send";
import SettingsIcon from "@mui/icons-material/Settings";
import CloudDoneIcon from "@mui/icons-material/CloudDone";
import ContactPhoneIcon from "@mui/icons-material/ContactPhone";
import { ApiService, Destinatario } from "@/lib/api";

export default function CanalWhatsAppPage() {
  const [config, setConfig] = useState<any>(null);
  const [destinatarios, setDestinatarios] = useState<Destinatario[]>([]);
  const [mensajes, setMensajes] = useState<any[]>([]);
  const [loading, setLoading] = useState<boolean>(true);
  const [testing, setTesting] = useState<boolean>(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  // Valores activos consolidados (API + localStorage)
  const [activeKey, setActiveKey] = useState<string>("");
  const [activePhoneId, setActivePhoneId] = useState<string>("");
  const [activeDirectorPhone, setActiveDirectorPhone] = useState<string>("");

  const loadData = async () => {
    try {
      setLoading(true);
      setError(null);

      // Respaldo en localStorage
      const localKey = typeof window !== "undefined" ? localStorage.getItem("exposureiq_kapso_api_key") || "" : "";
      const localPhoneId = typeof window !== "undefined" ? localStorage.getItem("exposureiq_kapso_phone_id") || "" : "";
      const localDirectorPhone = typeof window !== "undefined" ? localStorage.getItem("exposureiq_director_phone") || "" : "";

      const [cfg, dests, historial] = await Promise.all([
        ApiService.getConfiguraciones().catch(() => null),
        ApiService.getDestinatarios().catch(() => []),
        ApiService.getHistorialMensajes().catch(() => []),
      ]);

      const resolvedKey = (cfg?.kapso_api_key && cfg.kapso_api_key.trim()) ? cfg.kapso_api_key.trim() : localKey;
      const resolvedPhoneId = (cfg?.kapso_phone_number_id && cfg.kapso_phone_number_id.trim()) ? cfg.kapso_phone_number_id.trim() : localPhoneId;
      const rawCandidatePhone = (cfg?.director_whatsapp_phone && cfg.director_whatsapp_phone.trim()) ? cfg.director_whatsapp_phone.trim() : localDirectorPhone;
      const resolvedDirectorPhone = (rawCandidatePhone && rawCandidatePhone !== "5215512345678") ? rawCandidatePhone : "";

      setActiveKey(resolvedKey);
      setActivePhoneId(resolvedPhoneId);
      setActiveDirectorPhone(resolvedDirectorPhone);

      setConfig(cfg);
      setDestinatarios(dests || []);
      setMensajes(historial || []);

      // Auto-sincronizar a la base de datos si estaba en localStorage pero ausente en backend
      if ((!cfg?.kapso_api_key || !cfg?.kapso_phone_number_id) && (resolvedKey && resolvedPhoneId)) {
        ApiService.updateConfiguracion({
          kapso_api_key: resolvedKey,
          kapso_phone_number_id: resolvedPhoneId,
          director_whatsapp_phone: resolvedDirectorPhone || undefined,
        }).catch(() => {});
      }
    } catch (err: any) {
      setError(err.message || "Error al cargar la información del canal WhatsApp");
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadData();
    const interval = setInterval(loadData, 20000);
    return () => clearInterval(interval);
  }, []);

  const handleTestConnection = async () => {
    try {
      setTesting(true);
      setError(null);
      setSuccess(null);

      const res = await ApiService.testWhatsapp({
        phone: activeDirectorPhone,
        api_key: activeKey,
        phone_number_id: activePhoneId,
      });

      if (res.ok) {
        setSuccess(
          `✅ Mensaje de prueba despachado con éxito. (ID: ${
            res.message_id || res.id || "Entregado"
          })`
        );
      } else {
        setError(`Fallo al enviar mensaje de prueba: ${res.error || res.mensaje || "Error desconocido"}`);
      }
      setTimeout(loadData, 2000);
    } catch (err: any) {
      setError(`Error en prueba de envío: ${err.message || "Revisa tus credenciales de WhatsApp"}`);
    } finally {
      setTesting(false);
    }
  };

  const getEstadoMensajeChip = (estado: string) => {
    switch (estado) {
      case "confirmado":
        return <Chip label="Entregado" color="success" size="small" icon={<CheckCircleOutlineIcon />} />;
      case "entregado_a_n8n":
      case "procesando":
        return <Chip label="Enviando..." color="info" size="small" icon={<HourglassEmptyIcon />} />;
      case "pendiente":
        return <Chip label="En cola" color="warning" size="small" />;
      default:
        return <Chip label={estado || "Error"} color="error" size="small" icon={<ErrorOutlineIcon />} />;
    }
  };

  const isConnected = Boolean(activePhoneId && (activeKey || config?.kapso_api_key));
  const activeDestinatariosCount = destinatarios.filter((d) => d.activo).length;

  return (
    <Box sx={{ p: 3 }}>
      {/* Encabezado */}
      <Box sx={{ display: "flex", justifyContent: "space-between", alignItems: "center", mb: 3 }}>
        <Box>
          <Typography variant="h5" sx={{ fontWeight: 700, color: "#1e293b" }}>
            Canal WhatsApp
          </Typography>
          <Typography variant="body2" sx={{ color: "#64748b" }}>
            Monitoreo de entregas y estado del servicio de mensajería ejecutiva.
          </Typography>
        </Box>
        <Button
          variant="outlined"
          startIcon={<RefreshIcon />}
          onClick={loadData}
          disabled={loading}
          sx={{ borderRadius: "8px" }}
        >
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
        {/* Tarjeta de Estado del Canal */}
        <Grid size={{ xs: 12, md: 4 }}>
          <Card sx={{ borderRadius: "12px", boxShadow: "0 2px 6px rgba(0,0,0,0.04)" }}>
            <CardContent>
              <Box sx={{ display: "flex", alignItems: "center", gap: 1.5, mb: 2 }}>
                <WhatsAppIcon sx={{ color: "#25D366", fontSize: 32 }} />
                <Box>
                  <Typography variant="h6" sx={{ fontWeight: 600, lineHeight: 1.2 }}>
                    WhatsApp Business Cloud
                  </Typography>
                  <Typography variant="caption" sx={{ color: "#64748b" }}>
                    Conexión Oficial
                  </Typography>
                </Box>
              </Box>

              <Box sx={{ mb: 2.5, p: 2, backgroundColor: "#f8fafc", borderRadius: "8px", border: "1px solid #e2e8f0" }}>
                <Typography variant="caption" sx={{ color: "#64748b", fontWeight: 700, textTransform: "uppercase" }}>
                  ESTADO DEL SERVICIO
                </Typography>
                <Box sx={{ display: "flex", alignItems: "center", gap: 1, mt: 0.5 }}>
                  <Chip
                    icon={isConnected ? <CloudDoneIcon /> : <ErrorOutlineIcon />}
                    label={isConnected ? "CONECTADO" : "PENDIENTE"}
                    color={isConnected ? "success" : "warning"}
                    sx={{ fontWeight: 700 }}
                  />
                </Box>
              </Box>

              <Box sx={{ mb: 2.5, display: "flex", flexDirection: "column", gap: 1.5 }}>
                <Box sx={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                  <Typography variant="body2" sx={{ color: "#64748b" }}>
                    Proveedor:
                  </Typography>
                  <Typography variant="body2" sx={{ fontWeight: 600, color: "#1e293b" }}>
                    WhatsApp Business Cloud API
                  </Typography>
                </Box>
                <Divider />
                <Box sx={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                  <Typography variant="body2" sx={{ color: "#64748b" }}>
                    Phone Number ID:
                  </Typography>
                  <Typography
                    variant="body2"
                    sx={{
                      fontWeight: 600,
                      fontFamily: "monospace",
                      backgroundColor: "#f1f5f9",
                      px: 1,
                      py: 0.2,
                      borderRadius: "4px",
                    }}
                  >
                    {activePhoneId || "No asignado"}
                  </Typography>
                </Box>
                <Divider />
                <Box sx={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                  <Typography variant="body2" sx={{ color: "#64748b" }}>
                    Destino Principal:
                  </Typography>
                  <Typography variant="body2" sx={{ fontWeight: 600, fontFamily: "monospace" }}>
                    {activeDirectorPhone || "No configurado"}
                  </Typography>
                </Box>
                <Divider />
                <Box sx={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                  <Typography variant="body2" sx={{ color: "#64748b", display: "flex", alignItems: "center", gap: 0.5 }}>
                    <ContactPhoneIcon sx={{ fontSize: 16 }} /> Contactos en Lista:
                  </Typography>
                  <Chip
                    label={`${activeDestinatariosCount} activos`}
                    size="small"
                    color={activeDestinatariosCount > 0 ? "primary" : "default"}
                    sx={{ fontWeight: 600 }}
                  />
                </Box>
              </Box>

              <Box sx={{ display: "flex", flexDirection: "column", gap: 1.5 }}>
                <Button
                  variant="contained"
                  fullWidth
                  color="success"
                  startIcon={testing ? <CircularProgress size={18} color="inherit" /> : <SendIcon />}
                  disabled={testing || !isConnected}
                  onClick={handleTestConnection}
                  sx={{
                    fontWeight: 600,
                    backgroundColor: "#16a34a",
                    "&:hover": { backgroundColor: "#15803d" },
                  }}
                >
                  {testing ? "Enviando prueba..." : "Enviar Mensaje de Prueba"}
                </Button>

                <Button
                  component={Link}
                  href="/configuracion"
                  variant="outlined"
                  fullWidth
                  startIcon={<SettingsIcon />}
                  sx={{ fontWeight: 600, color: "#475569", borderColor: "#cbd5e1" }}
                >
                  Configurar Parámetros
                </Button>
              </Box>
            </CardContent>
          </Card>
        </Grid>

        {/* Tarjeta de Log de Mensajes */}
        <Grid size={{ xs: 12, md: 8 }}>
          <Card sx={{ borderRadius: "12px", boxShadow: "0 2px 6px rgba(0,0,0,0.04)" }}>
            <CardContent>
              <Box sx={{ display: "flex", justifyContent: "space-between", alignItems: "center", mb: 2 }}>
                <Box>
                  <Typography variant="h6" sx={{ fontWeight: 600 }}>
                    Registro de Despachos
                  </Typography>
                  <Typography variant="caption" sx={{ color: "#64748b" }}>
                    Historial de mensajes emitidos en tiempo real
                  </Typography>
                </Box>
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
                <TableContainer component={Paper} elevation={0} sx={{ maxHeight: 600 }}>
                  <Table size="small" stickyHeader>
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
                          <TableCell sx={{ maxWidth: 300 }}>
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
