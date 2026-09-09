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
  Tabs,
  Tab,
  TextField,
  Tooltip,
} from "@mui/material";
import SettingsIcon from "@mui/icons-material/Settings";
import SmartToyIcon from "@mui/icons-material/SmartToy";
import DescriptionIcon from "@mui/icons-material/Description";
import SpeedIcon from "@mui/icons-material/Speed";
import PsychologyIcon from "@mui/icons-material/Psychology";
import CheckCircleIcon from "@mui/icons-material/CheckCircle";
import ErrorOutlineIcon from "@mui/icons-material/ErrorOutline";
import SaveIcon from "@mui/icons-material/Save";
import PlayArrowIcon from "@mui/icons-material/PlayArrow";
import RefreshIcon from "@mui/icons-material/Refresh";
import RestartAltIcon from "@mui/icons-material/RestartAlt";
import AutoFixHighIcon from "@mui/icons-material/AutoFixHigh";
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
  const [tabIndex, setTabIndex] = useState<number>(0);

  // Estados para Modelos
  const [activeModel, setActiveModel] = useState<string>("claude-sonnet-4-5-20250929");
  const [selectedModel, setSelectedModel] = useState<string>("claude-sonnet-4-5-20250929");
  const [availableModels, setAvailableModels] = useState<AvailableModel[]>([]);

  // Estados para Prompt
  const [systemPrompt, setSystemPrompt] = useState<string>("");
  const [savedPrompt, setSavedPrompt] = useState<string>("");
  const [defaultSystemPrompt, setDefaultSystemPrompt] = useState<string>("");

  // Estados generales
  const [loading, setLoading] = useState<boolean>(true);
  const [savingModel, setSavingModel] = useState<boolean>(false);
  const [savingPrompt, setSavingPrompt] = useState<boolean>(false);
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
        if (data.system_prompt) {
          setSystemPrompt(data.system_prompt);
          setSavedPrompt(data.system_prompt);
        }
        if (data.default_system_prompt) {
          setDefaultSystemPrompt(data.default_system_prompt);
        }
      }
    } catch (err: any) {
      console.error("Error cargando configuración:", err);
      setErrorMsg(err.message || "No se pudo cargar la configuración.");
    } finally {
      setLoading(false);
    }
  };

  const handleSaveModel = async () => {
    try {
      setSavingModel(true);
      setErrorMsg(null);
      setSuccessMsg(null);
      await ApiService.updateConfiguracion({ claude_model: selectedModel });
      setActiveModel(selectedModel);
      setSuccessMsg(`Modelo '${selectedModel}' guardado y activado exitosamente para todas las síntesis.`);
    } catch (err: any) {
      console.error("Error guardando modelo:", err);
      setErrorMsg(err.message || "Error al guardar el modelo.");
    } finally {
      setSavingModel(false);
    }
  };

  const handleSavePrompt = async () => {
    try {
      setSavingPrompt(true);
      setErrorMsg(null);
      setSuccessMsg(null);
      await ApiService.updateConfiguracion({ system_prompt: systemPrompt });
      setSavedPrompt(systemPrompt);
      setSuccessMsg("Instrucciones del Prompt guardadas exitosamente. Claude las aplicará en todos los nuevos boletines.");
    } catch (err: any) {
      console.error("Error guardando prompt:", err);
      setErrorMsg(err.message || "Error al guardar las instrucciones.");
    } finally {
      setSavingPrompt(false);
    }
  };

  const handleResetPromptToDefault = () => {
    if (defaultSystemPrompt) {
      setSystemPrompt(defaultSystemPrompt);
      setSuccessMsg("Se restauró el texto maestro por defecto. Recuerda hacer clic en 'Guardar Prompt' para confirmarlo.");
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
      {/* Encabezado Principal */}
      <Box sx={{ mb: 3, display: "flex", justifyContent: "space-between", alignItems: "center", flexWrap: "wrap", gap: 2 }}>
        <Box>
          <Typography variant="h5" sx={{ fontWeight: 700, color: "#1e293b", display: "flex", alignItems: "center", gap: 1 }}>
            <SettingsIcon sx={{ color: "#605DFF" }} />
            Configuración del Sistema
          </Typography>
          <Typography variant="body2" sx={{ color: "#64748b" }}>
            Control de Modelos de Inteligencia Artificial e Instrucciones de Síntesis Ejecutiva
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

      {/* Navegación por Pestañas */}
      <Box sx={{ borderBottom: 1, borderColor: "divider", mb: 3 }}>
        <Tabs
          value={tabIndex}
          onChange={(_, v) => {
            setTabIndex(v);
            setSuccessMsg(null);
            setErrorMsg(null);
          }}
          textColor="primary"
          indicatorColor="primary"
        >
          <Tab
            icon={<SmartToyIcon />}
            iconPosition="start"
            label="Modelos Claude"
            sx={{ fontWeight: 600, textTransform: "none", fontSize: "0.95rem" }}
          />
          <Tab
            icon={<DescriptionIcon />}
            iconPosition="start"
            label="Prompt / Instrucciones"
            sx={{ fontWeight: 600, textTransform: "none", fontSize: "0.95rem" }}
          />
        </Tabs>
      </Box>

      {/* Alertas Globales */}
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
        <>
          {/* ================================================================= */}
          {/* PESTAÑA 0: MODELOS CLAUDE                                        */}
          {/* ================================================================= */}
          {tabIndex === 0 && (
            <Grid container spacing={3}>
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
                                  color={m.badge === "Recomendado" ? "primary" : m.badge.includes("Nueva") || m.badge.includes("Último") ? "secondary" : "default"}
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
                      Modelos Disponibles en tu Cuenta de Anthropic:
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
                                <Box sx={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start", flexWrap: "wrap", gap: 1 }}>
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
                                            color={m.badge === "Recomendado" ? "primary" : m.badge.includes("Nueva") || m.badge.includes("Último") ? "secondary" : "default"}
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
                        onClick={handleSaveModel}
                        disabled={savingModel || testing || selectedModel === activeModel}
                        sx={{ fontWeight: 600, px: 3, textTransform: "none", borderRadius: "8px" }}
                      >
                        {savingModel ? "Guardando..." : "Guardar y Activar Modelo"}
                      </Button>

                      <Button
                        variant="outlined"
                        color="warning"
                        startIcon={testing ? <CircularProgress size={18} /> : <PlayArrowIcon />}
                        onClick={handleTest}
                        disabled={testing || savingModel}
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
                      Estado del Motor
                    </Typography>

                    <Box sx={{ display: "flex", flexDirection: "column", gap: 2 }}>
                      <Box sx={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                        <Typography variant="body2" sx={{ color: "#64748b" }}>
                          Modelo Activo:
                        </Typography>
                        <Chip label={activeModel} size="small" color="success" sx={{ fontWeight: 600, fontFamily: "monospace" }} />
                      </Box>

                      <Box sx={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                        <Typography variant="body2" sx={{ color: "#64748b" }}>
                          Tokens por Síntesis:
                        </Typography>
                        <Chip label="1,000 max" size="small" variant="outlined" sx={{ fontWeight: 600 }} />
                      </Box>

                      <Box sx={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                        <Typography variant="body2" sx={{ color: "#64748b" }}>
                          Persistencia Dinámica:
                        </Typography>
                        <Typography variant="body2" sx={{ fontWeight: 600, color: "#16a34a" }}>
                          En PostgreSQL (Inmediata)
                        </Typography>
                      </Box>
                    </Box>
                  </CardContent>
                </Card>

                <Card sx={{ borderRadius: "12px", boxShadow: "0 2px 6px rgba(0,0,0,0.04)", backgroundColor: "#f8fafc" }}>
                  <CardContent sx={{ p: 3 }}>
                    <Typography variant="subtitle2" sx={{ fontWeight: 700, color: "#1e293b", mb: 1 }}>
                      💡 Recomendación de Modelos
                    </Typography>
                    <Typography variant="body2" sx={{ color: "#475569", mb: 2, lineHeight: 1.6 }}>
                      • <strong>Claude Sonnet 4.5:</strong> El modelo recomendado por defecto. Redacción fluida, precisa para WhatsApp y compresión analítica superior.
                    </Typography>
                    <Typography variant="body2" sx={{ color: "#475569", lineHeight: 1.6 }}>
                      • <strong>Claude Haiku 4.5:</strong> Si se requiere ultra velocidad y costo de centavos al mes.
                    </Typography>
                  </CardContent>
                </Card>
              </Grid>
            </Grid>
          )}

          {/* ================================================================= */}
          {/* PESTAÑA 1: PROMPT / INSTRUCCIONES                                */}
          {/* ================================================================= */}
          {tabIndex === 1 && (
            <Grid container spacing={3}>
              <Grid size={{ xs: 12, lg: 8 }}>
                <Card sx={{ borderRadius: "12px", boxShadow: "0 2px 6px rgba(0,0,0,0.04)" }}>
                  <CardContent sx={{ p: 3 }}>
                    <Box sx={{ display: "flex", alignItems: "center", gap: 1, mb: 1 }}>
                      <AutoFixHighIcon sx={{ color: "#605DFF" }} />
                      <Typography variant="h6" sx={{ fontWeight: 600 }}>
                        Instrucciones del Sistema para Claude (System Prompt)
                      </Typography>
                    </Box>
                    <Typography variant="body2" sx={{ color: "#64748b", mb: 2 }}>
                      Estas directrices guían el razonamiento de Claude al leer las páginas del boletín. Puedes ajustar el enfoque de siniestralidad, el tono ejecutivo y las reglas de formato para WhatsApp.
                    </Typography>

                    {/* Chips de Ayuda y Contexto */}
                    <Box sx={{ mb: 2, display: "flex", gap: 1, flexWrap: "wrap", alignItems: "center" }}>
                      <Typography variant="caption" sx={{ fontWeight: 600, color: "#475569" }}>
                        Variables inyectadas en tiempo de ejecución:
                      </Typography>
                      <Chip label="{fecha_str}" size="small" variant="outlined" sx={{ fontFamily: "monospace", fontSize: "0.75rem" }} />
                      <Chip label="{extractos_tematicos_ocr}" size="small" variant="outlined" sx={{ fontFamily: "monospace", fontSize: "0.75rem" }} />
                      <Chip label="{paginas_inicio_fin}" size="small" variant="outlined" sx={{ fontFamily: "monospace", fontSize: "0.75rem" }} />
                    </Box>

                    {/* Editor de Texto de Prompt */}
                    <TextField
                      multiline
                      fullWidth
                      minRows={16}
                      maxRows={24}
                      value={systemPrompt}
                      onChange={(e) => setSystemPrompt(e.target.value)}
                      placeholder="Escribe aquí el System Prompt para Claude..."
                      sx={{
                        mb: 3,
                        "& .MuiInputBase-root": {
                          fontFamily: "'Fira Code', 'Consolas', monospace",
                          fontSize: "0.88rem",
                          lineHeight: 1.6,
                          backgroundColor: "#f8fafc",
                          borderRadius: "8px",
                        },
                      }}
                    />

                    {/* Botones de Guardar y Restaurar */}
                    <Box sx={{ display: "flex", justifyContent: "space-between", alignItems: "center", flexWrap: "wrap", gap: 2 }}>
                      <Box sx={{ display: "flex", gap: 2 }}>
                        <Button
                          variant="contained"
                          color="primary"
                          startIcon={<SaveIcon />}
                          onClick={handleSavePrompt}
                          disabled={savingPrompt || systemPrompt === savedPrompt || !systemPrompt.trim()}
                          sx={{ fontWeight: 600, px: 3, textTransform: "none", borderRadius: "8px" }}
                        >
                          {savingPrompt ? "Guardando..." : "Guardar Instrucciones"}
                        </Button>

                        <Button
                          variant="outlined"
                          color="inherit"
                          startIcon={<RestartAltIcon />}
                          onClick={handleResetPromptToDefault}
                          disabled={savingPrompt || systemPrompt === defaultSystemPrompt}
                          sx={{ textTransform: "none", borderRadius: "8px" }}
                        >
                          Restaurar por Defecto
                        </Button>
                      </Box>

                      {systemPrompt !== savedPrompt && (
                        <Typography variant="caption" sx={{ color: "#d97706", fontWeight: 600 }}>
                          ⚠️ Tienes cambios sin guardar en el prompt.
                        </Typography>
                      )}
                    </Box>
                  </CardContent>
                </Card>
              </Grid>

              {/* Panel Lateral de Consejos para el Prompt */}
              <Grid size={{ xs: 12, lg: 4 }}>
                <Card sx={{ borderRadius: "12px", boxShadow: "0 2px 6px rgba(0,0,0,0.04)", mb: 3 }}>
                  <CardContent sx={{ p: 3 }}>
                    <Typography variant="h6" sx={{ fontWeight: 600, mb: 1.5 }}>
                      💡 Mejores Prácticas
                    </Typography>

                    <Typography variant="body2" sx={{ color: "#475569", mb: 2, lineHeight: 1.6 }}>
                      1. <strong>Enfoque en Aseguradoras:</strong> Mantén siempre la directriz de correlacionar noticias con ramos de seguro (Transporte/Carga, Autos, Gastos Médicos, Daños).
                    </Typography>

                    <Typography variant="body2" sx={{ color: "#475569", mb: 2, lineHeight: 1.6 }}>
                      2. <strong>Filtrado de Ruido Político:</strong> Instruye a Claude descartar declaraciones partidistas que no afecten operaciones o marcos regulatorios (CNSF/SHCP).
                    </Typography>

                    <Typography variant="body2" sx={{ color: "#475569", mb: 2, lineHeight: 1.6 }}>
                      3. <strong>Formato para WhatsApp:</strong> Pide usar negritas con un solo asterisco (<code>*Título*</code>) y viñetas con guiones (<code>- punto</code>) para que el mensaje sea limpio en pantalla móvil.
                    </Typography>

                    <Typography variant="body2" sx={{ color: "#475569", lineHeight: 1.6 }}>
                      4. <strong>Llamado a la Acción:</strong> Concluye siempre con un bloque de <em>"Atención Operativa Sugerida"</em> para el Director de Operaciones.
                    </Typography>
                  </CardContent>
                </Card>
              </Grid>
            </Grid>
          )}
        </>
      )}
    </Box>
  );
}
