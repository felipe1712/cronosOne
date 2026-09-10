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
  IconButton,
  InputAdornment,
} from "@mui/material";
import SettingsIcon from "@mui/icons-material/Settings";
import SmartToyIcon from "@mui/icons-material/SmartToy";
import DescriptionIcon from "@mui/icons-material/Description";
import WhatsAppIcon from "@mui/icons-material/WhatsApp";
import SpeedIcon from "@mui/icons-material/Speed";
import PsychologyIcon from "@mui/icons-material/Psychology";
import CheckCircleIcon from "@mui/icons-material/CheckCircle";
import ErrorOutlineIcon from "@mui/icons-material/ErrorOutline";
import SaveIcon from "@mui/icons-material/Save";
import PlayArrowIcon from "@mui/icons-material/PlayArrow";
import RefreshIcon from "@mui/icons-material/Refresh";
import RestartAltIcon from "@mui/icons-material/RestartAlt";
import AutoFixHighIcon from "@mui/icons-material/AutoFixHigh";
import SendIcon from "@mui/icons-material/Send";
import VisibilityIcon from "@mui/icons-material/Visibility";
import VisibilityOffIcon from "@mui/icons-material/VisibilityOff";
import LaunchIcon from "@mui/icons-material/Launch";
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

  // Estados para WhatsApp (Kapso)
  const [whatsappProvider, setWhatsappProvider] = useState<string>("kapso");
  const [kapsoApiKey, setKapsoApiKey] = useState<string>("");
  const [kapsoPhoneNumberId, setKapsoPhoneNumberId] = useState<string>("");
  const [directorWhatsappPhone, setDirectorWhatsappPhone] = useState<string>("5215512345678");
  const [showApiKey, setShowApiKey] = useState<boolean>(false);
  const [savingWhatsapp, setSavingWhatsapp] = useState<boolean>(false);
  const [testingWhatsapp, setTestingWhatsapp] = useState<boolean>(false);
  const [whatsappTestResult, setWhatsappTestResult] = useState<{
    ok: boolean;
    latency_ms?: number;
    mensaje: string;
    error?: string;
  } | null>(null);

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
        if (data.whatsapp_provider) {
          setWhatsappProvider(data.whatsapp_provider);
        }
        if (data.kapso_api_key) {
          setKapsoApiKey(data.kapso_api_key);
        }
        if (data.kapso_phone_number_id) {
          setKapsoPhoneNumberId(data.kapso_phone_number_id);
        }
        if (data.director_whatsapp_phone) {
          setDirectorWhatsappPhone(data.director_whatsapp_phone);
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

  const handleSaveWhatsapp = async () => {
    try {
      setSavingWhatsapp(true);
      setErrorMsg(null);
      setSuccessMsg(null);
      await ApiService.updateConfiguracion({
        whatsapp_provider: whatsappProvider,
        kapso_api_key: kapsoApiKey,
        kapso_phone_number_id: kapsoPhoneNumberId,
        director_whatsapp_phone: directorWhatsappPhone,
      });
      setSuccessMsg("Configuración de WhatsApp guardada exitosamente en el sistema.");
    } catch (err: any) {
      console.error("Error guardando configuración de WhatsApp:", err);
      setErrorMsg(err.message || "Error al guardar configuración de WhatsApp.");
    } finally {
      setSavingWhatsapp(false);
    }
  };

  const handleTestWhatsapp = async () => {
    try {
      setTestingWhatsapp(true);
      setWhatsappTestResult(null);
      setErrorMsg(null);
      const res = await ApiService.testWhatsapp({
        phone: directorWhatsappPhone,
        message: "🔔 *Prueba de Conexión ExposureIQ — WhatsApp Cloud API via Kapso*\n\nSi estás leyendo esto, la entrega oficial de mensajes está 100% activa y funcionando.",
      });
      setWhatsappTestResult(res);
    } catch (err: any) {
      console.error("Error probando WhatsApp con Kapso:", err);
      setWhatsappTestResult({
        ok: false,
        mensaje: `Error al probar WhatsApp: ${err.message}`,
      });
    } finally {
      setTestingWhatsapp(false);
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
            Modelos de IA, Directrices de Prompt y Canal de Entrega WhatsApp (Kapso Cloud API)
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
          <Tab
            icon={<WhatsAppIcon sx={{ color: "#25D366" }} />}
            iconPosition="start"
            label="Canal WhatsApp (Kapso)"
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

          {/* ================================================================= */}
          {/* PESTAÑA 2: CANAL WHATSAPP (KAPSO CLOUD API)                       */}
          {/* ================================================================= */}
          {tabIndex === 2 && (
            <Grid container spacing={3}>
              <Grid size={{ xs: 12, lg: 8 }}>
                <Card sx={{ borderRadius: "12px", boxShadow: "0 2px 6px rgba(0,0,0,0.04)" }}>
                  <CardContent sx={{ p: 3 }}>
                    <Box sx={{ display: "flex", alignItems: "center", gap: 1, mb: 1 }}>
                      <WhatsAppIcon sx={{ color: "#25D366" }} />
                      <Typography variant="h6" sx={{ fontWeight: 600 }}>
                        Integración WhatsApp Cloud API (vía Kapso)
                      </Typography>
                    </Box>
                    <Typography variant="body2" sx={{ color: "#64748b", mb: 3 }}>
                      Entrega directa y oficial a través de los servidores de Meta. No requiere escanear códigos QR, no sufre caídas de sesión de WhatsApp Web y entrega los mensajes en milisegundos.
                    </Typography>

                    <Grid container spacing={2.5}>
                      {/* Proveedor activo */}
                      <Grid size={{ xs: 12, sm: 6 }}>
                        <FormControl fullWidth>
                          <InputLabel id="whatsapp-provider-label">Proveedor de Entrega</InputLabel>
                          <Select
                            labelId="whatsapp-provider-label"
                            value={whatsappProvider}
                            label="Proveedor de Entrega"
                            onChange={(e) => setWhatsappProvider(e.target.value)}
                          >
                            <MenuItem value="kapso">
                              <Box sx={{ display: "flex", alignItems: "center", gap: 1 }}>
                                <strong>Kapso</strong> (Cloud API Oficial Meta · Recomendado)
                              </Box>
                            </MenuItem>
                            <MenuItem value="waha">
                              <Box sx={{ display: "flex", alignItems: "center", gap: 1 }}>
                                <span>WAHA</span> (WhatsApp Web Local / n8n)
                              </Box>
                            </MenuItem>
                          </Select>
                        </FormControl>
                      </Grid>

                      {/* Teléfono del Director */}
                      <Grid size={{ xs: 12, sm: 6 }}>
                        <TextField
                          fullWidth
                          label="Teléfono WhatsApp del Director"
                          value={directorWhatsappPhone}
                          onChange={(e) => setDirectorWhatsappPhone(e.target.value)}
                          placeholder="5215512345678"
                          helperText="Formato internacional E.164 sin signo + (ej: 5215512345678)"
                        />
                      </Grid>

                      {/* Kapso API Key */}
                      <Grid size={{ xs: 12 }}>
                        <TextField
                          fullWidth
                          label="Kapso API Key (X-API-Key)"
                          type={showApiKey ? "text" : "password"}
                          value={kapsoApiKey}
                          onChange={(e) => setKapsoApiKey(e.target.value)}
                          placeholder="kapso_live_..."
                          helperText="Obtenla en dashboard.kapso.ai → Integrations → API keys"
                          InputProps={{
                            endAdornment: (
                              <InputAdornment position="end">
                                <IconButton onClick={() => setShowApiKey(!showApiKey)} edge="end">
                                  {showApiKey ? <VisibilityOffIcon /> : <VisibilityIcon />}
                                </IconButton>
                              </InputAdornment>
                            ),
                          }}
                        />
                      </Grid>

                      {/* Phone Number ID */}
                      <Grid size={{ xs: 12 }}>
                        <TextField
                          fullWidth
                          label="Kapso / Meta Phone Number ID"
                          value={kapsoPhoneNumberId}
                          onChange={(e) => setKapsoPhoneNumberId(e.target.value)}
                          placeholder="647015955153740"
                          helperText="ID numérico asignado en dashboard.kapso.ai → WhatsApp → Phone numbers"
                        />
                      </Grid>
                    </Grid>

                    <Divider sx={{ my: 3 }} />

                    {/* Resultado del test de WhatsApp */}
                    {whatsappTestResult && (
                      <Alert
                        severity={whatsappTestResult.ok ? "success" : "error"}
                        sx={{ mb: 3 }}
                        icon={whatsappTestResult.ok ? <CheckCircleIcon /> : <ErrorOutlineIcon />}
                      >
                        <Typography variant="subtitle2" sx={{ fontWeight: 700 }}>
                          {whatsappTestResult.ok ? "Mensaje Enviado con Éxito a WhatsApp" : "Fallo en la Conexión de WhatsApp"}
                        </Typography>
                        <Typography variant="body2">{whatsappTestResult.mensaje}</Typography>
                        {whatsappTestResult.latency_ms && (
                          <Typography variant="caption" sx={{ display: "block", mt: 0.5, color: "#64748b" }}>
                            Latencia de entrega a Meta: {whatsappTestResult.latency_ms} ms
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
                        onClick={handleSaveWhatsapp}
                        disabled={savingWhatsapp || testingWhatsapp}
                        sx={{ fontWeight: 600, px: 3, textTransform: "none", borderRadius: "8px" }}
                      >
                        {savingWhatsapp ? "Guardando..." : "Guardar Configuración WhatsApp"}
                      </Button>

                      <Button
                        variant="outlined"
                        color="success"
                        startIcon={testingWhatsapp ? <CircularProgress size={18} /> : <SendIcon />}
                        onClick={handleTestWhatsapp}
                        disabled={testingWhatsapp || savingWhatsapp || !kapsoApiKey.trim() || !kapsoPhoneNumberId.trim()}
                        sx={{ fontWeight: 600, px: 2.5, textTransform: "none", borderRadius: "8px" }}
                      >
                        {testingWhatsapp ? "Enviando mensaje de prueba..." : "Enviar Mensaje de Prueba"}
                      </Button>
                    </Box>
                  </CardContent>
                </Card>
              </Grid>

              {/* Panel Lateral: Guía Rápida Kapso */}
              <Grid size={{ xs: 12, lg: 4 }}>
                <Card sx={{ borderRadius: "12px", boxShadow: "0 2px 6px rgba(0,0,0,0.04)", mb: 3 }}>
                  <CardContent sx={{ p: 3 }}>
                    <Typography variant="h6" sx={{ fontWeight: 600, mb: 2 }}>
                      Estado del Canal
                    </Typography>

                    <Box sx={{ display: "flex", flexDirection: "column", gap: 2 }}>
                      <Box sx={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                        <Typography variant="body2" sx={{ color: "#64748b" }}>
                          Proveedor Activo:
                        </Typography>
                        <Chip
                          label={whatsappProvider.toUpperCase()}
                          size="small"
                          color={whatsappProvider === "kapso" ? "success" : "default"}
                          sx={{ fontWeight: 700 }}
                        />
                      </Box>

                      <Box sx={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                        <Typography variant="body2" sx={{ color: "#64748b" }}>
                          Destino del Director:
                        </Typography>
                        <Typography variant="body2" sx={{ fontWeight: 600, fontFamily: "monospace" }}>
                          +{directorWhatsappPhone}
                        </Typography>
                      </Box>

                      <Box sx={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                        <Typography variant="body2" sx={{ color: "#64748b" }}>
                          Infraestructura:
                        </Typography>
                        <Chip label="Meta Cloud API v24.0" size="small" variant="outlined" color="primary" sx={{ fontWeight: 600 }} />
                      </Box>
                    </Box>
                  </CardContent>
                </Card>

                <Card sx={{ borderRadius: "12px", boxShadow: "0 2px 6px rgba(0,0,0,0.04)", backgroundColor: "#f0fdf4", border: "1px solid #bbf7d0" }}>
                  <CardContent sx={{ p: 3 }}>
                    <Typography variant="subtitle2" sx={{ fontWeight: 700, color: "#166534", mb: 1, display: "flex", alignItems: "center", gap: 0.5 }}>
                      🚀 Cómo obtener tus credenciales de Kapso:
                    </Typography>

                    <Typography variant="body2" sx={{ color: "#14532d", mb: 1.5, lineHeight: 1.5 }}>
                      1. Entra a <strong><a href="https://dashboard.kapso.ai" target="_blank" rel="noopener noreferrer" style={{ color: "#15803d" }}>dashboard.kapso.ai <LaunchIcon sx={{ fontSize: 13 }} /></a></strong> y crea tu cuenta gratuita.
                    </Typography>

                    <Typography variant="body2" sx={{ color: "#14532d", mb: 1.5, lineHeight: 1.5 }}>
                      2. Ve a <strong>Integrations → API keys</strong> y copia tu clave.
                    </Typography>

                    <Typography variant="body2" sx={{ color: "#14532d", mb: 1.5, lineHeight: 1.5 }}>
                      3. Ve a <strong>WhatsApp → Phone numbers</strong> y copia el <strong>Phone Number ID</strong> (puedes usar el número de prueba gratuito o tu propio número).
                    </Typography>

                    <Typography variant="body2" sx={{ color: "#14532d", lineHeight: 1.5 }}>
                      4. Pégalos aquí y haz clic en <strong>"Enviar Mensaje de Prueba"</strong>.
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
