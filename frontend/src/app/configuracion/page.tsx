"use client";

import React, { useState, useEffect } from "react";
import {
  Box,
  Card,
  CardContent,
  Typography,
  Grid,
  Button,
  Select,
  MenuItem,
  FormControl,
  InputLabel,
  Chip,
  CircularProgress,
  Alert,
  Divider,
  Paper,
  Radio,
  RadioGroup,
  FormControlLabel,
} from "@mui/material";
import SettingsIcon from "@mui/icons-material/Settings";
import SmartToyIcon from "@mui/icons-material/SmartToy";
import SpeedIcon from "@mui/icons-material/Speed";
import PsychologyIcon from "@mui/icons-material/Psychology";
import CheckCircleIcon from "@mui/icons-material/CheckCircle";
import ErrorOutlineIcon from "@mui/icons-material/ErrorOutline";
import SaveIcon from "@mui/icons-material/Save";
import PlayArrowIcon from "@mui/icons-material/PlayArrow";
import RefreshIcon from "@mui/icons-material/Refresh";
import { ApiService } from "@/lib/api";

interface AvailableModel {
  id: string;
  name: string;
  description: string;
  badge?: string;
  speed: string;
  intelligence: string;
}

export default function ConfiguracionPage() {
  const [activeModel, setActiveModel] = useState<string>("claude-3-5-sonnet-20241022");
  const [selectedModel, setSelectedModel] = useState<string>("claude-3-5-sonnet-20241022");
  const [availableModels, setAvailableModels] = useState<AvailableModel[]>([]);
  const [loading, setLoading] = useState<boolean>(true);
  const [saving, setSaving] = useState<boolean>(false);
  const [testing, setTesting] = useState<boolean>(false);
  const [testResult, setTestResult] = useState<{
    ok: boolean;
    latency_ms?: number;
    mensaje: string;
    error?: string;
  } | null>(null);
  const [successMsg, setSuccessMsg] = useState<string | null>(null);
  const [errorMsg, setErrorMsg] = useState<string | null>(null);

  useEffect(() => {
    loadConfig();
  }, []);

  const loadConfig = async () => {
    try {
      setLoading(true);
      setErrorMsg(null);
      const data = await ApiService.getConfiguraciones();
      if (data) {
        if (data.claude_model) {
          setActiveModel(data.claude_model);
          setSelectedModel(data.claude_model);
        }
        if (data.available_models) {
          setAvailableModels(data.available_models);
        }
      }
    } catch (err: any) {
      console.error("Error cargando configuración:", err);
      setErrorMsg(err.message || "No se pudo cargar la configuración.");
    } finally {
      setLoading(false);
    }
  };

  const handleSave = async () => {
    try {
      setSaving(true);
      setErrorMsg(null);
      setSuccessMsg(null);
      const res = await ApiService.updateConfiguracion(selectedModel);
      setActiveModel(selectedModel);
      setSuccessMsg(`Modelo '${selectedModel}' guardado y activado exitosamente para todas las síntesis.`);
    } catch (err: any) {
      console.error("Error guardando modelo:", err);
      setErrorMsg(err.message || "Error al guardar el modelo.");
    } finally {
      setSaving(false);
    }
  };

  const handleTest = async () => {
    try {
      setTesting(true);
      setTestResult(null);
      const res = await ApiService.testClaude(selectedModel);
      setTestResult(res);
    } catch (err: any) {
      console.error("Error probando Claude:", err);
      setTestResult({
        ok: false,
        mensaje: `Error al probar el modelo: ${err.message}`,
      });
    } finally {
      setTesting(false);
    }
  };

  return (
    <Box sx={{ p: { xs: 2, md: 3 } }}>
      {/* Encabezado */}
      <Box sx={{ mb: 3, display: "flex", justifyContent: "space-between", alignItems: "center", flexWrap: "wrap", gap: 2 }}>
        <Box>
          <Typography variant="h5" sx={{ fontWeight: 700, color: "#1e293b", display: "flex", alignItems: "center", gap: 1 }}>
            <SettingsIcon sx={{ color: "#605DFF" }} />
            Configuración del Sistema
          </Typography>
          <Typography variant="body2" sx={{ color: "#64748b" }}>
            Selección de Modelos de Inteligencia Artificial (Claude LLM) y Parámetros Operativos
          </Typography>
        </Box>

        <Button
          variant="outlined"
          startIcon={<RefreshIcon />}
          onClick={loadConfig}
          disabled={loading}
          sx={{ textTransform: "none" }}
        >
          Recargar
        </Button>
      </Box>

      {/* Alertas */}
      {successMsg && (
        <Alert severity="success" sx={{ mb: 3 }} onClose={() => setSuccessMsg(null)}>
          {successMsg}
        </Alert>
      )}
      {errorMsg && (
        <Alert severity="error" sx={{ mb: 3 }} onClose={() => setErrorMsg(null)}>
          {errorMsg}
        </Alert>
      )}

      {loading ? (
        <Box sx={{ display: "flex", justifyContent: "center", p: 8 }}>
          <CircularProgress />
        </Box>
      ) : (
        <Grid container spacing={3}>
          {/* Tarjeta de Selección y Gestión de Modelos */}
          <Grid size={{ xs: 12, lg: 8 }}>
            <Card sx={{ borderRadius: "12px", boxShadow: "0 2px 6px rgba(0,0,0,0.04)" }}>
              <CardContent sx={{ p: 3 }}>
                <Box sx={{ display: "flex", alignItems: "center", gap: 1, mb: 1 }}>
                  <SmartToyIcon sx={{ color: "#d97706" }} />
                  <Typography variant="h6" sx={{ fontWeight: 600 }}>
                    Motor de Síntesis Ejecutiva (Anthropic Claude)
                  </Typography>
                </Box>
                <Typography variant="body2" sx={{ color: "#64748b", mb: 3 }}>
                  Elige el modelo con el que se analizarán y sintetizarán los boletines de Coparmex. Puedes probar la conexión antes de guardar para verificar que tu clave de Anthropic lo admita.
                </Typography>

                {/* Selector Dropdown rápido */}
                <FormControl fullWidth sx={{ mb: 3 }}>
                  <InputLabel id="select-claude-model-label">Modelo Activo para el Pipeline</InputLabel>
                  <Select
                    labelId="select-claude-model-label"
                    value={selectedModel}
                    label="Modelo Activo para el Pipeline"
                    onChange={(e) => {
                      setSelectedModel(e.target.value);
                      setTestResult(null);
                    }}
                  >
                    {availableModels.map((m) => (
                      <MenuItem key={m.id} value={m.id}>
                        <Box sx={{ display: "flex", alignItems: "center", gap: 1, width: "100%", justifyContent: "space-between" }}>
                          <span>
                            <strong>{m.name}</strong> <span style={{ color: "#94a3b8", fontSize: "0.85rem" }}>({m.id})</span>
                          </span>
                          {m.badge && (
                            <Chip
                              label={m.badge}
                              size="small"
                              color={m.badge === "Recomendado" ? "primary" : m.badge.includes("Último") ? "secondary" : "default"}
                              sx={{ height: 20, fontSize: "0.7rem" }}
                            />
                          )}
                        </Box>
                      </MenuItem>
                    ))}
                  </Select>
                </FormControl>

                {/* Lista de Modelos con detalle en Tarjetas */}
                <Typography variant="subtitle2" sx={{ fontWeight: 600, color: "#475569", mb: 1.5 }}>
                  Modelos Disponibles:
                </Typography>

                <RadioGroup
                  value={selectedModel}
                  onChange={(e) => {
                    setSelectedModel(e.target.value);
                    setTestResult(null);
                  }}
                >
                  <Grid container spacing={2}>
                    {availableModels.map((m) => {
                      const isSelected = selectedModel === m.id;
                      const isActiveInProd = activeModel === m.id;

                      return (
                        <Grid size={{ xs: 12 }} key={m.id}>
                          <Paper
                            onClick={() => {
                              setSelectedModel(m.id);
                              setTestResult(null);
                            }}
                            sx={{
                              p: 2,
                              borderRadius: "10px",
                              cursor: "pointer",
                              border: isSelected ? "2px solid #605DFF" : "1px solid #e2e8f0",
                              backgroundColor: isSelected ? "#f8f9ff" : "#ffffff",
                              transition: "all 0.2s ease",
                              "&:hover": {
                                borderColor: "#605DFF",
                              },
                            }}
                          >
                            <Box sx={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start" }}>
                              <Box sx={{ display: "flex", alignItems: "center", gap: 1 }}>
                                <Radio
                                  checked={isSelected}
                                  value={m.id}
                                  size="small"
                                  sx={{ p: 0.5, color: "#605DFF" }}
                                />
                                <Box>
                                  <Typography variant="subtitle1" sx={{ fontWeight: 700, color: "#1e293b", display: "flex", alignItems: "center", gap: 1 }}>
                                    {m.name}
                                    {isActiveInProd && (
                                      <Chip
                                        label="EN USO ACTUAL"
                                        size="small"
                                        color="success"
                                        sx={{ fontWeight: 700, height: 20, fontSize: "0.65rem" }}
                                      />
                                    )}
                                    {m.badge && (
                                      <Chip
                                        label={m.badge}
                                        size="small"
                                        color={m.badge === "Recomendado" ? "primary" : m.badge.includes("Último") ? "secondary" : "default"}
                                        sx={{ height: 20, fontSize: "0.65rem" }}
                                      />
                                    )}
                                  </Typography>
                                  <Typography variant="caption" sx={{ color: "#64748b", fontFamily: "monospace" }}>
                                    ID: {m.id}
                                  </Typography>
                                </Box>
                              </Box>

                              <Box sx={{ display: "flex", gap: 1 }}>
                                <Chip
                                  icon={<SpeedIcon sx={{ fontSize: "14px !important" }} />}
                                  label={`Velocidad: ${m.speed}`}
                                  size="small"
                                  variant="outlined"
                                  sx={{ fontSize: "0.75rem", height: 24 }}
                                />
                                <Chip
                                  icon={<PsychologyIcon sx={{ fontSize: "14px !important" }} />}
                                  label={`Capacidad: ${m.intelligence}`}
                                  size="small"
                                  variant="outlined"
                                  sx={{ fontSize: "0.75rem", height: 24 }}
                                />
                              </Box>
                            </Box>

                            <Typography variant="body2" sx={{ color: "#475569", mt: 1, ml: 4 }}>
                              {m.description}
                            </Typography>
                          </Paper>
                        </Grid>
                      );
                    })}
                  </Grid>
                </RadioGroup>

                <Divider sx={{ my: 3 }} />

                {/* Resultado de la prueba */}
                {testResult && (
                  <Alert
                    severity={testResult.ok ? "success" : "error"}
                    sx={{ mb: 3 }}
                    icon={testResult.ok ? <CheckCircleIcon /> : <ErrorOutlineIcon />}
                  >
                    <Typography variant="subtitle2" sx={{ fontWeight: 700 }}>
                      {testResult.ok ? "Prueba Exitosa con Anthropic" : "Error de Conexión con el Modelo"}
                    </Typography>
                    <Typography variant="body2">{testResult.mensaje}</Typography>
                    {testResult.latency_ms && (
                      <Typography variant="caption" sx={{ display: "block", mt: 0.5, color: "#64748b" }}>
                        Tiempo de respuesta: {testResult.latency_ms} ms
                      </Typography>
                    )}
                  </Alert>
                )}

                {/* Botones de Acción */}
                <Box sx={{ display: "flex", gap: 2, flexWrap: "wrap" }}>
                  <Button
                    variant="contained"
                    color="primary"
                    startIcon={<SaveIcon />}
                    onClick={handleSave}
                    disabled={saving || testing || selectedModel === activeModel}
                    sx={{ fontWeight: 600, px: 3, textTransform: "none", borderRadius: "8px" }}
                  >
                    {saving ? "Guardando..." : "Guardar y Activar Modelo"}
                  </Button>

                  <Button
                    variant="outlined"
                    color="warning"
                    startIcon={testing ? <CircularProgress size={18} /> : <PlayArrowIcon />}
                    onClick={handleTest}
                    disabled={testing || saving}
                    sx={{ fontWeight: 600, px: 2.5, textTransform: "none", borderRadius: "8px" }}
                  >
                    {testing ? "Probando con Claude..." : `Probar '${selectedModel}'`}
                  </Button>
                </Box>
              </CardContent>
            </Card>
          </Grid>

          {/* Panel Lateral Informativo */}
          <Grid size={{ xs: 12, lg: 4 }}>
            <Card sx={{ borderRadius: "12px", boxShadow: "0 2px 6px rgba(0,0,0,0.04)", mb: 3 }}>
              <CardContent sx={{ p: 3 }}>
                <Typography variant="h6" sx={{ fontWeight: 600, mb: 2 }}>
                  Estado de los Servicios
                </Typography>

                <Box sx={{ display: "flex", flexDirection: "column", gap: 2 }}>
                  <Box sx={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                    <Typography variant="body2" sx={{ color: "#64748b" }}>
                      Modelo Activo en Producción:
                    </Typography>
                    <Chip label={activeModel} size="small" color="success" sx={{ fontWeight: 600, fontFamily: "monospace" }} />
                  </Box>

                  <Box sx={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                    <Typography variant="body2" sx={{ color: "#64748b" }}>
                      Pipeline Python (Workers):
                    </Typography>
                    <Chip label="Puerto 8091 · Activo" size="small" variant="outlined" color="primary" sx={{ fontWeight: 600 }} />
                  </Box>

                  <Box sx={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                    <Typography variant="body2" sx={{ color: "#64748b" }}>
                      Backend Rust (API Core):
                    </Typography>
                    <Chip label="Puerto 8090 · Activo" size="small" variant="outlined" color="primary" sx={{ fontWeight: 600 }} />
                  </Box>

                  <Box sx={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                    <Typography variant="body2" sx={{ color: "#64748b" }}>
                      Persistencia:
                    </Typography>
                    <Typography variant="body2" sx={{ fontWeight: 600, color: "#1e293b" }}>
                      PostgreSQL (exposureiq_db)
                    </Typography>
                  </Box>
                </Box>
              </CardContent>
            </Card>

            <Card sx={{ borderRadius: "12px", boxShadow: "0 2px 6px rgba(0,0,0,0.04)", backgroundColor: "#f8fafc" }}>
              <CardContent sx={{ p: 3 }}>
                <Typography variant="subtitle2" sx={{ fontWeight: 700, color: "#1e293b", mb: 1 }}>
                  💡 Guía de Selección de Modelos
                </Typography>
                <Typography variant="body2" sx={{ color: "#64748b", mb: 2, lineHeight: 1.6 }}>
                  • <strong>Claude 3.5 Sonnet:</strong> La opción recomendada para el briefing del Director. Genera análisis estructurado y detecta impacto en siniestros y regulación con precisión quirúrgica.
                </Typography>
                <Typography variant="body2" sx={{ color: "#64748b", mb: 2, lineHeight: 1.6 }}>
                  • <strong>Claude 3.5 Haiku:</strong> Excelente alternativa si buscas la máxima velocidad de respuesta a un costo operativo mínimo.
                </Typography>
                <Typography variant="body2" sx={{ color: "#64748b", lineHeight: 1.6 }}>
                  • <strong>Claude 3.7 Sonnet:</strong> La generación más reciente de Anthropic con capacidad de razonamiento híbrido para casos de análisis profundo.
                </Typography>
              </CardContent>
            </Card>
          </Grid>
        </Grid>
      )}
    </Box>
  );
}
