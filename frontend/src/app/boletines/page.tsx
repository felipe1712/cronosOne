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
  Checkbox,
  FormControlLabel,
  Stack,
} from "@mui/material";
import CloudUploadIcon from "@mui/icons-material/CloudUpload";
import RefreshIcon from "@mui/icons-material/Refresh";
import DescriptionIcon from "@mui/icons-material/Description";
import VisibilityIcon from "@mui/icons-material/Visibility";
import CheckCircleIcon from "@mui/icons-material/CheckCircle";
import SaveIcon from "@mui/icons-material/Save";
import SendIcon from "@mui/icons-material/Send";
import FormatBoldIcon from "@mui/icons-material/FormatBold";
import FormatItalicIcon from "@mui/icons-material/FormatItalic";
import FormatStrikethroughIcon from "@mui/icons-material/FormatStrikethrough";
import FormatListBulletedIcon from "@mui/icons-material/FormatListBulleted";
import CodeIcon from "@mui/icons-material/Code";
import DoneAllIcon from "@mui/icons-material/DoneAll";
import WhatsAppIcon from "@mui/icons-material/WhatsApp";
import AccountBalanceIcon from "@mui/icons-material/AccountBalance";
import CalendarTodayIcon from "@mui/icons-material/CalendarToday";
import ArrowBackIosNewIcon from "@mui/icons-material/ArrowBackIosNew";
import ArrowForwardIosIcon from "@mui/icons-material/ArrowForwardIos";
import AutoAwesomeIcon from "@mui/icons-material/AutoAwesome";
import DeleteOutlineIcon from "@mui/icons-material/DeleteOutline";
import PictureAsPdfIcon from "@mui/icons-material/PictureAsPdf";
import FolderOpenIcon from "@mui/icons-material/FolderOpen";
import CloseIcon from "@mui/icons-material/Close";
import CheckBoxIcon from "@mui/icons-material/CheckBox";
import LayersIcon from "@mui/icons-material/Layers";
import SearchIcon from "@mui/icons-material/Search";
import InfoOutlinedIcon from "@mui/icons-material/InfoOutlined";
import AutorenewIcon from "@mui/icons-material/Autorenew";

import {
  ApiService,
  SintesisDiariaResumen,
  WorkspaceFechaResponse,
  SeccionSenado,
  DetalleSeccionSenado,
} from "@/lib/api";

const SECCIONES_SENADO_DEFAULT: SeccionSenado[] = [
  { id: "portada", nombre: "Síntesis Digital Informativa", archivo: "SINTESIS.pdf", orden: 1 },
  { id: "primeras_planas", nombre: "Primeras Planas", archivo: "PRIMERASPLANAS.pdf", orden: 2 },
  { id: "primeras_planas_int", nombre: "Primeras Planas Internacionales", archivo: "PRIMERASPLANASINTERNACIONALES.pdf", orden: 3 },
  { id: "redes", nombre: "Redes", archivo: "REDES.pdf", orden: 4 },
  { id: "senado", nombre: "Senado", archivo: "SENADO.pdf", orden: 5 },
  { id: "senadores_escriben", nombre: "Senadores Escriben", archivo: "SENADORESESCRIBEN.pdf", orden: 6 },
  { id: "columnas_senado", nombre: "Columnas Senado", archivo: "COLUMNAS_S.pdf", orden: 7 },
  { id: "diputados", nombre: "Diputados", archivo: "DIPUTADOS.pdf", orden: 8 },
  { id: "panorama_nacional", nombre: "Panorama Nacional", archivo: "PANORAMANACIONAL.pdf", orden: 9 },
  { id: "columnas", nombre: "Columnas", archivo: "COLUMNAS.pdf", orden: 10 },
];

export default function BoletinesPage() {
  // Fecha activa de trabajo (Workspace)
  const [fechaTrabajo, setFechaTrabajo] = useState<string>(
    new Date().toISOString().split("T")[0]
  );

  // Historial de Fechas de Síntesis
  const [fechasHistorial, setFechasHistorial] = useState<SintesisDiariaResumen[]>([]);
  const [loadingHistorial, setLoadingHistorial] = useState<boolean>(true);

  // Espacio de Trabajo del Día Activo
  const [workspace, setWorkspace] = useState<WorkspaceFechaResponse | null>(null);
  const [loadingWorkspace, setLoadingWorkspace] = useState<boolean>(false);

  // Carga Múltiple de Documentos
  const [stagedFiles, setStagedFiles] = useState<File[]>([]);
  const [uploadingFiles, setUploadingFiles] = useState<boolean>(false);
  const [uploadModalOpen, setUploadModalOpen] = useState<boolean>(false);
  const [uploadModalError, setUploadModalError] = useState<string | null>(null);

  // Diálogo de Selección de Secciones del Senado
  const [senadoModalOpen, setSenadoModalOpen] = useState<boolean>(false);
  const [seccionesCatalogo, setSeccionesCatalogo] = useState<SeccionSenado[]>(SECCIONES_SENADO_DEFAULT);
  const [seccionesSeleccionadas, setSeccionesSeleccionadas] = useState<string[]>(
    SECCIONES_SENADO_DEFAULT.map((s) => s.id)
  );

  // Diagnóstico y Verificación de Disponibilidad del Senado
  const [checkingDisponibilidad, setCheckingDisponibilidad] = useState<boolean>(false);
  const [disponibilidadMap, setDisponibilidadMap] = useState<Record<string, DetalleSeccionSenado> | null>(null);
  const [diagnosticoModalOpen, setDiagnosticoModalOpen] = useState<boolean>(false);
  const [ultimoDetalleSenado, setUltimoDetalleSenado] = useState<DetalleSeccionSenado[] | null>(null);

  // Scraper Status y Polling
  const [scrapingStatus, setScrapingStatus] = useState<any>(null);
  const [startingScrape, setStartingScrape] = useState<boolean>(false);
  const scraperPollingRef = useRef<NodeJS.Timeout | null>(null);

  // Generación y Edición de Síntesis Consolidada del Día
  const [generatingSynthesis, setGeneratingSynthesis] = useState<boolean>(false);
  const [editedText, setEditedText] = useState<string>("");
  const [savingDraft, setSavingDraft] = useState<boolean>(false);
  const [approving, setApproving] = useState<boolean>(false);
  const [confirmApproveOpen, setConfirmApproveOpen] = useState<boolean>(false);
  const [enviarWhatsAppCheck, setEnviarWhatsAppCheck] = useState<boolean>(true);

  // Previsualización de Documento Individual
  const [previewDocModalOpen, setPreviewDocModalOpen] = useState<boolean>(false);
  const [previewDocDetail, setPreviewDocDetail] = useState<any | null>(null);
  const [loadingPreviewDoc, setLoadingPreviewDoc] = useState<boolean>(false);

  // Revisión y Eliminación de Síntesis Anteriores
  const [reviewModalOpen, setReviewModalOpen] = useState<boolean>(false);
  const [reviewSintesisData, setReviewSintesisData] = useState<{
    fecha: string;
    texto: string;
    estado: string;
    modelo?: string;
  } | null>(null);
  const [loadingReview, setLoadingReview] = useState<boolean>(false);
  const [deleteConfirmOpen, setDeleteConfirmOpen] = useState<boolean>(false);
  const [dateToDelete, setDateToDelete] = useState<string | null>(null);
  const [deleteDateWithDocs, setDeleteDateWithDocs] = useState<boolean>(false);
  const [deletingSintesis, setDeletingSintesis] = useState<boolean>(false);

  // Eliminación de Documentos Individuales del Universo del Día
  const [docToDelete, setDocToDelete] = useState<{ id: string; nombre: string } | null>(null);
  const [deleteDocModalOpen, setDeleteDocModalOpen] = useState<boolean>(false);
  const [deletingDoc, setDeletingDoc] = useState<boolean>(false);

  // Mensajes de Alerta
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  const textareaRef = useRef<HTMLTextAreaElement | null>(null);
  const workspaceSectionRef = useRef<HTMLDivElement | null>(null);
  const editorSectionRef = useRef<HTMLDivElement | null>(null);

  // ==========================================================================
  // 1. Carga de Datos Inicial y Workspace
  // ==========================================================================

  const fetchHistorial = async () => {
    try {
      setLoadingHistorial(true);
      const data = await ApiService.getFechasSintesis(40, 0);
      setFechasHistorial(data);
    } catch (err: any) {
      console.error("Error al cargar historial de fechas:", err);
    } finally {
      setLoadingHistorial(false);
    }
  };

  const fetchWorkspace = async (targetFecha: string) => {
    try {
      setLoadingWorkspace(true);
      setError(null);
      const resp = await ApiService.getWorkspaceFecha(targetFecha);
      setWorkspace(resp);
      if (resp.sintesis && resp.sintesis.texto) {
        setEditedText(resp.sintesis.texto);
      } else {
        setEditedText("");
      }
    } catch (err: any) {
      setError(err.message || "Error al cargar los documentos de la fecha.");
    } finally {
      setLoadingWorkspace(false);
    }
  };

  const fetchCatalogoSenado = async () => {
    try {
      const data = await ApiService.getSeccionesSenado();
      if (data && data.length > 0) {
        setSeccionesCatalogo(data);
        setSeccionesSeleccionadas(data.map((s) => s.id));
      }
    } catch {
      // Usar catálogo por defecto
    }
  };

  useEffect(() => {
    fetchHistorial();
    fetchWorkspace(fechaTrabajo);
    fetchCatalogoSenado();
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

  // Sondeo reactivo mientras haya documentos en procesamiento OCR (Surya / Claude)
  useEffect(() => {
    const hayDocsEnProceso = workspace?.documentos?.some(
      (d) => d.estado === "pendiente_ocr" || d.estado === "en_ocr"
    );

    if (hayDocsEnProceso) {
      const interval = setInterval(async () => {
        try {
          const resp = await ApiService.getWorkspaceFecha(fechaTrabajo);
          setWorkspace(resp);
          if (resp.sintesis && resp.sintesis.texto && !editedText) {
            setEditedText(resp.sintesis.texto);
          }
        } catch {
          // Ignorar errores transitorios de polling en segundo plano
        }
      }, 3500);

      return () => clearInterval(interval);
    }
  }, [workspace?.documentos, fechaTrabajo, editedText]);

  const handleCambiarFecha = (nuevaFecha: string) => {
    setFechaTrabajo(nuevaFecha);
    setDisponibilidadMap(null);
    fetchWorkspace(nuevaFecha);
  };

  const handleStepFecha = (dias: number) => {
    try {
      const [y, m, d] = fechaTrabajo.split("-").map(Number);
      const cur = new Date(y, m - 1, d);
      cur.setDate(cur.getDate() + dias);
      const nueva = cur.toISOString().split("T")[0];
      handleCambiarFecha(nueva);
    } catch {
      // Fallback
    }
  };

  // ==========================================================================
  // 2. Scraper del Senado con Selección de Secciones
  // ==========================================================================

  const checkScraperStatus = async () => {
    try {
      const status = await ApiService.getScraperSenadoStatus();
      setScrapingStatus(status);
      return status;
    } catch {
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
        if (st.detalle_secciones && st.detalle_secciones.length > 0) {
          setUltimoDetalleSenado(st.detalle_secciones);
        }
        const noDescargados = st.detalle_secciones
          ? st.detalle_secciones.filter((d: any) => d.estado !== "descargado")
          : [];

        if (st.archivos_descargados && st.archivos_descargados.length > 0) {
          if (noDescargados.length > 0) {
            setSuccess(
              `✅ Sincronización completada: se descargaron y procesaron ${st.archivos_descargados.length} documentos. Hay ${noDescargados.length} sección(es) que el Senado no emitió para esta fecha.`
            );
          } else {
            setSuccess(
              `✅ Sincronización completada: se descargaron y procesaron ${st.archivos_descargados.length} documentos del Senado para ${st.fecha_objetivo || fechaTrabajo}.`
            );
          }
        } else if (st.errores && st.errores.length > 0) {
          setError(
            `⚠️ El portal del Senado no tiene documentos disponibles para la fecha ${st.fecha_objetivo || fechaTrabajo} (${st.errores[0]}). Puede verificar el diagnóstico o cargar documentos manualmente.`
          );
        } else {
          setSuccess(`Sincronización del Senado concluida.`);
        }
        fetchWorkspace(fechaTrabajo);
        fetchHistorial();
      }
    }, 2500);
  };

  const handleCheckDisponibilidad = async () => {
    try {
      setCheckingDisponibilidad(true);
      setError(null);
      const res = await ApiService.verificarDisponibilidadSenado(
        fechaTrabajo,
        seccionesCatalogo.map((s) => s.id)
      );
      const map: Record<string, DetalleSeccionSenado> = {};
      if (res && res.detalles) {
        res.detalles.forEach((d) => {
          map[d.id] = d;
        });
        setDisponibilidadMap(map);
        setUltimoDetalleSenado(res.detalles);
      }
    } catch (err: any) {
      setError(err.message || "Error al verificar disponibilidad en el portal del Senado.");
    } finally {
      setCheckingDisponibilidad(false);
    }
  };

  const handleSeleccionarSoloDisponibles = () => {
    if (!disponibilidadMap) return;
    const disponibles = seccionesCatalogo
      .filter((s) => disponibilidadMap[s.id]?.disponible)
      .map((s) => s.id);
    setSeccionesSeleccionadas(disponibles);
  };

  const handleTriggerScraperConSecciones = async () => {
    if (seccionesSeleccionadas.length === 0) {
      setError("Debe seleccionar al menos una sección del Senado para sincronizar.");
      return;
    }
    try {
      setStartingScrape(true);
      setError(null);
      await ApiService.ejecutarScraperSenadoConSecciones(fechaTrabajo, seccionesSeleccionadas);
      setSenadoModalOpen(false);
      setSuccess(
        `Sincronización del Senado iniciada para el día ${fechaTrabajo} (${seccionesSeleccionadas.length} secciones).`
      );
      await checkScraperStatus();
      startScraperPolling();
    } catch (err: any) {
      setError(err.message || "Error al iniciar sincronización del Senado.");
    } finally {
      setStartingScrape(false);
    }
  };

  const toggleSeccionSenado = (secId: string) => {
    setSeccionesSeleccionadas((prev) =>
      prev.includes(secId) ? prev.filter((id) => id !== secId) : [...prev, secId]
    );
  };

  const toggleAllSeccionesSenado = () => {
    if (seccionesSeleccionadas.length === seccionesCatalogo.length) {
      setSeccionesSeleccionadas([]);
    } else {
      setSeccionesSeleccionadas(seccionesCatalogo.map((s) => s.id));
    }
  };

  // ==========================================================================
  // 3. Carga Múltiple de Documentos ("N" Archivos)
  // ==========================================================================

  const handleFilesSelected = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (e.target.files && e.target.files.length > 0) {
      const newFiles = Array.from(e.target.files).filter((f) =>
        f.name.toLowerCase().endsWith(".pdf")
      );
      setStagedFiles((prev) => [...prev, ...newFiles]);
    }
  };

  const removeStagedFile = (index: number) => {
    setStagedFiles((prev) => prev.filter((_, i) => i !== index));
  };

  const handleUploadStagedFiles = async () => {
    if (stagedFiles.length === 0) {
      setUploadModalError("Seleccione al menos un archivo PDF para subir.");
      return;
    }

    try {
      setUploadingFiles(true);
      setUploadModalError(null);
      setError(null);
      const formData = new FormData();
      formData.append("fecha", fechaTrabajo);
      formData.append("origen", "manual");
      stagedFiles.forEach((file) => {
        formData.append("files", file);
      });

      const res = await ApiService.uploadMultipleBoletines(formData);
      setSuccess(`✅ ${res.mensaje}`);
      setStagedFiles([]);
      setUploadModalOpen(false);
      setUploadModalError(null);
      await fetchWorkspace(fechaTrabajo);
      await fetchHistorial();
    } catch (err: any) {
      const msg = err.message || "Error al subir los documentos.";
      setUploadModalError(msg);
      setError(msg);
    } finally {
      setUploadingFiles(false);
    }
  };

  // ==========================================================================
  // 4. Selección del Universo de Documentos para Síntesis
  // ==========================================================================

  const handleToggleDocSeleccion = async (docId: string, currentIncluido: boolean) => {
    try {
      const nuevoEstado = !currentIncluido;
      if (workspace) {
        const docsActualizados = workspace.documentos.map((d) =>
          d.id === docId ? { ...d, incluido_en_sintesis: nuevoEstado } : d
        );
        const countIncluidos = docsActualizados.filter((d) => d.incluido_en_sintesis).length;
        setWorkspace({
          ...workspace,
          documentos: docsActualizados,
          documentos_incluidos: countIncluidos,
        });
      }
      await ApiService.toggleDocumentoSeleccion(docId, nuevoEstado);
    } catch (err: any) {
      setError(err.message || "Error al actualizar la selección del documento.");
      fetchWorkspace(fechaTrabajo);
    }
  };

  const handleToggleAllDocs = async () => {
    if (!workspace || workspace.documentos.length === 0) return;
    const allSelected = workspace.documentos.every((d) => d.incluido_en_sintesis);
    const nuevoValor = !allSelected;

    try {
      const docsActualizados = workspace.documentos.map((d) => ({
        ...d,
        incluido_en_sintesis: nuevoValor,
      }));
      setWorkspace({
        ...workspace,
        documentos: docsActualizados,
        documentos_incluidos: nuevoValor ? docsActualizados.length : 0,
      });

      for (const d of workspace.documentos) {
        if (d.incluido_en_sintesis !== nuevoValor) {
          await ApiService.toggleDocumentoSeleccion(d.id, nuevoValor);
        }
      }
    } catch (err: any) {
      setError(err.message || "Error al alternar selección múltiple.");
      fetchWorkspace(fechaTrabajo);
    }
  };

  // ==========================================================================
  // 5. Consolidación de Síntesis del Día con Claude LLM
  // ==========================================================================

  const handleGenerarSintesisConsolidada = async () => {
    if (!workspace || workspace.documentos.length === 0) {
      setError("No hay documentos registrados para esta fecha.");
      return;
    }

    let docsIncluidos = workspace.documentos
      .filter((d) => d.incluido_en_sintesis)
      .map((d) => d.id);

    // Si ninguna casilla está marcada manualmente, incluir automáticamente todos los documentos registrados
    if (docsIncluidos.length === 0) {
      docsIncluidos = workspace.documentos.map((d) => d.id);
    }

    try {
      setGeneratingSynthesis(true);
      setError(null);
      const res = await ApiService.consolidarSintesisFecha(fechaTrabajo, docsIncluidos);
      if (res && res.sintesis) {
        setEditedText(res.sintesis.texto || "");
        setSuccess(
          `✅ Síntesis consolidada generada con éxito a partir de ${res.documentos_procesados || docsIncluidos.length} documentos.`
        );
        setTimeout(() => {
          if (editorSectionRef.current) {
            editorSectionRef.current.scrollIntoView({ behavior: "smooth" });
          }
        }, 150);
      }
      fetchWorkspace(fechaTrabajo);
      fetchHistorial();
    } catch (err: any) {
      console.error("[Consolidar Síntesis Error]:", err);
      setError(err.message || "Error al consolidar la síntesis del día.");
    } finally {
      setGeneratingSynthesis(false);
    }
  };

  // ==========================================================================
  // 6. Editor de Síntesis y Aprobación
  // ==========================================================================

  const applyTextFormat = (prefix: string, suffix: string = prefix) => {
    const textarea = textareaRef.current;
    if (!textarea) return;
    const start = textarea.selectionStart;
    const end = textarea.selectionEnd;
    const selected = editedText.substring(start, end);
    const replacement = prefix + selected + suffix;
    const newText = editedText.substring(0, start) + replacement + editedText.substring(end);
    setEditedText(newText);
    setTimeout(() => {
      textarea.focus();
      textarea.setSelectionRange(start + prefix.length, end + prefix.length);
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
    try {
      setSavingDraft(true);
      await ApiService.actualizarSintesisDiaria(fechaTrabajo, editedText);
      setSuccess("Borrador de síntesis consolidada guardado exitosamente.");
      fetchWorkspace(fechaTrabajo);
      fetchHistorial();
    } catch (err: any) {
      setError(err.message || "Error al guardar el borrador de la síntesis.");
    } finally {
      setSavingDraft(false);
    }
  };

  const handleApproveAndDispatch = async () => {
    try {
      setApproving(true);
      const res = await ApiService.aprobarSintesisDiaria(fechaTrabajo, editedText, enviarWhatsAppCheck);
      setSuccess(
        `✅ ${res.mensaje}. Destinatarios notificados: ${res.destinatarios_notificados}`
      );
      setConfirmApproveOpen(false);
      fetchWorkspace(fechaTrabajo);
      fetchHistorial();
    } catch (err: any) {
      setError(err.message || "Error al aprobar y despachar la síntesis diaria.");
    } finally {
      setApproving(false);
    }
  };

  const formatWhatsAppText = (text: string) => {
    if (!text) return "No hay contenido redactado todavía.";
    const escaped = text.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
    return escaped
      .replace(/\*([^*\n]+)\*/g, "<strong>$1</strong>")
      .replace(/_([^_\n]+)_/g, "<em>$1</em>")
      .replace(/~([^~\n]+)~/g, "<del>$1</del>")
      .replace(/```([^`]+)```/g, "<pre style='background:#f1f5f9;padding:4px;border-radius:4px;font-family:monospace'>$1</pre>")
      .replace(/`([^`\n]+)`/g, "<code style='background:#f1f5f9;padding:2px 4px;border-radius:3px;font-family:monospace'>$1</code>")
      .replace(/\n/g, "<br />");
  };

  const handleOpenDocPreview = async (docId: string) => {
    try {
      setLoadingPreviewDoc(true);
      setPreviewDocModalOpen(true);
      const detail = await ApiService.getBoletinDetail(docId);
      setPreviewDocDetail(detail);
    } catch (err: any) {
      setError(err.message || "Error al cargar el detalle del documento.");
    } finally {
      setLoadingPreviewDoc(false);
    }
  };

  const handleOpenReview = async (fecha: string) => {
    try {
      setLoadingReview(true);
      setReviewModalOpen(true);
      const ws = await ApiService.getWorkspaceFecha(fecha);
      if (ws && ws.sintesis) {
        setReviewSintesisData({
          fecha,
          texto: ws.sintesis.texto || "(Sin contenido redactado)",
          estado: ws.sintesis.estado || "borrador",
          modelo: ws.sintesis.modelo_usado || ws.sintesis.modelo_llm || "Claude 3.5 Sonnet",
        });
      } else {
        setReviewSintesisData({
          fecha,
          texto: "No se encontró el texto consolidado para esta fecha.",
          estado: "pendiente",
        });
      }
    } catch (err: any) {
      setError(err.message || "Error al cargar la síntesis para revisión.");
      setReviewModalOpen(false);
    } finally {
      setLoadingReview(false);
    }
  };

  const handlePromptDeleteSintesis = (fecha: string, defaultDeleteWithDocs = false) => {
    setDateToDelete(fecha);
    setDeleteDateWithDocs(defaultDeleteWithDocs);
    setDeleteConfirmOpen(true);
  };

  const handleConfirmDeleteSintesis = async () => {
    if (!dateToDelete) return;
    try {
      setDeletingSintesis(true);
      const res = await ApiService.eliminarSintesisDiaria(dateToDelete, deleteDateWithDocs);
      setSuccess(`✅ ${res.mensaje}`);
      setDeleteConfirmOpen(false);
      if (reviewModalOpen && reviewSintesisData?.fecha === dateToDelete) {
        setReviewModalOpen(false);
      }
      if (fechaTrabajo === dateToDelete) {
        await fetchWorkspace(dateToDelete);
      }
      await fetchHistorial();
    } catch (err: any) {
      setError(err.message || "Error al eliminar la síntesis diaria.");
    } finally {
      setDeletingSintesis(false);
      setDateToDelete(null);
      setDeleteDateWithDocs(false);
    }
  };

  const handlePromptDeleteDoc = (id: string, nombre: string) => {
    setDocToDelete({ id, nombre });
    setDeleteDocModalOpen(true);
  };

  const handleConfirmDeleteDoc = async () => {
    if (!docToDelete) return;
    try {
      setDeletingDoc(true);
      const res = await ApiService.eliminarBoletin(docToDelete.id);
      setSuccess(`✅ ${res.mensaje}`);
      setDeleteDocModalOpen(false);
      await fetchWorkspace(fechaTrabajo);
      await fetchHistorial();
    } catch (err: any) {
      setError(err.message || "Error al eliminar el documento.");
    } finally {
      setDeletingDoc(false);
      setDocToDelete(null);
    }
  };

  const handleReprocessDoc = async (id: string, nombre: string) => {
    try {
      setSuccess(`⏳ Encolando re-procesamiento OCR con Surya para: "${nombre}"...`);
      await ApiService.procesarBoletin(id);
      setSuccess(`✅ Re-procesamiento iniciado para "${nombre}". Los modelos Surya extraerán el contenido de las portadas.`);
      setTimeout(() => {
        fetchWorkspace(fechaTrabajo);
      }, 2500);
    } catch (err: any) {
      setError(err.message || "Error al solicitar el re-procesamiento OCR.");
    }
  };

  const getStatusChip = (estado: string) => {
    switch (estado) {
      case "aprobado":
        return <Chip label="Aprobado / Enviado" color="success" size="small" sx={{ fontWeight: 600 }} />;
      case "sintesis_lista":
        return <Chip label="Síntesis Lista" color="warning" size="small" sx={{ fontWeight: 600 }} />;
      case "ocr_completo":
        return <Chip label="OCR Completo" color="info" size="small" />;
      case "en_ocr":
        return <Chip label="En OCR..." color="warning" size="small" />;
      case "pendiente_ocr":
        return <Chip label="En Cola OCR" color="default" size="small" />;
      case "borrador":
        return <Chip label="Borrador" color="default" size="small" />;
      default:
        return <Chip label={estado} color="error" size="small" />;
    }
  };

  return (
    <Box sx={{ p: { xs: 2, md: 3 } }}>
      {/* Encabezado Principal */}
      <Box
        sx={{
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
          mb: 3,
          flexWrap: "wrap",
          gap: 2,
        }}
      >
        <Box>
          <Typography variant="h5" sx={{ fontWeight: 700, color: "#1e293b", display: "flex", alignItems: "center", gap: 1 }}>
            <LayersIcon sx={{ color: "#6366f1" }} /> Monitoreo Ejecutivo & Síntesis Diaria Consolidada
          </Typography>
          <Typography variant="body2" sx={{ color: "#64748b" }}>
            Universo de documentos por fecha (Senado de la República + cargas sectoriales) con síntesis ejecutiva única consolidada para WhatsApp.
          </Typography>
        </Box>
        <Box sx={{ display: "flex", gap: 1.5, alignItems: "center" }}>
          <Button
            variant="outlined"
            startIcon={<RefreshIcon />}
            onClick={() => {
              fetchHistorial();
              fetchWorkspace(fechaTrabajo);
            }}
            disabled={loadingWorkspace || loadingHistorial}
            sx={{ textTransform: "none", borderRadius: "8px" }}
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
          sx={{ mb: 3, borderRadius: "10px", alignItems: "center" }}
        >
          <strong>Sincronización desatendida del Senado en ejecución:</strong>{" "}
          {scrapingStatus.mensaje || "Descargando secciones..."}{" "}
          ({scrapingStatus.archivos_descargados?.length || 0} descargados,{" "}
          {scrapingStatus.archivos_procesados?.length || 0} procesados)
        </Alert>
      )}

      {error && (
        <Alert
          severity="error"
          sx={{ mb: 3, borderRadius: "8px", alignItems: "center" }}
          onClose={() => setError(null)}
          action={
            ultimoDetalleSenado && ultimoDetalleSenado.length > 0 ? (
              <Button
                color="inherit"
                size="small"
                onClick={() => setDiagnosticoModalOpen(true)}
                sx={{ fontWeight: 700, textTransform: "none", textDecoration: "underline" }}
              >
                Ver Diagnóstico
              </Button>
            ) : undefined
          }
        >
          {error}
        </Alert>
      )}

      {success && (
        <Alert
          severity="success"
          sx={{ mb: 3, borderRadius: "8px", alignItems: "center" }}
          onClose={() => setSuccess(null)}
          action={
            ultimoDetalleSenado && ultimoDetalleSenado.length > 0 ? (
              <Button
                color="inherit"
                size="small"
                onClick={() => setDiagnosticoModalOpen(true)}
                sx={{ fontWeight: 700, textTransform: "none", textDecoration: "underline" }}
              >
                Ver Diagnóstico
              </Button>
            ) : undefined
          }
        >
          {success}
        </Alert>
      )}

      {/* ===================================================================== */}
      {/* BARRA SUPERIOR: ESPACIO DE TRABAJO POR FECHA                          */}
      {/* ===================================================================== */}
      <Card
        ref={workspaceSectionRef}
        sx={{
          mb: 3,
          borderRadius: "14px",
          border: "1px solid #e2e8f0",
          boxShadow: "0 4px 12px rgba(0,0,0,0.03)",
          background: "linear-gradient(135deg, #ffffff 0%, #f8fafc 100%)",
        }}
      >
        <CardContent sx={{ p: 2.5 }}>
          <Grid container spacing={2} alignItems="center">
            {/* Selector de Fecha */}
            <Grid size={{ xs: 12, md: 5 }}>
              <Typography variant="caption" sx={{ color: "#64748b", fontWeight: 700, textTransform: "uppercase", letterSpacing: 0.5 }}>
                Jornada Activa de Trabajo
              </Typography>
              <Box sx={{ display: "flex", alignItems: "center", gap: 1, mt: 0.5 }}>
                <IconButton size="small" onClick={() => handleStepFecha(-1)} title="Día Anterior">
                  <ArrowBackIosNewIcon fontSize="small" />
                </IconButton>
                <TextField
                  type="date"
                  size="small"
                  value={fechaTrabajo}
                  onChange={(e) => handleCambiarFecha(e.target.value)}
                  sx={{ width: 175, backgroundColor: "#ffffff" }}
                />
                <IconButton size="small" onClick={() => handleStepFecha(1)} title="Día Siguiente">
                  <ArrowForwardIosIcon fontSize="small" />
                </IconButton>
                <Button
                  size="small"
                  variant="outlined"
                  onClick={() => handleCambiarFecha(new Date().toISOString().split("T")[0])}
                  sx={{ textTransform: "none", ml: 0.5 }}
                >
                  Hoy
                </Button>
              </Box>
            </Grid>

            {/* KPIs del Workspace del Día */}
            <Grid size={{ xs: 12, md: 4 }}>
              <Box sx={{ display: "flex", gap: 1.5, flexWrap: "wrap", alignItems: "center" }}>
                <Chip
                  icon={<PictureAsPdfIcon fontSize="small" />}
                  label={`${workspace?.total_documentos || 0} Docs Registrados`}
                  variant="outlined"
                  color="default"
                  sx={{ fontWeight: 600, backgroundColor: "#ffffff" }}
                />
                <Chip
                  icon={<CheckBoxIcon fontSize="small" />}
                  label={`${workspace?.documentos_incluidos || 0} en Síntesis`}
                  color="primary"
                  variant={workspace?.documentos_incluidos ? "filled" : "outlined"}
                  sx={{ fontWeight: 600 }}
                />
                {workspace?.sintesis?.estado && (
                  <Chip
                    label={`Síntesis: ${workspace.sintesis.estado.toUpperCase()}`}
                    color={
                      workspace.sintesis.estado === "aprobado"
                        ? "success"
                        : workspace.sintesis.estado === "sintesis_lista"
                        ? "warning"
                        : "default"
                    }
                    size="small"
                    sx={{ fontWeight: 700 }}
                  />
                )}
              </Box>
            </Grid>

            {/* Acciones para Nutrir el Día */}
            <Grid size={{ xs: 12, md: 3 }} sx={{ display: "flex", justifyContent: { xs: "flex-start", md: "flex-end" }, gap: 1 }}>
              <Button
                variant="contained"
                startIcon={<AccountBalanceIcon />}
                onClick={() => setSenadoModalOpen(true)}
                sx={{
                  backgroundColor: "#0f172a",
                  color: "#ffffff",
                  textTransform: "none",
                  fontWeight: 600,
                  borderRadius: "8px",
                  "&:hover": { backgroundColor: "#1e293b" },
                }}
              >
                Sincronizar Senado
              </Button>
              <Button
                variant="outlined"
                startIcon={<CloudUploadIcon />}
                onClick={() => {
                  setUploadModalError(null);
                  setUploadModalOpen(true);
                }}
                sx={{ textTransform: "none", fontWeight: 600, borderRadius: "8px" }}
              >
                Subir Documentos
              </Button>
            </Grid>
          </Grid>
        </CardContent>
      </Card>

      {/* ===================================================================== */}
      {/* SECCIÓN 1: UNIVERSO DE DOCUMENTOS DEL DÍA                             */}
      {/* ===================================================================== */}
      <Card sx={{ mb: 4, borderRadius: "12px", border: "1px solid #e2e8f0" }}>
        <Box
          sx={{
            p: 2,
            backgroundColor: "#f8fafc",
            borderBottom: "1px solid #e2e8f0",
            display: "flex",
            justifyContent: "space-between",
            alignItems: "center",
            flexWrap: "wrap",
            gap: 1.5,
          }}
        >
          <Box sx={{ display: "flex", alignItems: "center", gap: 1 }}>
            <Typography variant="h6" sx={{ fontWeight: 700, color: "#1e293b", fontSize: "1.05rem" }}>
              Universo de Documentos del Día ({fechaTrabajo})
            </Typography>
            <Chip
              label={`${workspace?.documentos_incluidos || 0} seleccionados para procesar`}
              size="small"
              color="primary"
              variant="outlined"
            />
          </Box>

          <Box sx={{ display: "flex", gap: 1, alignItems: "center" }}>
            <Button
              size="small"
              variant="text"
              onClick={handleToggleAllDocs}
              sx={{ textTransform: "none", fontWeight: 600 }}
            >
              {workspace?.documentos.every((d) => d.incluido_en_sintesis)
                ? "Deseleccionar Todos"
                : "Seleccionar Todos"}
            </Button>
            <Button
              variant="contained"
              startIcon={
                generatingSynthesis ? (
                  <CircularProgress size={18} sx={{ color: "#ffffff" }} />
                ) : (
                  <AutoAwesomeIcon sx={{ color: "#ffffff !important" }} />
                )
              }
              onClick={handleGenerarSintesisConsolidada}
              disabled={generatingSynthesis || !workspace || workspace.documentos.length === 0}
              sx={{
                textTransform: "none",
                fontWeight: 700,
                color: "#ffffff !important",
                "& *": { color: "#ffffff !important" },
                "&.Mui-disabled": { opacity: 0.6, color: "rgba(255, 255, 255, 0.7) !important" },
                borderRadius: "8px",
                background: "linear-gradient(135deg, #6366f1 0%, #4f46e5 100%)",
                boxShadow: "0 2px 8px rgba(99, 102, 241, 0.35)",
              }}
            >
              {generatingSynthesis
                ? "Consolidando con Claude..."
                : `⚡ Generar Síntesis Consolidada del Día (${
                    (workspace?.documentos_incluidos && workspace.documentos_incluidos > 0)
                      ? workspace.documentos_incluidos
                      : (workspace?.documentos?.length || 0)
                  })`}
            </Button>
          </Box>
        </Box>

        {loadingWorkspace ? (
          <Box sx={{ p: 4, textAlign: "center" }}>
            <CircularProgress size={32} />
            <Typography variant="body2" sx={{ mt: 1, color: "#64748b" }}>
              Cargando universo de documentos para {fechaTrabajo}...
            </Typography>
          </Box>
        ) : !workspace || workspace.documentos.length === 0 ? (
          <Box sx={{ p: 5, textAlign: "center" }}>
            <DescriptionIcon sx={{ fontSize: 48, color: "#cbd5e1", mb: 1 }} />
            <Typography variant="subtitle1" sx={{ fontWeight: 600, color: "#475569" }}>
              No hay documentos registrados para la fecha {fechaTrabajo}
            </Typography>
            <Typography variant="body2" sx={{ color: "#94a3b8", mb: 2, maxWidth: 500, mx: "auto" }}>
              Puede sincronizar automáticamente las síntesis desde el portal del Senado de la República o subir varios archivos PDF simultáneamente.
            </Typography>
            <Stack direction="row" spacing={2} justifyContent="center">
              <Button
                variant="contained"
                startIcon={<AccountBalanceIcon />}
                onClick={() => setSenadoModalOpen(true)}
                sx={{ backgroundColor: "#0f172a", textTransform: "none" }}
              >
                Sincronizar del Senado
              </Button>
              <Button
                variant="outlined"
                startIcon={<CloudUploadIcon />}
                onClick={() => setUploadModalOpen(true)}
                sx={{ textTransform: "none" }}
              >
                Subir Archivos PDF
              </Button>
            </Stack>
          </Box>
        ) : (
          <TableContainer>
            <Table size="small">
              <TableHead sx={{ backgroundColor: "#f1f5f9" }}>
                <TableRow>
                  <TableCell padding="checkbox" sx={{ pl: 2 }}>
                    <Checkbox
                      size="small"
                      checked={
                        workspace.documentos.length > 0 &&
                        workspace.documentos.every((d) => d.incluido_en_sintesis)
                      }
                      indeterminate={
                        workspace.documentos.some((d) => d.incluido_en_sintesis) &&
                        !workspace.documentos.every((d) => d.incluido_en_sintesis)
                      }
                      onChange={handleToggleAllDocs}
                      title="Seleccionar / Deseleccionar todos"
                    />
                  </TableCell>
                  <TableCell sx={{ fontWeight: 700, color: "#475569" }}>Documento / Archivo</TableCell>
                  <TableCell sx={{ fontWeight: 700, color: "#475569" }}>Origen</TableCell>
                  <TableCell sx={{ fontWeight: 700, color: "#475569" }}>Páginas</TableCell>
                  <TableCell sx={{ fontWeight: 700, color: "#475569" }}>Estado OCR</TableCell>
                  <TableCell sx={{ fontWeight: 700, color: "#475569" }} align="right">
                    Acciones
                  </TableCell>
                </TableRow>
              </TableHead>
              <TableBody>
                {workspace.documentos.map((doc) => (
                  <TableRow
                    key={doc.id}
                    hover
                    sx={{
                      backgroundColor: doc.incluido_en_sintesis ? "#ffffff" : "#f8fafc",
                      opacity: doc.incluido_en_sintesis ? 1 : 0.65,
                      transition: "background-color 0.15s ease",
                    }}
                  >
                    <TableCell padding="checkbox" sx={{ pl: 2 }}>
                      <Checkbox
                        size="small"
                        checked={doc.incluido_en_sintesis}
                        onChange={() => handleToggleDocSeleccion(doc.id, doc.incluido_en_sintesis)}
                        color="primary"
                      />
                    </TableCell>
                    <TableCell>
                      <Box sx={{ display: "flex", alignItems: "center", gap: 1 }}>
                        <PictureAsPdfIcon sx={{ color: "#ef4444", fontSize: 20 }} />
                        <Typography variant="body2" sx={{ fontWeight: 600, color: "#1e293b" }}>
                          {doc.nombre_archivo}
                        </Typography>
                      </Box>
                    </TableCell>
                    <TableCell>
                      {doc.origen === "senado" ? (
                        <Chip
                          icon={<AccountBalanceIcon style={{ fontSize: 14 }} />}
                          label="Senado"
                          size="small"
                          sx={{
                            backgroundColor: "#e0f2fe",
                            color: "#0369a1",
                            fontWeight: 700,
                            fontSize: "0.75rem",
                          }}
                        />
                      ) : (
                        <Chip
                          icon={<DescriptionIcon style={{ fontSize: 14 }} />}
                          label={doc.origen.toUpperCase()}
                          size="small"
                          sx={{
                            backgroundColor: "#ede9fe",
                            color: "#6d28d9",
                            fontWeight: 700,
                            fontSize: "0.75rem",
                          }}
                        />
                      )}
                    </TableCell>
                    <TableCell>
                      <Typography variant="body2" sx={{ color: "#64748b" }}>
                        {doc.total_paginas ? `${doc.total_paginas} págs.` : "—"}
                      </Typography>
                    </TableCell>
                    <TableCell>{getStatusChip(doc.estado)}</TableCell>
                    <TableCell align="right">
                      <Stack direction="row" spacing={0.5} justifyContent="flex-end">
                        <Tooltip title="Ver texto extraído y secciones">
                          <IconButton size="small" onClick={() => handleOpenDocPreview(doc.id)}>
                            <VisibilityIcon fontSize="small" />
                          </IconButton>
                        </Tooltip>
                        <Tooltip title="Re-procesar con Surya OCR">
                          <IconButton
                            size="small"
                            color="primary"
                            onClick={() => handleReprocessDoc(doc.id, doc.nombre_archivo)}
                            sx={{
                              border: "1px solid #bfdbfe",
                              backgroundColor: "#eff6ff",
                              "&:hover": { backgroundColor: "#dbeafe" },
                            }}
                          >
                            <AutorenewIcon fontSize="small" />
                          </IconButton>
                        </Tooltip>
                        <Tooltip title="Eliminar documento del universo">
                          <IconButton
                            size="small"
                            color="error"
                            onClick={() => handlePromptDeleteDoc(doc.id, doc.nombre_archivo)}
                            sx={{
                              border: "1px solid #fecaca",
                              backgroundColor: "#fef2f2",
                              "&:hover": { backgroundColor: "#fee2e2" },
                            }}
                          >
                            <DeleteOutlineIcon fontSize="small" />
                          </IconButton>
                        </Tooltip>
                      </Stack>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </TableContainer>
        )}
      </Card>

      {/* ===================================================================== */}
      {/* SECCIÓN 2: SÍNTESIS CONSOLIDADA DEL DÍA (EDITOR & SIMULADOR)          */}
      {/* ===================================================================== */}
      <Box ref={editorSectionRef} sx={{ pt: 1 }}>
        <Typography variant="h6" sx={{ fontWeight: 700, color: "#1e293b", mb: 2, display: "flex", alignItems: "center", gap: 1 }}>
          <AutoAwesomeIcon sx={{ color: "#f59e0b" }} /> Síntesis Diaria Consolidada — {fechaTrabajo}
        </Typography>
      </Box>

      <Grid container spacing={3} sx={{ mb: 4 }}>
        {/* Columna Izquierda: Editor Estilo Word */}
        <Grid size={{ xs: 12, lg: 7 }}>
          <Card sx={{ borderRadius: "12px", border: "1px solid #e2e8f0", height: "100%", display: "flex", flexDirection: "column" }}>
            {/* Header del Editor */}
            <Box
              sx={{
                p: 2,
                backgroundColor: "#f8fafc",
                borderBottom: "1px solid #e2e8f0",
                display: "flex",
                justifyContent: "space-between",
                alignItems: "center",
                flexWrap: "wrap",
                gap: 1,
              }}
            >
              <Box>
                <Typography variant="subtitle1" sx={{ fontWeight: 700, color: "#1e293b" }}>
                  Redacción del Briefing Ejecutivo
                </Typography>
                <Typography variant="caption" sx={{ color: "#64748b" }}>
                  Edite y dé formato antes de aprobar el envío. Las etiquetas de formato de WhatsApp se aplican automáticamente.
                </Typography>
              </Box>
              <Box sx={{ display: "flex", gap: 1 }}>
                <Button
                  size="small"
                  variant="outlined"
                  startIcon={savingDraft ? <CircularProgress size={16} /> : <SaveIcon />}
                  onClick={handleSaveDraft}
                  disabled={savingDraft || !editedText.trim()}
                  sx={{ textTransform: "none", fontWeight: 600 }}
                >
                  Guardar Borrador
                </Button>
                <Button
                  size="small"
                  variant="contained"
                  color="success"
                  startIcon={<SendIcon />}
                  onClick={() => setConfirmApproveOpen(true)}
                  disabled={!editedText.trim()}
                  sx={{ textTransform: "none", fontWeight: 700 }}
                >
                  Aprobar & Enviar
                </Button>
              </Box>
            </Box>

            {/* Barra de Herramientas Estilo Word */}
            <Box
              sx={{
                p: 1.5,
                borderBottom: "1px solid #e2e8f0",
                backgroundColor: "#ffffff",
                display: "flex",
                gap: 0.5,
                flexWrap: "wrap",
                alignItems: "center",
              }}
            >
              <ButtonGroup size="small" variant="outlined">
                <Tooltip title="Negrita (*texto*)">
                  <IconButton size="small" onClick={() => applyTextFormat("*")}>
                    <FormatBoldIcon fontSize="small" />
                  </IconButton>
                </Tooltip>
                <Tooltip title="Cursiva (_texto_)">
                  <IconButton size="small" onClick={() => applyTextFormat("_")}>
                    <FormatItalicIcon fontSize="small" />
                  </IconButton>
                </Tooltip>
                <Tooltip title="Tachado (~texto~)">
                  <IconButton size="small" onClick={() => applyTextFormat("~")}>
                    <FormatStrikethroughIcon fontSize="small" />
                  </IconButton>
                </Tooltip>
                <Tooltip title="Monoespaciado (```texto```)">
                  <IconButton size="small" onClick={() => applyTextFormat("```")}>
                    <CodeIcon fontSize="small" />
                  </IconButton>
                </Tooltip>
              </ButtonGroup>

              <Divider orientation="vertical" flexItem sx={{ mx: 0.5 }} />

              <ButtonGroup size="small" variant="outlined">
                <Tooltip title="Viñeta">
                  <IconButton size="small" onClick={() => insertQuickBadge("•")}>
                    <FormatListBulletedIcon fontSize="small" />
                  </IconButton>
                </Tooltip>
                <Tooltip title="Pin Operativo">
                  <IconButton size="small" onClick={() => insertQuickBadge("📌")}>
                    <Typography sx={{ fontSize: 13, lineHeight: 1 }}>📌</Typography>
                  </IconButton>
                </Tooltip>
                <Tooltip title="Alerta Crítica">
                  <IconButton size="small" onClick={() => insertQuickBadge("🚨")}>
                    <Typography sx={{ fontSize: 13, lineHeight: 1 }}>🚨</Typography>
                  </IconButton>
                </Tooltip>
              </ButtonGroup>

              <Divider orientation="vertical" flexItem sx={{ mx: 0.5 }} />

              {/* Botones de Etiquetas Rápidas */}
              <Box sx={{ display: "flex", gap: 0.5, flexWrap: "wrap" }}>
                <Chip
                  label="REGULATORIO"
                  size="small"
                  onClick={() => insertQuickBadge("*Regulación y Cumplimiento:*")}
                  sx={{ cursor: "pointer", fontSize: "0.7rem", height: 24 }}
                />
                <Chip
                  label="SEGURIDAD"
                  size="small"
                  onClick={() => insertQuickBadge("*Siniestralidad y Seguridad:*")}
                  sx={{ cursor: "pointer", fontSize: "0.7rem", height: 24 }}
                />
                <Chip
                  label="ECONOMÍA"
                  size="small"
                  onClick={() => insertQuickBadge("*Entorno Económico:*")}
                  sx={{ cursor: "pointer", fontSize: "0.7rem", height: 24 }}
                />
                <Chip
                  label="ATENCIÓN"
                  size="small"
                  color="warning"
                  onClick={() => insertQuickBadge("📌 *Atención Operativa:*")}
                  sx={{ cursor: "pointer", fontSize: "0.7rem", height: 24 }}
                />
              </Box>
            </Box>

            {/* Área de Texto Principal */}
            <Box sx={{ p: 2, flex: 1, display: "flex", flexDirection: "column" }}>
              <TextField
                inputRef={textareaRef}
                multiline
                rows={18}
                fullWidth
                placeholder="La síntesis consolidada del día aparecerá aquí tras generarla con Claude, o puede comenzar a redactarla manualmente..."
                value={editedText}
                onChange={(e) => setEditedText(e.target.value)}
                sx={{
                  flex: 1,
                  "& .MuiOutlinedInput-root": {
                    fontFamily: "monospace",
                    fontSize: "0.9rem",
                    lineHeight: 1.6,
                    backgroundColor: "#fafafa",
                  },
                }}
              />
              <Box sx={{ display: "flex", justifyContent: "space-between", mt: 1 }}>
                <Typography variant="caption" sx={{ color: "#94a3b8" }}>
                  {editedText.length} caracteres | {editedText.trim() ? editedText.trim().split(/\s+/).length : 0} palabras
                </Typography>
                {workspace?.sintesis?.modelo_usado && (
                  <Typography variant="caption" sx={{ color: "#64748b" }}>
                    Modelo: {workspace.sintesis.modelo_usado}
                  </Typography>
                )}
              </Box>
            </Box>
          </Card>
        </Grid>

        {/* Columna Derecha: Simulador de WhatsApp */}
        <Grid size={{ xs: 12, lg: 5 }}>
          <Card
            sx={{
              borderRadius: "14px",
              border: "1px solid #cbd5e1",
              backgroundColor: "#eae6df",
              overflow: "hidden",
              height: "100%",
              display: "flex",
              flexDirection: "column",
              boxShadow: "0 4px 14px rgba(0,0,0,0.06)",
            }}
          >
            {/* Header del Simulador WhatsApp */}
            <Box
              sx={{
                backgroundColor: "#075e54",
                color: "#ffffff",
                p: 1.5,
                display: "flex",
                alignItems: "center",
                gap: 1.5,
              }}
            >
              <Box
                sx={{
                  width: 38,
                  height: 38,
                  borderRadius: "50%",
                  backgroundColor: "#25d366",
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "center",
                }}
              >
                <WhatsAppIcon sx={{ color: "#ffffff", fontSize: 24 }} />
              </Box>
              <Box sx={{ flex: 1 }}>
                <Typography variant="subtitle2" sx={{ fontWeight: 700, lineHeight: 1.2 }}>
                  Coparmex — Monitoreo Ejecutivo
                </Typography>
                <Typography variant="caption" sx={{ opacity: 0.85, fontSize: "0.72rem" }}>
                  Canal Oficial de Inteligencia Operativa
                </Typography>
              </Box>
            </Box>

            {/* Chat Canvas */}
            <Box
              sx={{
                p: 2,
                flex: 1,
                overflowY: "auto",
                backgroundImage:
                  "radial-gradient(#cbd5e1 0.75px, transparent 0.75px)",
                backgroundSize: "16px 16px",
                display: "flex",
                flexDirection: "column",
                justifyContent: "flex-start",
              }}
            >
              <Box
                sx={{
                  maxWidth: "92%",
                  alignSelf: "flex-end",
                  backgroundColor: "#d9fdd3",
                  borderRadius: "8px 8px 0px 8px",
                  p: 1.75,
                  boxShadow: "0 1px 2px rgba(0,0,0,0.15)",
                  position: "relative",
                  wordBreak: "break-word",
                }}
              >
                <Typography
                  component="div"
                  variant="body2"
                  dangerouslySetInnerHTML={{ __html: formatWhatsAppText(editedText) }}
                  sx={{
                    color: "#111b21",
                    fontSize: "0.88rem",
                    lineHeight: 1.5,
                    "& strong": { fontWeight: 700 },
                    "& em": { fontStyle: "italic" },
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
                  <Typography variant="caption" sx={{ color: "#667781", fontSize: "0.7rem" }}>
                    {new Date().toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })}
                  </Typography>
                  <DoneAllIcon sx={{ color: "#53bdeb", fontSize: 16 }} />
                </Box>
              </Box>
            </Box>

            {/* Footer Simulador */}
            <Box
              sx={{
                p: 1.5,
                backgroundColor: "#f0f2f5",
                borderTop: "1px solid #d1d7db",
                textAlign: "center",
              }}
            >
              <Typography variant="caption" sx={{ color: "#667781" }}>
                Previsualización fiel del mensaje que recibirán los contactos vía Kapso WhatsApp Cloud API
              </Typography>
            </Box>
          </Card>
        </Grid>
      </Grid>

      {/* ===================================================================== */}
      {/* SECCIÓN 3: HISTORIAL DE FECHAS DE SÍNTESIS (VISTA PRINCIPAL)          */}
      {/* ===================================================================== */}
      <Card sx={{ borderRadius: "12px", border: "1px solid #e2e8f0" }}>
        <Box
          sx={{
            p: 2,
            backgroundColor: "#f8fafc",
            borderBottom: "1px solid #e2e8f0",
            display: "flex",
            justifyContent: "space-between",
            alignItems: "center",
          }}
        >
          <Box>
            <Typography variant="h6" sx={{ fontWeight: 700, color: "#1e293b", fontSize: "1.05rem" }}>
              Historial de Fechas de Síntesis
            </Typography>
            <Typography variant="caption" sx={{ color: "#64748b" }}>
              Seleccione cualquier fecha histórica para cargar su universo de documentos o editar su síntesis consolidada.
            </Typography>
          </Box>
        </Box>

        {loadingHistorial ? (
          <Box sx={{ p: 4, textAlign: "center" }}>
            <CircularProgress size={28} />
          </Box>
        ) : fechasHistorial.length === 0 ? (
          <Box sx={{ p: 4, textAlign: "center", color: "#64748b" }}>
            No se han registrado síntesis todavía. Sincronice el Senado o cargue documentos para comenzar.
          </Box>
        ) : (
          <TableContainer>
            <Table size="medium">
              <TableHead sx={{ backgroundColor: "#f1f5f9" }}>
                <TableRow>
                  <TableCell sx={{ fontWeight: 700, color: "#475569" }}>Fecha</TableCell>
                  <TableCell sx={{ fontWeight: 700, color: "#475569" }}>Universo Documental</TableCell>
                  <TableCell sx={{ fontWeight: 700, color: "#475569" }}>Páginas</TableCell>
                  <TableCell sx={{ fontWeight: 700, color: "#475569" }}>Estado Síntesis</TableCell>
                  <TableCell sx={{ fontWeight: 700, color: "#475569" }}>Vista Previa</TableCell>
                  <TableCell sx={{ fontWeight: 700, color: "#475569" }} align="right">
                    Acción
                  </TableCell>
                </TableRow>
              </TableHead>
              <TableBody>
                {fechasHistorial.map((f) => {
                  const isCurrent = f.fecha === fechaTrabajo;
                  return (
                    <TableRow
                      key={f.fecha}
                      hover
                      sx={{
                        backgroundColor: isCurrent ? "#f0fdf4" : "inherit",
                      }}
                    >
                      <TableCell>
                        <Box sx={{ display: "flex", alignItems: "center", gap: 1 }}>
                          <CalendarTodayIcon sx={{ fontSize: 18, color: "#6366f1" }} />
                          <Typography variant="subtitle2" sx={{ fontWeight: 700, color: "#1e293b" }}>
                            {f.fecha}
                          </Typography>
                          {isCurrent && (
                            <Chip label="Día Activo" color="success" size="small" sx={{ height: 20, fontSize: "0.68rem" }} />
                          )}
                        </Box>
                      </TableCell>
                      <TableCell>
                        <Box sx={{ display: "flex", gap: 0.75, alignItems: "center" }}>
                          <Chip
                            label={`${f.total_documentos} Total`}
                            size="small"
                            variant="outlined"
                            sx={{ fontWeight: 600 }}
                          />
                          {f.total_senado > 0 && (
                            <Chip
                              label={`${f.total_senado} Senado`}
                              size="small"
                              sx={{ backgroundColor: "#e0f2fe", color: "#0369a1", fontSize: "0.7rem", height: 20 }}
                            />
                          )}
                          {f.total_manual > 0 && (
                            <Chip
                              label={`${f.total_manual} Manuales`}
                              size="small"
                              sx={{ backgroundColor: "#ede9fe", color: "#6d28d9", fontSize: "0.7rem", height: 20 }}
                            />
                          )}
                        </Box>
                      </TableCell>
                      <TableCell>
                        <Typography variant="body2" sx={{ color: "#64748b" }}>
                          {f.total_paginas} págs.
                        </Typography>
                      </TableCell>
                      <TableCell>{getStatusChip(f.estado_sintesis || "pendiente")}</TableCell>
                      <TableCell sx={{ maxWidth: 300 }}>
                        <Typography
                          variant="caption"
                          sx={{
                            color: "#64748b",
                            display: "-webkit-box",
                            WebkitLineClamp: 2,
                            WebkitBoxOrient: "vertical",
                            overflow: "hidden",
                          }}
                        >
                          {f.texto_preview || "Sin síntesis consolidada redactada."}
                        </Typography>
                      </TableCell>
                      <TableCell align="right">
                        <Stack direction="row" spacing={1} justifyContent="flex-end" alignItems="center">
                          <Button
                            size="small"
                            variant={isCurrent ? "contained" : "outlined"}
                            color={isCurrent ? "success" : "primary"}
                            startIcon={<FolderOpenIcon />}
                            onClick={() => {
                              handleCambiarFecha(f.fecha);
                              if (workspaceSectionRef.current) {
                                workspaceSectionRef.current.scrollIntoView({ behavior: "smooth" });
                              }
                            }}
                            sx={{ textTransform: "none", fontWeight: 600, borderRadius: "6px" }}
                          >
                            {isCurrent ? "Trabajando Día" : "Trabajar Día"}
                          </Button>
                          {f.sintesis_id && (
                            <Tooltip title="Revisar Síntesis Consolidada">
                              <IconButton
                                size="small"
                                color="info"
                                onClick={() => handleOpenReview(f.fecha)}
                                sx={{
                                  border: "1px solid #bfdbfe",
                                  backgroundColor: "#eff6ff",
                                  "&:hover": { backgroundColor: "#dbeafe" },
                                }}
                              >
                                <VisibilityIcon fontSize="small" />
                              </IconButton>
                            </Tooltip>
                          )}
                          <Tooltip title={f.sintesis_id ? "Eliminar Síntesis o Registro del Día" : "Eliminar Fecha y Registros no Procesados"}>
                            <IconButton
                              size="small"
                              color="error"
                              onClick={() => handlePromptDeleteSintesis(f.fecha, !f.sintesis_id || f.total_documentos > 0)}
                              sx={{
                                border: "1px solid #fecaca",
                                backgroundColor: "#fef2f2",
                                "&:hover": { backgroundColor: "#fee2e2" },
                              }}
                            >
                              <DeleteOutlineIcon fontSize="small" />
                            </IconButton>
                          </Tooltip>
                        </Stack>
                      </TableCell>
                    </TableRow>
                  );
                })}
              </TableBody>
            </Table>
          </TableContainer>
        )}
      </Card>

      {/* ===================================================================== */}
      {/* DIÁLOGO: SELECCIÓN DE SECCIONES DEL SENADO                            */}
      {/* ===================================================================== */}
      <Dialog
        open={senadoModalOpen}
        onClose={() => setSenadoModalOpen(false)}
        maxWidth="md"
        fullWidth
        PaperProps={{ sx: { borderRadius: "12px" } }}
      >
        <DialogTitle sx={{ fontWeight: 700, borderBottom: "1px solid #e2e8f0" }}>
          Sincronizar Síntesis del Senado de la República
        </DialogTitle>
        <DialogContent sx={{ pt: 2.5 }}>
          <Typography variant="body2" sx={{ color: "#475569", mb: 2 }}>
            Seleccione qué secciones del portal del Senado desea extraer automáticamente para el día{" "}
            <strong>{fechaTrabajo}</strong>. Los documentos se agregarán al universo del día.
          </Typography>

          <Box sx={{ display: "flex", flexWrap: "wrap", justifyContent: "space-between", alignItems: "center", mb: 2, gap: 1 }}>
            <Box sx={{ display: "flex", flexWrap: "wrap", alignItems: "center", gap: 1 }}>
              <Button
                size="small"
                variant="outlined"
                color="secondary"
                startIcon={checkingDisponibilidad ? <CircularProgress size={14} color="inherit" /> : <SearchIcon fontSize="small" />}
                onClick={handleCheckDisponibilidad}
                disabled={checkingDisponibilidad || startingScrape}
                sx={{ textTransform: "none", fontWeight: 600, borderRadius: "6px" }}
              >
                {checkingDisponibilidad ? "Comprobando en portal..." : "Comprobar Disponibilidad en Senado"}
              </Button>
              {disponibilidadMap && (
                <Button
                  size="small"
                  variant="text"
                  color="success"
                  onClick={handleSeleccionarSoloDisponibles}
                  sx={{ textTransform: "none", fontWeight: 600 }}
                >
                  Seleccionar solo disponibles ({Object.values(disponibilidadMap).filter((d) => d.disponible).length})
                </Button>
              )}
            </Box>
            <Button size="small" onClick={toggleAllSeccionesSenado} sx={{ textTransform: "none" }}>
              {seccionesSeleccionadas.length === seccionesCatalogo.length
                ? "Deseleccionar Todas"
                : "Seleccionar Todas"}
            </Button>
          </Box>

          {disponibilidadMap && (
            <Alert
              severity="info"
              icon={<InfoOutlinedIcon fontSize="inherit" />}
              sx={{ mb: 2, borderRadius: "8px", py: 0.5 }}
            >
              Comprobación completada para <strong>{fechaTrabajo}</strong>:{" "}
              <strong>{Object.values(disponibilidadMap).filter((d) => d.disponible).length}</strong> publicadas y disponibles para descarga,{" "}
              <strong>{Object.values(disponibilidadMap).filter((d) => !d.disponible).length}</strong> no emitidas hoy en el servidor del Senado.
            </Alert>
          )}

          <Grid container spacing={1}>
            {seccionesCatalogo.map((sec) => {
              const isChecked = seccionesSeleccionadas.includes(sec.id);
              const infoDisp = disponibilidadMap ? disponibilidadMap[sec.id] : null;
              return (
                <Grid size={{ xs: 12, sm: 6 }} key={sec.id}>
                  <Card
                    variant="outlined"
                    onClick={() => toggleSeccionSenado(sec.id)}
                    sx={{
                      cursor: "pointer",
                      p: 1.25,
                      borderRadius: "8px",
                      backgroundColor: isChecked ? "#f0f9ff" : "#ffffff",
                      borderColor: isChecked ? "#0284c7" : "#e2e8f0",
                      display: "flex",
                      alignItems: "center",
                      gap: 1.5,
                      transition: "all 0.15s ease",
                      "&:hover": { borderColor: "#0284c7" },
                    }}
                  >
                    <Checkbox checked={isChecked} size="small" color="primary" />
                    <Box sx={{ flex: 1, minWidth: 0 }}>
                      <Box sx={{ display: "flex", alignItems: "center", justifyContent: "space-between", gap: 1 }}>
                        <Typography variant="body2" sx={{ fontWeight: 600, color: "#1e293b", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                          {sec.nombre}
                        </Typography>
                        {infoDisp && (
                          infoDisp.disponible ? (
                            <Chip
                              size="small"
                              label={infoDisp.tamano_bytes ? `${Math.round(infoDisp.tamano_bytes / 1024)} KB` : "Disponible"}
                              sx={{
                                bgcolor: "#dcfce7",
                                color: "#15803d",
                                fontWeight: 700,
                                fontSize: "0.7rem",
                                height: 20,
                              }}
                            />
                          ) : (
                            <Tooltip title={infoDisp.motivo || "No publicado por el Senado"} arrow>
                              <Chip
                                size="small"
                                label={infoDisp.codigo_http === 404 ? "No publicado (404)" : "No disp."}
                                sx={{
                                  bgcolor: "#fee2e2",
                                  color: "#b91c1c",
                                  fontWeight: 600,
                                  fontSize: "0.7rem",
                                  height: 20,
                                }}
                              />
                            </Tooltip>
                          )
                        )}
                      </Box>
                      <Typography variant="caption" sx={{ color: "#64748b" }}>
                        {sec.archivo}
                      </Typography>
                    </Box>
                  </Card>
                </Grid>
              );
            })}
          </Grid>
        </DialogContent>
        <DialogActions sx={{ p: 2, borderTop: "1px solid #e2e8f0" }}>
          {ultimoDetalleSenado && ultimoDetalleSenado.length > 0 && (
            <Button
              size="small"
              onClick={() => {
                setSenadoModalOpen(false);
                setDiagnosticoModalOpen(true);
              }}
              sx={{ textTransform: "none", mr: "auto" }}
            >
              Ver Tabla de Diagnóstico
            </Button>
          )}
          <Button onClick={() => setSenadoModalOpen(false)} sx={{ textTransform: "none" }}>
            Cancelar
          </Button>
          <Button
            variant="contained"
            startIcon={startingScrape ? <CircularProgress size={16} color="inherit" /> : <AccountBalanceIcon />}
            onClick={handleTriggerScraperConSecciones}
            disabled={startingScrape || seccionesSeleccionadas.length === 0}
            sx={{
              backgroundColor: "#0f172a",
              textTransform: "none",
              fontWeight: 700,
              "&:hover": { backgroundColor: "#1e293b" },
            }}
          >
            Iniciar Descarga ({seccionesSeleccionadas.length} Secciones)
          </Button>
        </DialogActions>
      </Dialog>

      {/* ===================================================================== */}
      {/* DIÁLOGO: DIAGNÓSTICO DE DESCARGA / DISPONIBILIDAD DEL SENADO          */}
      {/* ===================================================================== */}
      <Dialog
        open={diagnosticoModalOpen}
        onClose={() => setDiagnosticoModalOpen(false)}
        maxWidth="md"
        fullWidth
        PaperProps={{ sx: { borderRadius: "12px" } }}
      >
        <DialogTitle sx={{ fontWeight: 700, borderBottom: "1px solid #e2e8f0", display: "flex", justifyContent: "space-between", alignItems: "center" }}>
          <Box sx={{ display: "flex", alignItems: "center", gap: 1 }}>
            <SearchIcon sx={{ color: "#0284c7" }} />
            <span>Diagnóstico de Disponibilidad — Senado de la República</span>
          </Box>
          <IconButton size="small" onClick={() => setDiagnosticoModalOpen(false)}>
            <CloseIcon fontSize="small" />
          </IconButton>
        </DialogTitle>
        <DialogContent sx={{ pt: 2.5 }}>
          <Typography variant="body2" sx={{ color: "#475569", mb: 2 }}>
            A continuación se detalla el estado de cada sección consultada en el servidor oficial del Senado (<code>comunicacionsocial.senado.gob.mx</code>) para la fecha <strong>{fechaTrabajo}</strong>.
          </Typography>

          <TableContainer component={Paper} variant="outlined" sx={{ borderRadius: "8px", mb: 2.5 }}>
            <Table size="small">
              <TableHead sx={{ backgroundColor: "#f8fafc" }}>
                <TableRow>
                  <TableCell sx={{ fontWeight: 700, color: "#475569" }}>Sección</TableCell>
                  <TableCell sx={{ fontWeight: 700, color: "#475569" }}>Archivo Oficial</TableCell>
                  <TableCell sx={{ fontWeight: 700, color: "#475569" }}>Estado en Servidor</TableCell>
                  <TableCell sx={{ fontWeight: 700, color: "#475569" }}>Detalle / Motivo</TableCell>
                </TableRow>
              </TableHead>
              <TableBody>
                {ultimoDetalleSenado && ultimoDetalleSenado.length > 0 ? (
                  ultimoDetalleSenado.map((det) => {
                    const isOk = det.estado === "descargado" || det.disponible;
                    return (
                      <TableRow key={det.id} hover>
                        <TableCell sx={{ fontWeight: 600, color: "#1e293b" }}>
                          {det.nombre}
                        </TableCell>
                        <TableCell sx={{ fontFamily: "monospace", fontSize: "0.8rem", color: "#64748b" }}>
                          {det.archivo}
                        </TableCell>
                        <TableCell>
                          {isOk ? (
                            <Chip
                              size="small"
                              label={`Disponible (HTTP ${det.codigo_http || 200})`}
                              sx={{ bgcolor: "#dcfce7", color: "#15803d", fontWeight: 700, fontSize: "0.75rem" }}
                            />
                          ) : (
                            <Chip
                              size="small"
                              label={`No disponible (HTTP ${det.codigo_http || 404})`}
                              sx={{ bgcolor: "#fee2e2", color: "#b91c1c", fontWeight: 700, fontSize: "0.75rem" }}
                            />
                          )}
                        </TableCell>
                        <TableCell sx={{ fontSize: "0.82rem", color: isOk ? "#15803d" : "#64748b" }}>
                          {det.motivo}
                          {det.tamano_bytes ? ` (${Math.round(det.tamano_bytes / 1024)} KB)` : ""}
                        </TableCell>
                      </TableRow>
                    );
                  })
                ) : (
                  <TableRow>
                    <TableCell colSpan={4} align="center" sx={{ py: 3, color: "#64748b" }}>
                      No hay diagnóstico registrado aún. Ejecute "Comprobar Disponibilidad" o una sincronización del Senado.
                    </TableCell>
                  </TableRow>
                )}
              </TableBody>
            </Table>
          </TableContainer>

          <Alert severity="info" sx={{ borderRadius: "8px" }}>
            <strong>¿Por qué faltan secciones?</strong> El Senado de la República no emite las 10 secciones todos los días. En fines de semana, días festivos o días sin sesión parlamentaria, secciones como <em>Redes</em>, <em>Columnas Senado</em> o <em>Diputados</em> no se publican y devuelven <strong>HTTP 404</strong>. Si cuenta con esos archivos por otro medio, puede añadirlos manualmente con el botón <strong>Subir Documentos</strong>.
          </Alert>
        </DialogContent>
        <DialogActions sx={{ p: 2, borderTop: "1px solid #e2e8f0" }}>
          <Button onClick={() => setDiagnosticoModalOpen(false)} sx={{ textTransform: "none" }}>
            Cerrar
          </Button>
          <Button
            variant="contained"
            onClick={() => {
              setDiagnosticoModalOpen(false);
              setSenadoModalOpen(true);
            }}
            sx={{ textTransform: "none", backgroundColor: "#0f172a" }}
          >
            Volver a Sincronización
          </Button>
        </DialogActions>
      </Dialog>

      {/* ===================================================================== */}
      {/* DIÁLOGO: CARGA MÚLTIPLE DE DOCUMENTOS ("N" ARCHIVOS)                  */}
      {/* ===================================================================== */}
      <Dialog
        open={uploadModalOpen}
        onClose={() => setUploadModalOpen(false)}
        maxWidth="sm"
        fullWidth
        PaperProps={{ sx: { borderRadius: "12px" } }}
      >
        <DialogTitle sx={{ fontWeight: 700, borderBottom: "1px solid #e2e8f0" }}>
          Subir Múltiples Documentos al Día {fechaTrabajo}
        </DialogTitle>
        <DialogContent sx={{ pt: 2.5 }}>
          {uploadModalError && (
            <Alert
              severity="error"
              sx={{ mb: 2, borderRadius: "8px" }}
              onClose={() => setUploadModalError(null)}
            >
              {uploadModalError}
            </Alert>
          )}

          <Typography variant="body2" sx={{ color: "#475569", mb: 2 }}>
            Arrastre o seleccione todos los boletines, alertas o reportes sectoriales (formato PDF) que formarán parte de la síntesis del día.
          </Typography>

          <Box
            component="label"
            sx={{
              display: "flex",
              flexDirection: "column",
              alignItems: "center",
              justifyContent: "center",
              border: "2px dashed #cbd5e1",
              borderRadius: "10px",
              p: 3,
              backgroundColor: "#f8fafc",
              cursor: "pointer",
              mb: 2,
              "&:hover": { borderColor: "#6366f1", backgroundColor: "#f1f5f9" },
            }}
          >
            <input
              type="file"
              accept=".pdf,application/pdf"
              multiple
              style={{ display: "none" }}
              onChange={handleFilesSelected}
            />
            <CloudUploadIcon sx={{ fontSize: 40, color: "#6366f1", mb: 1 }} />
            <Typography variant="body2" sx={{ fontWeight: 600, color: "#1e293b" }}>
              Haga clic aquí o arrastre múltiples archivos PDF
            </Typography>
            <Typography variant="caption" sx={{ color: "#64748b" }}>
              Puede seleccionar varios documentos al mismo tiempo
            </Typography>
          </Box>

          {stagedFiles.length > 0 && (
            <Box sx={{ mt: 2 }}>
              <Typography variant="caption" sx={{ fontWeight: 700, color: "#64748b", mb: 1, display: "block" }}>
                ARCHIVOS PREPARADOS ({stagedFiles.length})
              </Typography>
              <Stack spacing={1} sx={{ maxHeight: 220, overflowY: "auto" }}>
                {stagedFiles.map((file, idx) => (
                  <Box
                    key={idx}
                    sx={{
                      p: 1,
                      backgroundColor: "#f1f5f9",
                      borderRadius: "6px",
                      display: "flex",
                      alignItems: "center",
                      justifyContent: "space-between",
                    }}
                  >
                    <Box sx={{ display: "flex", alignItems: "center", gap: 1, overflow: "hidden" }}>
                      <PictureAsPdfIcon sx={{ color: "#ef4444", fontSize: 18 }} />
                      <Typography variant="body2" noWrap sx={{ maxWidth: 360, fontWeight: 500 }}>
                        {file.name}
                      </Typography>
                      <Typography variant="caption" sx={{ color: "#94a3b8" }}>
                        ({(file.size / 1024 / 1024).toFixed(2)} MB)
                      </Typography>
                    </Box>
                    <IconButton size="small" onClick={() => removeStagedFile(idx)}>
                      <DeleteOutlineIcon fontSize="small" sx={{ color: "#ef4444" }} />
                    </IconButton>
                  </Box>
                ))}
              </Stack>
            </Box>
          )}

          {uploadingFiles && (
            <Box sx={{ mt: 2 }}>
              <LinearProgress sx={{ borderRadius: 4 }} />
              <Typography variant="caption" sx={{ mt: 0.5, display: "block", textAlign: "center", color: "#64748b" }}>
                Subiendo y encolando OCR Surya...
              </Typography>
            </Box>
          )}
        </DialogContent>
        <DialogActions sx={{ p: 2, borderTop: "1px solid #e2e8f0" }}>
          <Button onClick={() => setUploadModalOpen(false)} disabled={uploadingFiles} sx={{ textTransform: "none" }}>
            Cancelar
          </Button>
          <Button
            variant="contained"
            onClick={handleUploadStagedFiles}
            disabled={uploadingFiles || stagedFiles.length === 0}
            sx={{ textTransform: "none", fontWeight: 700 }}
          >
            Subir {stagedFiles.length} Archivos
          </Button>
        </DialogActions>
      </Dialog>

      {/* ===================================================================== */}
      {/* DIÁLOGO: CONFIRMACIÓN DE APROBACIÓN Y DESPACHO WHATSAPP               */}
      {/* ===================================================================== */}
      <Dialog
        open={confirmApproveOpen}
        onClose={() => setConfirmApproveOpen(false)}
        maxWidth="xs"
        fullWidth
        PaperProps={{ sx: { borderRadius: "12px" } }}
      >
        <DialogTitle sx={{ fontWeight: 700 }}>Confirmar Aprobación y Despacho</DialogTitle>
        <DialogContent>
          <Typography variant="body2" sx={{ color: "#475569", mb: 2 }}>
            ¿Desea aprobar la Síntesis Diaria Consolidada del <strong>{fechaTrabajo}</strong>?
          </Typography>
          <FormControlLabel
            control={
              <Checkbox
                checked={enviarWhatsAppCheck}
                onChange={(e) => setEnviarWhatsAppCheck(e.target.checked)}
                color="success"
              />
            }
            label={
              <Typography variant="body2" sx={{ fontWeight: 600 }}>
                Despachar inmediatamente vía WhatsApp a los contactos activos de la lista de distribución
              </Typography>
            }
          />
        </DialogContent>
        <DialogActions sx={{ p: 2 }}>
          <Button onClick={() => setConfirmApproveOpen(false)} sx={{ textTransform: "none" }}>
            Cancelar
          </Button>
          <Button
            variant="contained"
            color="success"
            startIcon={approving ? <CircularProgress size={16} color="inherit" /> : <CheckCircleIcon />}
            onClick={handleApproveAndDispatch}
            disabled={approving}
            sx={{ textTransform: "none", fontWeight: 700 }}
          >
            {approving ? "Enviando..." : "Confirmar & Enviar"}
          </Button>
        </DialogActions>
      </Dialog>

      {/* ===================================================================== */}
      {/* DIÁLOGO: PREVISUALIZAR DETALLE DE UN DOCUMENTO INDIVIDUAL             */}
      {/* ===================================================================== */}
      <Dialog
        open={previewDocModalOpen}
        onClose={() => setPreviewDocModalOpen(false)}
        maxWidth="md"
        fullWidth
        PaperProps={{ sx: { borderRadius: "12px" } }}
      >
        <DialogTitle sx={{ fontWeight: 700, display: "flex", justifyContent: "space-between", alignItems: "center" }}>
          <span>Detalle del Documento</span>
          <IconButton size="small" onClick={() => setPreviewDocModalOpen(false)}>
            <CloseIcon fontSize="small" />
          </IconButton>
        </DialogTitle>
        <DialogContent sx={{ pt: 1 }}>
          {loadingPreviewDoc ? (
            <Box sx={{ p: 4, textAlign: "center" }}>
              <CircularProgress size={28} />
            </Box>
          ) : previewDocDetail ? (
            <Box>
              <Typography variant="subtitle1" sx={{ fontWeight: 700, color: "#1e293b" }}>
                {previewDocDetail.boletin.nombre_archivo}
              </Typography>
              <Box sx={{ display: "flex", justifyContent: "space-between", alignItems: "center", mb: 2, flexWrap: "wrap", gap: 1 }}>
                <Typography variant="caption" sx={{ color: "#64748b" }}>
                  ID: {previewDocDetail.boletin.id} | Fecha: {previewDocDetail.boletin.fecha_boletin} | Estado: {previewDocDetail.boletin.estado}
                </Typography>
                <Button
                  variant="outlined"
                  size="small"
                  color="primary"
                  startIcon={<AutorenewIcon />}
                  onClick={() => {
                    handleReprocessDoc(previewDocDetail.boletin.id, previewDocDetail.boletin.nombre_archivo);
                    setPreviewDocModalOpen(false);
                  }}
                  sx={{ textTransform: "none", fontSize: "0.78rem" }}
                >
                  Re-procesar OCR (Surya)
                </Button>
              </Box>

              <Typography variant="subtitle2" sx={{ fontWeight: 700, color: "#475569", mb: 1 }}>
                Secciones extraídas por OCR Surya ({previewDocDetail.secciones?.length || 0}):
              </Typography>
              <Stack spacing={1.5} sx={{ maxHeight: 380, overflowY: "auto", pr: 1 }}>
                {previewDocDetail.secciones && previewDocDetail.secciones.length > 0 ? (
                  previewDocDetail.secciones.map((sec: any, idx: number) => {
                    const texto =
                      sec.contenido?.texto_completo ||
                      (typeof sec.contenido === "string" ? sec.contenido : "");
                    return (
                      <Paper key={idx} variant="outlined" sx={{ p: 1.5, borderRadius: "8px", backgroundColor: "#f8fafc" }}>
                        <Box sx={{ display: "flex", justifyContent: "space-between", mb: 0.5 }}>
                          <Chip label={sec.tema || "General"} size="small" color="primary" sx={{ height: 20, fontSize: "0.7rem" }} />
                          <Typography variant="caption" sx={{ color: "#94a3b8" }}>
                            Páginas {sec.pagina_inicio} - {sec.pagina_fin}
                          </Typography>
                        </Box>
                        <Typography variant="body2" sx={{ color: "#334155", whiteSpace: "pre-line", maxHeight: 120, overflowY: "auto", fontSize: "0.82rem" }}>
                          {texto ? texto.substring(0, 450) + "..." : "Sin texto extraído."}
                        </Typography>
                      </Paper>
                    );
                  })
                ) : (
                  <Typography variant="body2" sx={{ color: "#94a3b8" }}>
                    No se han generado secciones todavía.
                  </Typography>
                )}
              </Stack>
            </Box>
          ) : null}
        </DialogContent>
      </Dialog>

      {/* ===================================================================== */}
      {/* DIÁLOGO: REVISIÓN DE SÍNTESIS CONSOLIDADA ANTERIOR                   */}
      {/* ===================================================================== */}
      <Dialog
        open={reviewModalOpen}
        onClose={() => setReviewModalOpen(false)}
        maxWidth="md"
        fullWidth
        PaperProps={{ sx: { borderRadius: "12px" } }}
      >
        <DialogTitle
          sx={{
            fontWeight: 700,
            display: "flex",
            justifyContent: "space-between",
            alignItems: "center",
            borderBottom: "1px solid #e2e8f0",
            backgroundColor: "#f8fafc",
          }}
        >
          <Box sx={{ display: "flex", alignItems: "center", gap: 1.5 }}>
            <VisibilityIcon sx={{ color: "#6366f1" }} />
            <span>Síntesis Consolidada del {reviewSintesisData?.fecha}</span>
            {reviewSintesisData?.estado && getStatusChip(reviewSintesisData.estado)}
          </Box>
          <IconButton size="small" onClick={() => setReviewModalOpen(false)}>
            <CloseIcon fontSize="small" />
          </IconButton>
        </DialogTitle>
        <DialogContent sx={{ pt: 2.5 }}>
          {loadingReview ? (
            <Box sx={{ p: 4, textAlign: "center" }}>
              <CircularProgress size={32} />
              <Typography variant="body2" sx={{ mt: 1, color: "#64748b" }}>
                Cargando contenido de la síntesis...
              </Typography>
            </Box>
          ) : reviewSintesisData ? (
            <Box sx={{ mt: 1 }}>
              <Box sx={{ display: "flex", justifyContent: "space-between", alignItems: "center", mb: 1.5 }}>
                <Typography variant="caption" sx={{ color: "#64748b", fontWeight: 600 }}>
                  Modelo: {reviewSintesisData.modelo || "Claude 3.5 Sonnet"}
                </Typography>
                <Button
                  size="small"
                  variant="outlined"
                  onClick={() => {
                    navigator.clipboard.writeText(reviewSintesisData.texto);
                    setSuccess("Texto de la síntesis copiado al portapapeles.");
                  }}
                  sx={{ textTransform: "none", fontSize: "0.75rem" }}
                >
                  Copiar Texto
                </Button>
              </Box>
              <Paper
                variant="outlined"
                sx={{
                  p: 2.5,
                  borderRadius: "8px",
                  backgroundColor: "#ffffff",
                  maxHeight: "55vh",
                  overflowY: "auto",
                  border: "1px solid #cbd5e1",
                  fontFamily: "inherit",
                }}
              >
                <Typography
                  component="div"
                  variant="body2"
                  dangerouslySetInnerHTML={{ __html: formatWhatsAppText(reviewSintesisData.texto) }}
                  sx={{
                    color: "#1e293b",
                    fontSize: "0.92rem",
                    lineHeight: 1.6,
                    "& strong": { fontWeight: 700 },
                    "& em": { fontStyle: "italic" },
                  }}
                />
              </Paper>
            </Box>
          ) : null}
        </DialogContent>
        <DialogActions sx={{ p: 2, borderTop: "1px solid #e2e8f0", justifyContent: "space-between" }}>
          {reviewSintesisData?.fecha && (
            <Button
              color="error"
              variant="outlined"
              startIcon={<DeleteOutlineIcon />}
              onClick={() => handlePromptDeleteSintesis(reviewSintesisData.fecha)}
              sx={{ textTransform: "none", fontWeight: 600 }}
            >
              Eliminar Síntesis
            </Button>
          )}
          <Box sx={{ display: "flex", gap: 1 }}>
            <Button onClick={() => setReviewModalOpen(false)} sx={{ textTransform: "none" }}>
              Cerrar
            </Button>
            {reviewSintesisData?.fecha && (
              <Button
                variant="contained"
                startIcon={<FolderOpenIcon />}
                onClick={() => {
                  handleCambiarFecha(reviewSintesisData.fecha);
                  setReviewModalOpen(false);
                  if (workspaceSectionRef.current) {
                    workspaceSectionRef.current.scrollIntoView({ behavior: "smooth" });
                  }
                }}
                sx={{ textTransform: "none", fontWeight: 600 }}
              >
                Cargar en Editor y Trabajar
              </Button>
            )}
          </Box>
        </DialogActions>
      </Dialog>

      {/* ===================================================================== */}
      {/* DIÁLOGO: CONFIRMAR ELIMINACIÓN DE SÍNTESIS / FECHA                    */}
      {/* ===================================================================== */}
      <Dialog
        open={deleteConfirmOpen}
        onClose={() => !deletingSintesis && setDeleteConfirmOpen(false)}
        maxWidth="xs"
        fullWidth
        PaperProps={{ sx: { borderRadius: "12px" } }}
      >
        <DialogTitle sx={{ fontWeight: 700, color: "#b91c1c" }}>
          Eliminar Registro / Síntesis del Día
        </DialogTitle>
        <DialogContent>
          <Typography variant="body2" sx={{ color: "#475569" }}>
            ¿Está seguro de que desea eliminar el registro de la fecha <strong>{dateToDelete}</strong>?
          </Typography>
          <Box sx={{ mt: 2, p: 1.5, borderRadius: "8px", backgroundColor: "#f8fafc", border: "1px solid #e2e8f0" }}>
            <FormControlLabel
              control={
                <Checkbox
                  checked={deleteDateWithDocs}
                  onChange={(e) => setDeleteDateWithDocs(e.target.checked)}
                  color="error"
                  size="small"
                />
              }
              label={
                <Typography variant="body2" sx={{ fontWeight: 600, color: "#334155" }}>
                  Eliminar también los documentos PDF de este día (limpiar fecha por completo)
                </Typography>
              }
            />
            <Typography variant="caption" sx={{ color: "#64748b", display: "block", pl: 3.8 }}>
              {deleteDateWithDocs
                ? "Se eliminarán los archivos y la fecha desaparecerá completamente del historial."
                : "Solo se eliminará la síntesis generada; los documentos permanecerán en el universo."}
            </Typography>
          </Box>
        </DialogContent>
        <DialogActions sx={{ p: 2 }}>
          <Button
            onClick={() => setDeleteConfirmOpen(false)}
            disabled={deletingSintesis}
            sx={{ textTransform: "none" }}
          >
            Cancelar
          </Button>
          <Button
            variant="contained"
            color="error"
            startIcon={deletingSintesis ? <CircularProgress size={16} color="inherit" /> : <DeleteOutlineIcon />}
            onClick={handleConfirmDeleteSintesis}
            disabled={deletingSintesis}
            sx={{ textTransform: "none", fontWeight: 700 }}
          >
            {deletingSintesis ? "Eliminando..." : "Sí, Eliminar"}
          </Button>
        </DialogActions>
      </Dialog>

      {/* ===================================================================== */}
      {/* DIÁLOGO: CONFIRMAR ELIMINACIÓN DE DOCUMENTO INDIVIDUAL                */}
      {/* ===================================================================== */}
      <Dialog
        open={deleteDocModalOpen}
        onClose={() => !deletingDoc && setDeleteDocModalOpen(false)}
        maxWidth="xs"
        fullWidth
        PaperProps={{ sx: { borderRadius: "12px" } }}
      >
        <DialogTitle sx={{ fontWeight: 700, color: "#b91c1c" }}>
          Eliminar Documento
        </DialogTitle>
        <DialogContent>
          <Typography variant="body2" sx={{ color: "#475569" }}>
            ¿Está seguro de que desea eliminar el documento <strong>{docToDelete?.nombre}</strong>?
          </Typography>
          <Typography variant="caption" sx={{ color: "#64748b", display: "block", mt: 1.5 }}>
            Se eliminará el archivo en disco, sus secciones extraídas y su asociación con la síntesis del día. Esta acción no se puede deshacer.
          </Typography>
        </DialogContent>
        <DialogActions sx={{ p: 2 }}>
          <Button
            onClick={() => setDeleteDocModalOpen(false)}
            disabled={deletingDoc}
            sx={{ textTransform: "none" }}
          >
            Cancelar
          </Button>
          <Button
            variant="contained"
            color="error"
            startIcon={deletingDoc ? <CircularProgress size={16} color="inherit" /> : <DeleteOutlineIcon />}
            onClick={handleConfirmDeleteDoc}
            disabled={deletingDoc}
            sx={{ textTransform: "none", fontWeight: 700 }}
          >
            {deletingDoc ? "Eliminando..." : "Sí, Eliminar Documento"}
          </Button>
        </DialogActions>
      </Dialog>
    </Box>
  );
}
