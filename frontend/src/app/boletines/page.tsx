"use client";

import React, { useState, useEffect, useRef } from "react";
import {
  Box,
  Card,
  CardContent,
  Typography,
  Grid,
  Button,
  TextField,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  Paper,
  Chip,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  CircularProgress,
  LinearProgress,
  Alert,
  IconButton,
  Tooltip,
  Divider,
  ButtonGroup,
  Accordion,
  AccordionSummary,
  AccordionDetails,
} from "@mui/material";
import CloudUploadIcon from "@mui/icons-material/CloudUpload";
import RefreshIcon from "@mui/icons-material/Refresh";
import DescriptionIcon from "@mui/icons-material/Description";
import VisibilityIcon from "@mui/icons-material/Visibility";
import EditNoteIcon from "@mui/icons-material/EditNote";
import CheckCircleIcon from "@mui/icons-material/CheckCircle";
import SaveIcon from "@mui/icons-material/Save";
import SendIcon from "@mui/icons-material/Send";
import FormatBoldIcon from "@mui/icons-material/FormatBold";
import FormatItalicIcon from "@mui/icons-material/FormatItalic";
import FormatStrikethroughIcon from "@mui/icons-material/FormatStrikethrough";
import FormatListBulletedIcon from "@mui/icons-material/FormatListBulleted";
import FormatListNumberedIcon from "@mui/icons-material/FormatListNumbered";
import TitleIcon from "@mui/icons-material/Title";
import CodeIcon from "@mui/icons-material/Code";
import ExpandMoreIcon from "@mui/icons-material/ExpandMore";
import DoneAllIcon from "@mui/icons-material/DoneAll";
import WhatsAppIcon from "@mui/icons-material/WhatsApp";
import PlayArrowIcon from "@mui/icons-material/PlayArrow";
import AccountBalanceIcon from "@mui/icons-material/AccountBalance";
import HourglassEmptyIcon from "@mui/icons-material/HourglassEmpty";
import { ApiService } from "@/lib/api";

export default function BoletinesPage() {
  const [boletines, setBoletines] = useState<any[]>([]);
  const [loading, setLoading] = useState<boolean>(true);
  const [uploading, setUploading] = useState<boolean>(false);
  const [selectedFile, setSelectedFile] = useState<File | null>(null);
  const [fechaBoletin, setFechaBoletin] = useState<string>(
    new Date().toISOString().split("T")[0]
  );
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  // Modal de revisión, edición estilo Word y aprobación
  const [selectedBoletin, setSelectedBoletin] = useState<any | null>(null);
  const [editorModalOpen, setEditorModalOpen] = useState<boolean>(false);
  const [loadingDetail, setLoadingDetail] = useState<boolean>(false);
  const [editedText, setEditedText] = useState<string>("");
  const [savingDraft, setSavingDraft] = useState<boolean>(false);
  const [approving, setApproving] = useState<boolean>(false);

  // Estado y polling para Scraper automatizado del Senado
  const [scrapingModalOpen, setScrapingModalOpen] = useState<boolean>(false);
  const [scrapingStatus, setScrapingStatus] = useState<any>(null);
  const [startingScrape, setStartingScrape] = useState<boolean>(false);
  const scraperPollingRef = useRef<NodeJS.Timeout | null>(null);

  const textareaRef = useRef<HTMLTextAreaElement | null>(null);

  const fetchBoletines = async () => {
    try {
      setLoading(true);
      setError(null);
      const data = await ApiService.getBoletines(50, 0);
      setBoletines(data);
    } catch (err: any) {
      setError(err.message || "Error al conectar con la API de ExposureIQ");
    } finally {
      setLoading(false);
    }
  };

  const checkScraperStatus = async () => {
    try {
      const status = await ApiService.getScraperSenadoStatus();
      setScrapingStatus(status);
      return status;
    } catch (err) {
      console.error("Error al consultar estado del scraper:", err);
      return null;
    }
  };

  const startScraperPolling = () => {
    if (scraperPollingRef.current) {
      clearInterval(scraperPollingRef.current);
    }
    scraperPollingRef.current = setInterval(async () => {
      const st = await checkScraperStatus();
      if (st && !st.en_progreso) {
        if (scraperPollingRef.current) {
          clearInterval(scraperPollingRef.current);
          scraperPollingRef.current = null;
        }
        fetchBoletines();
      }
    }, 2500);
  };

  const handleTriggerScraper = async () => {
    try {
      setStartingScrape(true);
      setError(null);
      await ApiService.ejecutarScraperSenado(fechaBoletin);
      setScrapingModalOpen(true);
      await checkScraperStatus();
      startScraperPolling();
    } catch (err: any) {
      setError(err.message || "Error al iniciar sincronización del Senado.");
    } finally {
      setStartingScrape(false);
    }
  };

  useEffect(() => {
    fetchBoletines();
    checkScraperStatus().then((st) => {
      if (st && st.en_progreso) {
        startScraperPolling();
      }
    });
    return () => {
      if (scraperPollingRef.current) {
        clearInterval(scraperPollingRef.current);
      }
    };
  }, []);

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (e.target.files && e.target.files[0]) {
      const file = e.target.files[0];
      if (!file.name.toLowerCase().endsWith(".pdf")) {
        setError("El archivo debe ser formato PDF");
        setSelectedFile(null);
        return;
      }
      setSelectedFile(file);
      setError(null);
    }
  };

  const handleUpload = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!selectedFile) {
      setError("Por favor selecciona un archivo PDF del boletín");
      return;
    }

    try {
      setUploading(true);
      setError(null);
      setSuccess(null);

      const formData = new FormData();
      formData.append("file", selectedFile);
      formData.append("fecha", fechaBoletin);

      await ApiService.uploadBoletin(formData);
      setSuccess("Boletín cargado exitosamente. Se ha puesto en cola para OCR y síntesis.");
      setSelectedFile(null);
      fetchBoletines();
    } catch (err: any) {
      setError(err.message || "Error al subir el boletín");
    } finally {
      setUploading(false);
    }
  };

  const handleOpenEditor = async (id: string) => {
    try {
      setLoadingDetail(true);
      setEditorModalOpen(true);
      const detail = await ApiService.getBoletinDetail(id);
      setSelectedBoletin(detail);
      setEditedText(detail.sintesis?.texto || "");
    } catch (err: any) {
      setError("No se pudo cargar el detalle del boletín");
    } finally {
      setLoadingDetail(false);
    }
  };

  const handleProcesarBoletin = async (id: string) => {
    try {
      setLoading(true);
      setError(null);
      await ApiService.procesarBoletin(id);
      setSuccess("Iniciando procesamiento OCR y síntesis con Claude. Espera unos segundos...");
      setTimeout(() => {
        fetchBoletines();
      }, 3000);
    } catch (err: any) {
      setError(err.message || "Error al solicitar procesamiento.");
    } finally {
      setLoading(false);
    }
  };

  // Acciones de formato estilo Word sobre el texto seleccionado
  const applyFormat = (prefix: string, suffix: string = prefix, defaultPlaceholder: string = "texto") => {
    const textarea = textareaRef.current;
    if (!textarea) {
      setEditedText((prev) => prev + `${prefix}${defaultPlaceholder}${suffix}`);
      return;
    }

    const start = textarea.selectionStart;
    const end = textarea.selectionEnd;
    const currentText = editedText;

    let selected = currentText.substring(start, end);
    if (!selected) selected = defaultPlaceholder;

    const replacement = `${prefix}${selected}${suffix}`;
    const newText = currentText.substring(0, start) + replacement + currentText.substring(end);

    setEditedText(newText);

    setTimeout(() => {
      textarea.focus();
      textarea.setSelectionRange(start + prefix.length, start + prefix.length + selected.length);
    }, 50);
  };

  const insertQuickBadge = (badge: string) => {
    const textarea = textareaRef.current;
    if (!textarea) {
      setEditedText((prev) => prev + "\n" + badge + " ");
      return;
    }
    const start = textarea.selectionStart;
    const end = textarea.selectionEnd;
    const newText = editedText.substring(0, start) + badge + " " + editedText.substring(end);
    setEditedText(newText);
    setTimeout(() => {
      textarea.focus();
      textarea.setSelectionRange(start + badge.length + 1, start + badge.length + 1);
    }, 50);
  };

  const handleSaveDraft = async () => {
    if (!selectedBoletin) return;
    try {
      setSavingDraft(true);
      await ApiService.actualizarSintesis(selectedBoletin.boletin.id, editedText);
      setSuccess("Borrador de síntesis guardado correctamente.");
    } catch (err: any) {
      setError(err.message || "Error al guardar el borrador.");
    } finally {
      setSavingDraft(false);
    }
  };

  const handleApprove = async () => {
    if (!selectedBoletin) return;
    try {
      setApproving(true);
      await ApiService.aprobarBoletin(selectedBoletin.boletin.id, editedText);
      setSuccess("✅ Boletín aprobado con éxito. El brief se envió vía WhatsApp a la lista de distribución.");
      setEditorModalOpen(false);
      fetchBoletines();
    } catch (err: any) {
      setError(err.message || "Error al aprobar y encolar el boletín.");
    } finally {
      setApproving(false);
    }
  };

  // Convertidor sencillo de formato WhatsApp (*negrita*, _cursiva_) para previsualización HTML
  const formatWhatsAppText = (text: string) => {
    if (!text) return "No hay contenido redactado todavía.";

    const escaped = text
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;");

    const formatted = escaped
      .replace(/\*([^*\n]+)\*/g, "<strong>$1</strong>")
      .replace(/_([^_\n]+)_/g, "<em>$1</em>")
      .replace(/~([^~\n]+)~/g, "<del>$1</del>")
      .replace(/```([^`]+)```/g, "<pre style='background:#f1f5f9;padding:4px;border-radius:4px;font-family:monospace'>$1</pre>")
      .replace(/`([^`\n]+)`/g, "<code style='background:#f1f5f9;padding:2px 4px;border-radius:3px;font-family:monospace'>$1</code>")
      .replace(/\n/g, "<br />");

    return formatted;
  };

  const getStatusChip = (estado: string) => {
    switch (estado) {
      case "aprobado":
        return <Chip label="Aprobado (En Cola WhatsApp)" color="success" size="small" sx={{ fontWeight: 600 }} />;
      case "sintesis_lista":
        return <Chip label="Pendiente de Visto Bueno" color="warning" size="small" sx={{ fontWeight: 600 }} />;
      case "ocr_completo":
        return <Chip label="OCR Completo" color="info" size="small" />;
      case "en_ocr":
        return <Chip label="En OCR..." color="warning" size="small" />;
      case "pendiente_ocr":
        return <Chip label="En Cola OCR" color="default" size="small" />;
      default:
        return <Chip label={estado} color="error" size="small" />;
    }
  };

  return (
    <Box sx={{ p: 3 }}>
      {/* Encabezado */}
      <Box sx={{ display: "flex", justifyContent: "space-between", alignItems: "center", mb: 3, flexWrap: "wrap", gap: 2 }}>
        <Box>
          <Typography variant="h5" sx={{ fontWeight: 700, color: "#1e293b" }}>
            Boletines & Síntesis Informativa
          </Typography>
          <Typography variant="body2" sx={{ color: "#64748b" }}>
            Ingesta automatizada del Senado de la República (Headless Browser) y carga manual de boletines, con OCR Surya y visto bueno para WhatsApp.
          </Typography>
        </Box>
        <Box sx={{ display: "flex", gap: 1.5, alignItems: "center" }}>
          <Button
            variant="contained"
            startIcon={
              startingScrape || scrapingStatus?.en_progreso ? (
                <CircularProgress size={18} color="inherit" />
              ) : (
                <AccountBalanceIcon />
              )
            }
            onClick={() => {
              if (scrapingStatus?.en_progreso) {
                setScrapingModalOpen(true);
              } else {
                handleTriggerScraper();
              }
            }}
            disabled={startingScrape}
            sx={{
              backgroundColor: scrapingStatus?.en_progreso ? "#0284c7" : "#0f172a",
              color: "#ffffff",
              fontWeight: 600,
              textTransform: "none",
              borderRadius: "8px",
              boxShadow: "0 2px 6px rgba(0,0,0,0.12)",
              "&:hover": {
                backgroundColor: scrapingStatus?.en_progreso ? "#0369a1" : "#1e293b",
              },
            }}
          >
            {scrapingStatus?.en_progreso
              ? "Sincronizando Senado..."
              : "Sincronizar Síntesis del Senado"}
          </Button>
          <Button
            variant="outlined"
            startIcon={<RefreshIcon />}
            onClick={fetchBoletines}
            disabled={loading}
          >
            Actualizar
          </Button>
        </Box>
      </Box>

      {/* Banner de progreso en vivo del Scraper del Senado */}
      {scrapingStatus?.en_progreso && (
        <Alert
          severity="info"
          icon={<CircularProgress size={20} color="inherit" />}
          action={
            <Button
              color="inherit"
              size="small"
              variant="outlined"
              onClick={() => setScrapingModalOpen(true)}
              sx={{ fontWeight: 600, textTransform: "none" }}
            >
              Ver Detalle
            </Button>
          }
          sx={{ mb: 3, borderRadius: "10px", alignItems: "center" }}
        >
          <strong>Sincronización desatendida del Senado en curso:</strong>{" "}
          {scrapingStatus.mensaje || "Navegando y extrayendo PDFs..."}{" "}
          ({scrapingStatus.archivos_descargados?.length || 0} descargados,{" "}
          {scrapingStatus.archivos_procesados?.length || 0}/10 sintetizados)
        </Alert>
      )}

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
        {/* Formulario de Carga */}
        <Grid size={{ xs: 12, md: 4 }}>
          <Card sx={{ borderRadius: "12px", boxShadow: "0 2px 6px rgba(0,0,0,0.04)" }}>
            <CardContent>
              <Typography variant="h6" sx={{ fontWeight: 600, mb: 2 }}>
                Cargar Boletín Coparmex
              </Typography>

              <form onSubmit={handleUpload}>
                <Box sx={{ mb: 2 }}>
                  <Typography variant="caption" sx={{ color: "#64748b", fontWeight: 600 }}>
                    FECHA DEL BOLETÍN
                  </Typography>
                  <TextField
                    type="date"
                    fullWidth
                    size="small"
                    value={fechaBoletin}
                    onChange={(e) => setFechaBoletin(e.target.value)}
                    sx={{ mt: 0.5 }}
                  />
                </Box>

                <Box
                  component="label"
                  htmlFor="boletin-file-upload"
                  sx={{
                    display: "flex",
                    flexDirection: "column",
                    alignItems: "center",
                    justifyContent: "center",
                    border: "2px dashed #cbd5e1",
                    borderRadius: "10px",
                    p: 3,
                    minHeight: "150px",
                    textAlign: "center",
                    backgroundColor: "#f8fafc",
                    cursor: "pointer",
                    mb: 3,
                    width: "100%",
                    boxSizing: "border-box",
                    transition: "all 0.2s ease-in-out",
                    "&:hover": {
                      borderColor: "#6366f1",
                      backgroundColor: "#f1f5f9",
                    },
                  }}
                >
                  <input
                    id="boletin-file-upload"
                    type="file"
                    accept=".pdf,application/pdf"
                    style={{ display: "none" }}
                    onChange={handleFileChange}
                  />
                  <CloudUploadIcon sx={{ fontSize: 44, color: "#6366f1", mb: 1 }} />
                  <Typography
                    variant="body2"
                    sx={{
                      fontWeight: 600,
                      color: "#1e293b",
                      mb: 0.5,
                      wordBreak: "break-word",
                    }}
                  >
                    {selectedFile ? selectedFile.name : "Haz clic o arrastra el archivo PDF aquí"}
                  </Typography>
                  <Typography
                    variant="caption"
                    sx={{ color: "#64748b", display: "block" }}
                  >
                    {selectedFile
                      ? `Tamaño: ${(selectedFile.size / (1024 * 1024)).toFixed(2)} MB`
                      : "Boletín diario (formato PDF de Coparmex)"}
                  </Typography>
                </Box>

                <Button
                  type="submit"
                  variant="contained"
                  fullWidth
                  disabled={uploading || !selectedFile}
                  sx={{
                    py: 1.3,
                    fontWeight: 600,
                    textTransform: "none",
                    borderRadius: "8px",
                  }}
                >
                  {uploading ? (
                    <Box sx={{ display: "flex", alignItems: "center", gap: 1 }}>
                      <CircularProgress size={20} color="inherit" />
                      <span>Procesando archivo...</span>
                    </Box>
                  ) : (
                    "Iniciar Procesamiento OCR"
                  )}
                </Button>
              </form>
            </CardContent>
          </Card>
        </Grid>

        {/* Tabla de Historial */}
        <Grid size={{ xs: 12, md: 8 }}>
          <Card sx={{ borderRadius: "12px", boxShadow: "0 2px 6px rgba(0,0,0,0.04)" }}>
            <CardContent>
              <Typography variant="h6" sx={{ fontWeight: 600, mb: 2 }}>
                Historial de Boletines Procesados
              </Typography>

              {loading ? (
                <Box sx={{ display: "flex", justifyContent: "center", p: 4 }}>
                  <CircularProgress />
                </Box>
              ) : boletines.length === 0 ? (
                <Box sx={{ textAlign: "center", p: 4, color: "#94a3b8" }}>
                  <DescriptionIcon sx={{ fontSize: 48, mb: 1 }} />
                  <Typography>No hay boletines cargados aún. Sube el primer PDF diario.</Typography>
                </Box>
              ) : (
                <TableContainer component={Paper} elevation={0}>
                  <Table size="small">
                    <TableHead sx={{ backgroundColor: "#f8fafc" }}>
                      <TableRow>
                        <TableCell sx={{ fontWeight: 600 }}>Fecha</TableCell>
                        <TableCell sx={{ fontWeight: 600 }}>Archivo</TableCell>
                        <TableCell sx={{ fontWeight: 600 }}>Estado</TableCell>
                        <TableCell sx={{ fontWeight: 600 }}>Páginas</TableCell>
                        <TableCell sx={{ fontWeight: 600, textAlign: "right" }}>Acción</TableCell>
                      </TableRow>
                    </TableHead>
                    <TableBody>
                      {boletines.map((b) => (
                        <TableRow key={b.id} hover>
                          <TableCell sx={{ fontWeight: 500 }}>{b.fecha_boletin}</TableCell>
                          <TableCell sx={{ maxWidth: 200, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                            {b.nombre_archivo}
                          </TableCell>
                          <TableCell>{getStatusChip(b.estado)}</TableCell>
                          <TableCell>{b.total_paginas || "-"}</TableCell>
                          <TableCell sx={{ textAlign: "right" }}>
                            {b.estado === "sintesis_lista" ? (
                              <Box sx={{ display: "flex", gap: 0.5, alignItems: "center" }}>
                                <Button
                                  size="small"
                                  variant="contained"
                                  color="primary"
                                  startIcon={<EditNoteIcon />}
                                  onClick={() => handleOpenEditor(b.id)}
                                  sx={{ fontWeight: 600, textTransform: "none" }}
                                >
                                  Revisar & Aprobar
                                </Button>
                                <Tooltip title="Reprocesar con IA / Claude">
                                  <IconButton
                                    size="small"
                                    color="warning"
                                    onClick={() => handleProcesarBoletin(b.id)}
                                    sx={{ border: "1px solid #fed7aa", borderRadius: "8px" }}
                                  >
                                    <PlayArrowIcon fontSize="small" />
                                  </IconButton>
                                </Tooltip>
                              </Box>
                            ) : b.estado === "pendiente_ocr" || b.estado === "error_sintesis" ? (
                              <Button
                                size="small"
                                variant="contained"
                                color="warning"
                                startIcon={<PlayArrowIcon />}
                                onClick={() => handleProcesarBoletin(b.id)}
                                sx={{ fontWeight: 600, textTransform: "none" }}
                              >
                                Procesar Ahora
                              </Button>
                            ) : (
                              <Button
                                size="small"
                                variant="outlined"
                                startIcon={<VisibilityIcon />}
                                onClick={() => handleOpenEditor(b.id)}
                                sx={{ textTransform: "none" }}
                              >
                                Ver Brief
                              </Button>
                            )}
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

      {/* ===================================================================== */}
      {/* MODAL / INTERFAZ DE EDICIÓN ESTILO WORD & SIMULADOR DE WHATSAPP      */}
      {/* ===================================================================== */}
      <Dialog
        open={editorModalOpen}
        onClose={() => setEditorModalOpen(false)}
        maxWidth="xl"
        fullWidth
        sx={{
          "& .MuiDialog-paper": {
            borderRadius: "14px",
            minHeight: "85vh",
          },
        }}
      >
        <DialogTitle
          sx={{
            fontWeight: 700,
            borderBottom: "1px solid #e2e8f0",
            display: "flex",
            justifyContent: "space-between",
            alignItems: "center",
            py: 2,
            px: 3,
          }}
        >
          <Box sx={{ display: "flex", alignItems: "center", gap: 1.5 }}>
            <Typography variant="h6" sx={{ fontWeight: 700, color: "#1e293b" }}>
              Revisión y Aprobación de Síntesis Ejecutiva
            </Typography>
            {selectedBoletin && getStatusChip(selectedBoletin.boletin.estado)}
          </Box>
          <Typography variant="caption" sx={{ color: "#64748b" }}>
            Boletín: {selectedBoletin?.boletin.fecha_boletin} ({selectedBoletin?.boletin.nombre_archivo})
          </Typography>
        </DialogTitle>

        <DialogContent sx={{ p: 3, backgroundColor: "#f8fafc" }}>
          {loadingDetail ? (
            <Box sx={{ display: "flex", justifyContent: "center", alignItems: "center", minHeight: "400px" }}>
              <CircularProgress />
            </Box>
          ) : selectedBoletin ? (
            <Grid container spacing={3}>
              {/* PANEL IZQUIERDO: EDITOR ESTILO WORD */}
              <Grid size={{ xs: 12, md: 7 }}>
                <Card sx={{ borderRadius: "10px", boxShadow: "0 2px 10px rgba(0,0,0,0.06)", border: "1px solid #e2e8f0" }}>
                  {/* BARRA DE HERRAMIENTAS ESTILO WORD (RIBBON) */}
                  <Box
                    sx={{
                      p: 1.2,
                      backgroundColor: "#f1f5f9",
                      borderBottom: "1px solid #cbd5e1",
                      display: "flex",
                      flexWrap: "wrap",
                      alignItems: "center",
                      gap: 1,
                    }}
                  >
                    {/* Formato de Texto */}
                    <ButtonGroup size="small" variant="outlined" sx={{ backgroundColor: "#fff" }}>
                      <Tooltip title="Negrita (*texto*)">
                        <IconButton size="small" onClick={() => applyFormat("*", "*", "texto en negrita")}>
                          <FormatBoldIcon fontSize="small" />
                        </IconButton>
                      </Tooltip>
                      <Tooltip title="Cursiva (_texto_)">
                        <IconButton size="small" onClick={() => applyFormat("_", "_", "texto en cursiva")}>
                          <FormatItalicIcon fontSize="small" />
                        </IconButton>
                      </Tooltip>
                      <Tooltip title="Tachado (~texto~)">
                        <IconButton size="small" onClick={() => applyFormat("~", "~", "texto tachado")}>
                          <FormatStrikethroughIcon fontSize="small" />
                        </IconButton>
                      </Tooltip>
                      <Tooltip title="Monoespaciado / Código">
                        <IconButton size="small" onClick={() => applyFormat("```\n", "\n```", "código / dato exacto")}>
                          <CodeIcon fontSize="small" />
                        </IconButton>
                      </Tooltip>
                    </ButtonGroup>

                    <Divider orientation="vertical" flexItem sx={{ mx: 0.5 }} />

                    {/* Estructura de Encabezados */}
                    <ButtonGroup size="small" variant="outlined" sx={{ backgroundColor: "#fff" }}>
                      <Tooltip title="Encabezado Principal">
                        <IconButton size="small" onClick={() => applyFormat("\n📌 *", "*\n", "TÍTULO PRINCIPAL")}>
                          <TitleIcon fontSize="small" />
                        </IconButton>
                      </Tooltip>
                      <Tooltip title="Lista con viñetas">
                        <IconButton size="small" onClick={() => applyFormat("\n• ", "", "Elemento clave de la lista")}>
                          <FormatListBulletedIcon fontSize="small" />
                        </IconButton>
                      </Tooltip>
                      <Tooltip title="Lista numerada">
                        <IconButton size="small" onClick={() => applyFormat("\n1. ", "", "Primer punto relevante")}>
                          <FormatListNumberedIcon fontSize="small" />
                        </IconButton>
                      </Tooltip>
                    </ButtonGroup>

                    <Divider orientation="vertical" flexItem sx={{ mx: 0.5 }} />

                    {/* Atajos Ejecutivos de un clic */}
                    <Box sx={{ display: "flex", gap: 0.5, flexWrap: "wrap" }}>
                      <Chip
                        label="🚨 Crítico"
                        size="small"
                        color="error"
                        clickable
                        onClick={() => insertQuickBadge("🚨 *ALTO IMPACTO:*")}
                      />
                      <Chip
                        label="📌 Relevante"
                        size="small"
                        color="primary"
                        clickable
                        onClick={() => insertQuickBadge("📌 *RELEVANTE:*")}
                      />
                      <Chip
                        label="⚠️ Alerta"
                        size="small"
                        color="warning"
                        clickable
                        onClick={() => insertQuickBadge("⚠️ *ADVERTENCIA:*")}
                      />
                      <Chip
                        label="📊 Cifras"
                        size="small"
                        color="info"
                        clickable
                        onClick={() => insertQuickBadge("📊 *ESTADÍSTICA:*")}
                      />
                      <Chip
                        label="💼 Seguros"
                        size="small"
                        variant="outlined"
                        clickable
                        onClick={() => insertQuickBadge("💼 *SECTOR ASEGURADOR:*")}
                      />
                      <Chip
                        label="✅ Acción"
                        size="small"
                        color="success"
                        clickable
                        onClick={() => insertQuickBadge("✅ *ACCIÓN SUGERIDA:*")}
                      />
                    </Box>
                  </Box>

                  {/* HOJA DE EDICIÓN ESTILO DOCUMENTO */}
                  <Box sx={{ p: 2.5, backgroundColor: "#fff", minHeight: "360px" }}>
                    <Typography variant="caption" sx={{ color: "#94a3b8", display: "block", mb: 1, fontWeight: 600 }}>
                      DOCUMENTO DE REDACCIÓN EJECUTIVA (Compatible con formato WhatsApp):
                    </Typography>
                    <TextField
                      inputRef={textareaRef}
                      multiline
                      rows={14}
                      fullWidth
                      value={editedText}
                      onChange={(e) => setEditedText(e.target.value)}
                      placeholder="Escribe o edita aquí el brief ejecutivo que será enviado..."
                      variant="outlined"
                      sx={{
                        "& .MuiOutlinedInput-root": {
                          fontFamily: "inherit",
                          fontSize: "0.95rem",
                          lineHeight: 1.6,
                          backgroundColor: "#fafafa",
                        },
                      }}
                    />
                  </Box>
                </Card>

                {/* Secciones de Referencia del PDF extraídas con OCR */}
                <Accordion sx={{ mt: 2, borderRadius: "8px !important", border: "1px solid #e2e8f0" }} elevation={0}>
                  <AccordionSummary expandIcon={<ExpandMoreIcon />}>
                    <Typography variant="subtitle2" sx={{ fontWeight: 600, color: "#475569" }}>
                      📄 Consultar Secciones Temáticas Originales ({selectedBoletin.secciones?.length || 0})
                    </Typography>
                  </AccordionSummary>
                  <AccordionDetails sx={{ maxHeight: "250px", overflowY: "auto", p: 2 }}>
                    {selectedBoletin.secciones?.map((s: any) => (
                      <Box key={s.id} sx={{ mb: 1.5, p: 1.5, backgroundColor: "#f8fafc", borderRadius: "6px" }}>
                        <Typography variant="subtitle2" sx={{ fontWeight: 600, color: "#1e293b" }}>
                          #{s.orden} — {s.tema?.toUpperCase()} (Páginas {s.pagina_inicio} a {s.pagina_fin})
                        </Typography>
                        <Typography variant="caption" sx={{ color: "#64748b", display: "block", mt: 0.5 }}>
                          {s.contenido?.texto_completo?.slice(0, 300)}...
                        </Typography>
                      </Box>
                    ))}
                  </AccordionDetails>
                </Accordion>
              </Grid>

              {/* PANEL DERECHO: SIMULADOR DE WHATSAPP EN VIVO */}
              <Grid size={{ xs: 12, md: 5 }}>
                <Card
                  sx={{
                    borderRadius: "14px",
                    boxShadow: "0 4px 15px rgba(0,0,0,0.1)",
                    border: "1px solid #cbd5e1",
                    overflow: "hidden",
                    backgroundColor: "#e5ddd5",
                  }}
                >
                  {/* Header de WhatsApp */}
                  <Box
                    sx={{
                      backgroundColor: "#075e54",
                      color: "#fff",
                      p: 1.5,
                      display: "flex",
                      alignItems: "center",
                      gap: 1.5,
                    }}
                  >
                    <WhatsAppIcon sx={{ fontSize: 28, color: "#25d366" }} />
                    <Box>
                      <Typography variant="subtitle2" sx={{ fontWeight: 700, lineHeight: 1.2 }}>
                        Director de Operaciones
                      </Typography>
                      <Typography variant="caption" sx={{ color: "#dcf8c6" }}>
                        ExposureIQ • Entrega Oficial
                      </Typography>
                    </Box>
                  </Box>

                  {/* Área del Chat */}
                  <Box
                    sx={{
                      p: 2.5,
                      minHeight: "440px",
                      maxHeight: "560px",
                      overflowY: "auto",
                      backgroundColor: "#efeae2",
                      backgroundImage: "radial-gradient(#d4ccc3 1px, transparent 1px)",
                      backgroundSize: "20px 20px",
                    }}
                  >
                    {/* Burbuja Verde de WhatsApp */}
                    <Box
                      sx={{
                        backgroundColor: "#d9fdd3",
                        p: 2,
                        borderRadius: "8px 8px 0px 8px",
                        boxShadow: "0 1px 2px rgba(0,0,0,0.15)",
                        maxWidth: "100%",
                        position: "relative",
                        ml: "auto",
                      }}
                    >
                      <Typography
                        component="div"
                        dangerouslySetInnerHTML={{ __html: formatWhatsAppText(editedText) }}
                        sx={{
                          fontSize: "0.88rem",
                          lineHeight: 1.5,
                          color: "#111b21",
                          wordBreak: "break-word",
                        }}
                      />
                      <Box
                        sx={{
                          display: "flex",
                          justifyContent: "flex-end",
                          alignItems: "center",
                          gap: 0.5,
                          mt: 1,
                        }}
                      >
                        <Typography variant="caption" sx={{ fontSize: "0.7rem", color: "#667781" }}>
                          {new Date().toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })}
                        </Typography>
                        <DoneAllIcon sx={{ fontSize: 16, color: "#53bdeb" }} />
                      </Box>
                    </Box>
                  </Box>

                  {/* Footer informativo del simulador */}
                  <Box sx={{ p: 1.5, backgroundColor: "#f0f2f5", borderTop: "1px solid #d1d7db", textAlign: "center" }}>
                    <Typography variant="caption" sx={{ color: "#54656f", fontWeight: 500 }}>
                      📱 Vista idéntica a la pantalla del smartphone del Director.
                    </Typography>
                  </Box>
                </Card>
              </Grid>
            </Grid>
          ) : null}
        </DialogContent>

        <DialogActions sx={{ borderTop: "1px solid #e2e8f0", p: 2.5, px: 3, justifyContent: "space-between" }}>
          <Button onClick={() => setEditorModalOpen(false)} color="inherit">
            Cerrar
          </Button>

          <Box sx={{ display: "flex", gap: 1.5 }}>
            <Button
              variant="outlined"
              color="warning"
              startIcon={<PlayArrowIcon />}
              onClick={() => {
                if (selectedBoletin) {
                  handleProcesarBoletin(selectedBoletin.boletin.id);
                  setEditorModalOpen(false);
                }
              }}
              disabled={savingDraft || approving || loadingDetail}
            >
              Reprocesar con Claude
            </Button>

            <Button
              variant="outlined"
              startIcon={<SaveIcon />}
              onClick={handleSaveDraft}
              disabled={savingDraft || approving || loadingDetail}
            >
              {savingDraft ? "Guardando..." : "Guardar Borrador"}
            </Button>

            <Button
              variant="contained"
              color="success"
              startIcon={<SendIcon />}
              onClick={handleApprove}
              disabled={approving || savingDraft || loadingDetail || !editedText.trim()}
              sx={{
                fontWeight: 700,
                px: 3,
                backgroundColor: "#16a34a",
                "&:hover": { backgroundColor: "#15803d" },
              }}
            >
              {approving ? (
                <Box sx={{ display: "flex", alignItems: "center", gap: 1 }}>
                  <CircularProgress size={18} color="inherit" />
                  <span>Aprobando...</span>
                </Box>
              ) : (
                "Aprobar y Enviar vía WhatsApp"
              )}
            </Button>
          </Box>
        </DialogActions>
      </Dialog>

      {/* ===================================================================== */}
      {/* MODAL DE PROGRESO Y CONTROL: SCRAPER SÍNTESIS SENADO DE LA REPÚBLICA */}
      {/* ===================================================================== */}
      <Dialog
        open={scrapingModalOpen}
        onClose={() => {
          if (!scrapingStatus?.en_progreso) {
            setScrapingModalOpen(false);
          }
        }}
        maxWidth="md"
        fullWidth
        sx={{
          "& .MuiDialog-paper": {
            borderRadius: "14px",
            p: 1,
          },
        }}
      >
        <DialogTitle sx={{ pb: 1 }}>
          <Box sx={{ display: "flex", alignItems: "center", gap: 1.5 }}>
            <AccountBalanceIcon sx={{ color: "#0284c7", fontSize: 32 }} />
            <Box>
              <Typography variant="h6" sx={{ fontWeight: 700, color: "#0f172a" }}>
                Sincronización Automatizada — Síntesis Senado
              </Typography>
              <Typography variant="caption" sx={{ color: "#64748b" }}>
                Navegación desatendida, descarga de 10 secciones (excluye Cartones), OCR y síntesis IA.
              </Typography>
            </Box>
          </Box>
        </DialogTitle>

        <DialogContent dividers sx={{ py: 2.5 }}>
          {scrapingStatus?.en_progreso ? (
            <Box sx={{ mb: 3 }}>
              <Box sx={{ display: "flex", justifyContent: "space-between", alignItems: "center", mb: 1 }}>
                <Typography variant="body2" sx={{ fontWeight: 600, color: "#0284c7" }}>
                  {scrapingStatus.mensaje || "Ejecutando proceso de extracción y síntesis..."}
                </Typography>
                <CircularProgress size={18} />
              </Box>
              <LinearProgress
                color="info"
                sx={{ height: 8, borderRadius: 4, backgroundColor: "#e2e8f0" }}
              />
            </Box>
          ) : scrapingStatus?.archivos_procesados?.length > 0 ? (
            <Alert severity="success" sx={{ mb: 2.5, borderRadius: "8px" }}>
              <strong>¡Sincronización y síntesis completada!</strong> Se descargaron y procesaron{" "}
              {scrapingStatus.archivos_procesados.length} secciones del Senado. Todas se encuentran en estado{" "}
              <strong>"Pendiente de Visto Bueno"</strong> (Opción A), listas para su revisión y aprobación.
            </Alert>
          ) : scrapingStatus?.errores?.length > 0 ? (
            <Alert severity="error" sx={{ mb: 2.5, borderRadius: "8px" }}>
              Hubo observaciones durante la sincronización: {scrapingStatus.errores.join(", ")}
            </Alert>
          ) : (
            <Alert severity="info" sx={{ mb: 2.5, borderRadius: "8px" }}>
              {scrapingStatus?.mensaje || "Scraper listo para sincronizar."}
            </Alert>
          )}

          <Grid container spacing={2} sx={{ mb: 2 }}>
            <Grid size={{ xs: 12, sm: 4 }}>
              <Paper variant="outlined" sx={{ p: 1.5, borderRadius: "8px", textAlign: "center" }}>
                <Typography variant="caption" sx={{ color: "#64748b", fontWeight: 600 }}>
                  FECHA CONSULTADA
                </Typography>
                <Typography variant="body1" sx={{ fontWeight: 700, color: "#1e293b", mt: 0.5 }}>
                  {scrapingStatus?.fecha_objetivo || fechaBoletin || "Hoy"}
                </Typography>
              </Paper>
            </Grid>
            <Grid size={{ xs: 12, sm: 4 }}>
              <Paper variant="outlined" sx={{ p: 1.5, borderRadius: "8px", textAlign: "center" }}>
                <Typography variant="caption" sx={{ color: "#64748b", fontWeight: 600 }}>
                  DESCARGADOS
                </Typography>
                <Typography variant="body1" sx={{ fontWeight: 700, color: "#0284c7", mt: 0.5 }}>
                  {scrapingStatus?.archivos_descargados?.length || 0} / 10 secciones
                </Typography>
              </Paper>
            </Grid>
            <Grid size={{ xs: 12, sm: 4 }}>
              <Paper variant="outlined" sx={{ p: 1.5, borderRadius: "8px", textAlign: "center" }}>
                <Typography variant="caption" sx={{ color: "#64748b", fontWeight: 600 }}>
                  SÍNTESIS GENERADAS
                </Typography>
                <Typography variant="body1" sx={{ fontWeight: 700, color: "#16a34a", mt: 0.5 }}>
                  {scrapingStatus?.archivos_procesados?.length || 0} / 10 listos
                </Typography>
              </Paper>
            </Grid>
          </Grid>

          {/* Lista de secciones procesadas o descargadas */}
          <Typography variant="subtitle2" sx={{ fontWeight: 700, color: "#334155", mb: 1 }}>
            Detalle de Secciones del Senado (10 solicitadas - Cartones Excluida):
          </Typography>

          <Box
            sx={{
              maxHeight: 240,
              overflowY: "auto",
              border: "1px solid #e2e8f0",
              borderRadius: "8px",
              backgroundColor: "#f8fafc",
              p: 1,
            }}
          >
            {scrapingStatus?.archivos_descargados && scrapingStatus.archivos_descargados.length > 0 ? (
              scrapingStatus.archivos_descargados.map((item: any, idx: number) => {
                const procesado = scrapingStatus.archivos_procesados?.some(
                  (p: any) => p.archivo === item.archivo
                );
                return (
                  <Box
                    key={idx}
                    sx={{
                      display: "flex",
                      alignItems: "center",
                      justifyContent: "space-between",
                      p: 1,
                      borderBottom:
                        idx < scrapingStatus.archivos_descargados.length - 1
                          ? "1px solid #e2e8f0"
                          : "none",
                    }}
                  >
                    <Box sx={{ display: "flex", alignItems: "center", gap: 1 }}>
                      {procesado ? (
                        <CheckCircleIcon sx={{ color: "#16a34a", fontSize: 20 }} />
                      ) : (
                        <HourglassEmptyIcon sx={{ color: "#f59e0b", fontSize: 20 }} />
                      )}
                      <Typography variant="body2" sx={{ fontWeight: 600, color: "#1e293b" }}>
                        {item.nombre || item.archivo}
                      </Typography>
                      <Typography variant="caption" sx={{ color: "#64748b" }}>
                        ({item.archivo} - {(item.tamano_kb || 0).toFixed(0)} KB)
                      </Typography>
                    </Box>
                    <Chip
                      size="small"
                      label={procesado ? "Síntesis Lista" : "En Procesamiento"}
                      color={procesado ? "success" : "warning"}
                      variant="outlined"
                    />
                  </Box>
                );
              })
            ) : (
              <Box sx={{ p: 2, textAlign: "center", color: "#64748b" }}>
                <Typography variant="body2">
                  {scrapingStatus?.en_progreso
                    ? "Iniciando navegación con Headless Browser..."
                    : "No hay descargas registradas en esta sesión."}
                </Typography>
              </Box>
            )}
          </Box>
        </DialogContent>

        <DialogActions sx={{ p: 2, justifyContent: "space-between" }}>
          <Button
            onClick={() => setScrapingModalOpen(false)}
            color="inherit"
          >
            {scrapingStatus?.en_progreso ? "Ocultar y Dejar en Segundo Plano" : "Cerrar"}
          </Button>

          <Button
            variant="contained"
            onClick={() => {
              setScrapingModalOpen(false);
              fetchBoletines();
            }}
            disabled={scrapingStatus?.en_progreso}
            sx={{
              backgroundColor: "#0284c7",
              "&:hover": { backgroundColor: "#0369a1" },
              fontWeight: 600,
              textTransform: "none",
            }}
          >
            Ver Boletines en Tabla
          </Button>
        </DialogActions>
      </Dialog>
    </Box>
  );
}
