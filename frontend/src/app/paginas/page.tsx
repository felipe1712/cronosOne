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
  Tabs,
  Tab,
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
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  TextField,
  Switch,
  FormControlLabel,
  IconButton,
  Tooltip,
  Snackbar,
  MenuItem,
  Select,
  FormControl,
  InputLabel,
} from "@mui/material";
import AddIcon from "@mui/icons-material/Add";
import EditIcon from "@mui/icons-material/Edit";
import DeleteOutlineIcon from "@mui/icons-material/DeleteOutline";
import RefreshIcon from "@mui/icons-material/Refresh";
import LaunchIcon from "@mui/icons-material/Launch";
import ContentCopyIcon from "@mui/icons-material/ContentCopy";
import SendIcon from "@mui/icons-material/Send";
import CheckCircleOutlineIcon from "@mui/icons-material/CheckCircleOutline";
import HourglassEmptyIcon from "@mui/icons-material/HourglassEmpty";
import ErrorOutlineIcon from "@mui/icons-material/ErrorOutline";
import SyncIcon from "@mui/icons-material/Sync";
import WhatsAppIcon from "@mui/icons-material/WhatsApp";
import ArticleIcon from "@mui/icons-material/Article";
import PeopleIcon from "@mui/icons-material/People";
import CampaignIcon from "@mui/icons-material/Campaign";
import WebIcon from "@mui/icons-material/Web";
import SaveIcon from "@mui/icons-material/Save";
import { ApiService } from "@/lib/api";

export default function CmsManagementPage() {
  const [currentTab, setCurrentTab] = useState<number>(0);
  const [loading, setLoading] = useState<boolean>(true);
  const [snackbar, setSnackbar] = useState<{ open: boolean; message: string; severity: "success" | "error" | "info" }>({
    open: false,
    message: "",
    severity: "info",
  });

  // Tab 0: Pages & Sections
  const [pages, setPages] = useState<any[]>([]);
  const [pageDialogOpen, setPageDialogOpen] = useState<boolean>(false);
  const [editingPage, setEditingPage] = useState<any | null>(null);
  const [pageForm, setPageForm] = useState({
    section_key: "",
    title: "",
    subtitle: "",
    color_hex: "#605DFF",
    icon_name: "article",
    orden: 0,
    is_active: true,
  });

  // Tab 1: Editions & Content
  const [editions, setEditions] = useState<any[]>([]);
  const [selectedEdition, setSelectedEdition] = useState<any | null>(null);
  const [editingSection, setEditingSection] = useState<any | null>(null);
  const [sectionDialogOpen, setSectionDialogOpen] = useState<boolean>(false);
  const [sectionForm, setSectionForm] = useState({
    title: "",
    summary: "",
    full_content: "",
    published: true,
  });
  const [publishing, setPublishing] = useState<boolean>(false);

  // Tab 2: Subscribers
  const [subscribers, setSubscribers] = useState<any[]>([]);
  const [subscriberStats, setSubscriberStats] = useState({ total: 0, subscribed: 0, unsubscribed: 0 });
  const [subFilter, setSubFilter] = useState<string>("");
  const [subDialogOpen, setSubDialogOpen] = useState<boolean>(false);
  const [subForm, setSubForm] = useState({
    phone_number: "",
    display_name: "",
    consent_source: "panel_admin",
    consent_text_version: "whatsapp_news_v1",
    consent_status: "subscribed",
  });

  // Tab 3: Broadcasts
  const [broadcasts, setBroadcasts] = useState<any[]>([]);
  const [broadcastDialogOpen, setBroadcastDialogOpen] = useState<boolean>(false);
  const [broadcastForm, setBroadcastForm] = useState({
    edition_date: "",
    send_immediately: false,
    test_phone: "",
  });
  const [triggering, setTriggering] = useState<boolean>(false);
  const [reconcilingId, setReconcilingId] = useState<string | null>(null);

  const showToast = (message: string, severity: "success" | "error" | "info" = "info") => {
    setSnackbar({ open: true, message, severity });
  };

  const loadAllData = async () => {
    try {
      setLoading(true);
      const [pagesRes, editionsRes, subsRes, broadcastsRes] = await Promise.all([
        ApiService.listCmsPages().catch(() => []),
        ApiService.listCmsEditions().catch(() => []),
        ApiService.listSubscribers().catch(() => ({ subscribers: [], stats: { total: 0, subscribed: 0, unsubscribed: 0 } })),
        ApiService.listBroadcasts().catch(() => []),
      ]);

      setPages(pagesRes || []);
      setEditions(editionsRes || []);
      if (editionsRes && editionsRes.length > 0) {
        setSelectedEdition(editionsRes[0]);
        setBroadcastForm((prev) => ({ ...prev, edition_date: editionsRes[0].edition_date }));
      }
      setSubscribers(subsRes?.subscribers || []);
      setSubscriberStats(subsRes?.stats || { total: 0, subscribed: 0, unsubscribed: 0 });
      setBroadcasts(broadcastsRes || []);
    } catch (err: any) {
      showToast(err.message || "Error al cargar datos del CMS", "error");
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadAllData();
  }, []);

  // ---------------------------------------------------------------------------
  // TAB 0: PAGES HANDLERS
  // ---------------------------------------------------------------------------
  const handleOpenPageDialog = (page?: any) => {
    if (page) {
      setEditingPage(page);
      setPageForm({
        section_key: page.section_key,
        title: page.title,
        subtitle: page.subtitle || "",
        color_hex: page.color_hex || "#605DFF",
        icon_name: page.icon_name || "article",
        orden: page.orden || 0,
        is_active: page.is_active,
      });
    } else {
      setEditingPage(null);
      setPageForm({
        section_key: "",
        title: "",
        subtitle: "",
        color_hex: "#605DFF",
        icon_name: "article",
        orden: pages.length * 10,
        is_active: true,
      });
    }
    setPageDialogOpen(true);
  };

  const handleSavePage = async () => {
    try {
      if (!pageForm.section_key.trim() || !pageForm.title.trim()) {
        showToast("El slug y el título son obligatorios", "error");
        return;
      }
      if (editingPage) {
        await ApiService.updateCmsPage(editingPage.id, pageForm);
        showToast("Página actualizada con éxito", "success");
      } else {
        await ApiService.createCmsPage(pageForm);
        showToast("Página creada con éxito", "success");
      }
      setPageDialogOpen(false);
      const updated = await ApiService.listCmsPages();
      setPages(updated || []);
    } catch (err: any) {
      showToast(err.message || "Error al guardar página", "error");
    }
  };

  const handleTogglePage = async (id: string) => {
    try {
      await ApiService.toggleCmsPage(id);
      showToast("Estado de página actualizado", "success");
      const updated = await ApiService.listCmsPages();
      setPages(updated || []);
    } catch (err: any) {
      showToast(err.message || "Error al cambiar estado", "error");
    }
  };

  const handleDeletePage = async (id: string, title: string) => {
    if (!window.confirm(`¿Estás seguro de desactivar o dar de baja la página "${title}"?`)) return;
    try {
      await ApiService.deleteCmsPage(id);
      showToast("Página dada de baja", "success");
      const updated = await ApiService.listCmsPages();
      setPages(updated || []);
    } catch (err: any) {
      showToast(err.message || "Error al eliminar página", "error");
    }
  };

  // ---------------------------------------------------------------------------
  // TAB 1: EDITIONS & SECTIONS HANDLERS
  // ---------------------------------------------------------------------------
  const handleSelectEdition = async (fecha: string) => {
    try {
      const edition = await ApiService.getCmsEdition(fecha);
      setSelectedEdition(edition);
      setBroadcastForm((prev) => ({ ...prev, edition_date: fecha }));
    } catch (err: any) {
      showToast(err.message || "Error al obtener edición", "error");
    }
  };

  const handleOpenSectionDialog = (sec: any) => {
    setEditingSection(sec);
    setSectionForm({
      title: sec.title || "",
      summary: sec.summary || "",
      full_content: sec.full_content || "",
      published: sec.published,
    });
    setSectionDialogOpen(true);
  };

  const handleSaveSection = async () => {
    if (!selectedEdition || !editingSection) return;
    try {
      await ApiService.updateCmsSection(selectedEdition.edition_date, editingSection.section_key, sectionForm);
      showToast("Sección actualizada correctamente", "success");
      setSectionDialogOpen(false);
      const refreshed = await ApiService.getCmsEdition(selectedEdition.edition_date);
      setSelectedEdition(refreshed);
    } catch (err: any) {
      showToast(err.message || "Error al actualizar sección", "error");
    }
  };

  const handlePublishEdition = async (sendBroadcast: boolean) => {
    if (!selectedEdition) return;
    if (sendBroadcast && !window.confirm(`¿Publicar edición ${selectedEdition.edition_date} y despachar broadcast de WhatsApp ahora a todos los suscriptores activos?`)) {
      return;
    }
    try {
      setPublishing(true);
      const res = await ApiService.publishCmsEdition(selectedEdition.edition_date, sendBroadcast);
      showToast(res.mensaje || "Edición publicada con éxito", "success");
      const [refreshedEd, bcasts] = await Promise.all([
        ApiService.getCmsEdition(selectedEdition.edition_date),
        ApiService.listBroadcasts(),
      ]);
      setSelectedEdition(refreshedEd);
      setBroadcasts(bcasts || []);
      const eds = await ApiService.listCmsEditions();
      setEditions(eds || []);
    } catch (err: any) {
      showToast(err.message || "Error al publicar edición", "error");
    } finally {
      setPublishing(false);
    }
  };

  // ---------------------------------------------------------------------------
  // TAB 2: SUBSCRIBERS HANDLERS
  // ---------------------------------------------------------------------------
  const handleSaveSubscriber = async () => {
    try {
      if (!subForm.phone_number.trim() || !subForm.display_name.trim()) {
        showToast("Teléfono y Nombre son obligatorios", "error");
        return;
      }
      await ApiService.createSubscriber(subForm);
      showToast("Suscriptor registrado exitosamente", "success");
      setSubDialogOpen(false);
      setSubForm({
        phone_number: "",
        display_name: "",
        consent_source: "panel_admin",
        consent_text_version: "whatsapp_news_v1",
        consent_status: "subscribed",
      });
      const res = await ApiService.listSubscribers();
      setSubscribers(res?.subscribers || []);
      setSubscriberStats(res?.stats || { total: 0, subscribed: 0, unsubscribed: 0 });
    } catch (err: any) {
      showToast(err.message || "Error al guardar suscriptor", "error");
    }
  };

  const handleToggleSubscriber = async (id: string) => {
    try {
      await ApiService.toggleSubscriber(id);
      showToast("Estado de suscripción actualizado", "success");
      const res = await ApiService.listSubscribers();
      setSubscribers(res?.subscribers || []);
      setSubscriberStats(res?.stats || { total: 0, subscribed: 0, unsubscribed: 0 });
    } catch (err: any) {
      showToast(err.message || "Error al actualizar estado", "error");
    }
  };

  // ---------------------------------------------------------------------------
  // TAB 3: BROADCASTS HANDLERS
  // ---------------------------------------------------------------------------
  const handleTriggerBroadcast = async () => {
    try {
      setTriggering(true);
      const res = await ApiService.triggerBroadcast({
        edition_date: broadcastForm.edition_date,
        send_immediately: broadcastForm.send_immediately,
        test_phone: broadcastForm.test_phone.trim() || undefined,
      });
      showToast(res.mensaje || "Campaña creada en Kapso con éxito", "success");
      setBroadcastDialogOpen(false);
      const bcasts = await ApiService.listBroadcasts();
      setBroadcasts(bcasts || []);
    } catch (err: any) {
      showToast(err.message || "Error al disparar broadcast en Kapso", "error");
    } finally {
      setTriggering(false);
    }
  };

  const handleReconcile = async (id: string) => {
    try {
      setReconcilingId(id);
      const res = await ApiService.reconcileBroadcast(id);
      showToast(res.mensaje || "Métricas reconciliadas con Kapso", "success");
      const bcasts = await ApiService.listBroadcasts();
      setBroadcasts(bcasts || []);
    } catch (err: any) {
      showToast(err.message || "Error al reconciliar con Kapso", "error");
    } finally {
      setReconcilingId(null);
    }
  };

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text);
    showToast("Enlace copiado al portapapeles: " + text, "success");
  };

  const filteredSubscribers = subscribers.filter((s) => {
    if (!subFilter.trim()) return true;
    const q = subFilter.toLowerCase();
    return (
      (s.display_name && s.display_name.toLowerCase().includes(q)) ||
      (s.phone_number && s.phone_number.includes(q))
    );
  });

  return (
    <Box sx={{ p: { xs: 2, md: 4 } }}>
      {/* Header */}
      <Box sx={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start", mb: 3, flexWrap: "wrap", gap: 2 }}>
        <Box>
          <Typography variant="h4" sx={{ fontWeight: 800, color: "#1e293b", letterSpacing: "-0.5px" }}>
            Gestor de Páginas & Canales CMS
          </Typography>
          <Typography variant="body2" sx={{ color: "#64748b", mt: 0.5 }}>
            Administración del feed de briefs informativos, URLs públicas estables, audiencia WhatsApp y despachos vía Kapso
          </Typography>
        </Box>
        <Box sx={{ display: "flex", gap: 1.5 }}>
          <Button
            variant="outlined"
            startIcon={<LaunchIcon />}
            href="/brief/politica"
            target="_blank"
            sx={{ textTransform: "none", borderColor: "#cbd5e1", color: "#334155" }}
          >
            Ver Portal Público
          </Button>
          <Button
            variant="contained"
            startIcon={<RefreshIcon />}
            onClick={loadAllData}
            disabled={loading}
            sx={{ textTransform: "none", bgcolor: "#605DFF", "&:hover": { bgcolor: "#4d4ad6" } }}
          >
            Refrescar
          </Button>
        </Box>
      </Box>

      {/* Tabs */}
      <Card sx={{ borderRadius: 3, mb: 3, boxShadow: "0 4px 20px rgba(0,0,0,0.04)", border: "1px solid #e2e8f0" }}>
        <Tabs
          value={currentTab}
          onChange={(_, v) => setCurrentTab(v)}
          variant="scrollable"
          scrollButtons="auto"
          sx={{
            px: 2,
            pt: 1,
            borderBottom: "1px solid #e2e8f0",
            "& .MuiTab-root": { textTransform: "none", fontWeight: 600, fontSize: "0.95rem", minHeight: 48 },
            "& .Mui-selected": { color: "#605DFF" },
            "& .MuiTabs-indicator": { backgroundColor: "#605DFF", height: 3 },
          }}
        >
          <Tab icon={<WebIcon sx={{ fontSize: 20 }} />} iconPosition="start" label="Páginas & Secciones" />
          <Tab icon={<ArticleIcon sx={{ fontSize: 20 }} />} iconPosition="start" label="Ediciones & Contenido" />
          <Tab icon={<PeopleIcon sx={{ fontSize: 20 }} />} iconPosition="start" label={`Audiencia WhatsApp (${subscriberStats.subscribed})`} />
          <Tab icon={<CampaignIcon sx={{ fontSize: 20 }} />} iconPosition="start" label="Campañas Kapso Broadcast" />
        </Tabs>
      </Card>

      {loading ? (
        <Box sx={{ display: "flex", justifyContent: "center", py: 8 }}>
          <CircularProgress sx={{ color: "#605DFF" }} />
        </Box>
      ) : (
        <>
          {/* ================================================================= */}
          {/* TAB 0: PAGINAS & SECCIONES                                         */}
          {/* ================================================================= */}
          {currentTab === 0 && (
            <Card sx={{ borderRadius: 3, boxShadow: "0 4px 20px rgba(0,0,0,0.04)", border: "1px solid #e2e8f0" }}>
              <CardContent sx={{ p: 3 }}>
                <Box sx={{ display: "flex", justifyContent: "space-between", alignItems: "center", mb: 2 }}>
                  <Box>
                    <Typography variant="h6" sx={{ fontWeight: 700, color: "#1e293b" }}>
                      Canales y Secciones Activas
                    </Typography>
                    <Typography variant="body2" sx={{ color: "#64748b" }}>
                      Cada sección tiene una URL pública fija y permanente optimizada para botones de WhatsApp.
                    </Typography>
                  </Box>
                  <Button
                    variant="contained"
                    startIcon={<AddIcon />}
                    onClick={() => handleOpenPageDialog()}
                    sx={{ textTransform: "none", bgcolor: "#605DFF", "&:hover": { bgcolor: "#4d4ad6" } }}
                  >
                    Nueva Sección / Canal
                  </Button>
                </Box>

                <TableContainer component={Paper} elevation={0} sx={{ border: "1px solid #f1f5f9", borderRadius: 2 }}>
                  <Table>
                    <TableHead sx={{ bgcolor: "#f8fafc" }}>
                      <TableRow>
                        <TableCell sx={{ fontWeight: 700, color: "#475569" }}>Sección & Título</TableCell>
                        <TableCell sx={{ fontWeight: 700, color: "#475569" }}>URL Pública Estable</TableCell>
                        <TableCell sx={{ fontWeight: 700, color: "#475569" }}>Color / Ícono</TableCell>
                        <TableCell sx={{ fontWeight: 700, color: "#475569" }}>Orden</TableCell>
                        <TableCell sx={{ fontWeight: 700, color: "#475569" }}>Estado</TableCell>
                        <TableCell align="right" sx={{ fontWeight: 700, color: "#475569" }}>Acciones</TableCell>
                      </TableRow>
                    </TableHead>
                    <TableBody>
                      {pages.map((p) => {
                        const stableUrl = `https://monitoreo.causer.com.mx/brief/${p.section_key}`;
                        return (
                          <TableRow key={p.id} hover>
                            <TableCell>
                              <Box sx={{ display: "flex", alignItems: "center", gap: 1.5 }}>
                                <Box
                                  sx={{
                                    width: 36,
                                    height: 36,
                                    borderRadius: 2,
                                    bgcolor: (p.color_hex || "#605DFF") + "15",
                                    color: p.color_hex || "#605DFF",
                                    display: "flex",
                                    alignItems: "center",
                                    justifyContent: "center",
                                    fontWeight: 700,
                                  }}
                                >
                                  <i className="material-symbols-outlined" style={{ fontSize: 20 }}>
                                    {p.icon_name || "article"}
                                  </i>
                                </Box>
                                <Box>
                                  <Typography variant="subtitle2" sx={{ fontWeight: 700, color: "#0f172a" }}>
                                    {p.title}
                                  </Typography>
                                  <Typography variant="caption" sx={{ color: "#64748b" }}>
                                    {p.subtitle || `Sección ${p.section_key}`}
                                  </Typography>
                                </Box>
                              </Box>
                            </TableCell>
                            <TableCell>
                              <Box sx={{ display: "flex", alignItems: "center", gap: 0.5 }}>
                                <Typography
                                  variant="body2"
                                  sx={{
                                    fontFamily: "monospace",
                                    bgcolor: "#f1f5f9",
                                    px: 1,
                                    py: 0.5,
                                    borderRadius: 1,
                                    fontSize: "0.8rem",
                                    color: "#334155",
                                  }}
                                >
                                  /brief/{p.section_key}
                                </Typography>
                                <Tooltip title="Copiar URL completa">
                                  <IconButton size="small" onClick={() => copyToClipboard(stableUrl)}>
                                    <ContentCopyIcon sx={{ fontSize: 16 }} />
                                  </IconButton>
                                </Tooltip>
                                <Tooltip title="Abrir en pestaña nueva">
                                  <IconButton size="small" component={Link} href={`/brief/${p.section_key}`} target="_blank">
                                    <LaunchIcon sx={{ fontSize: 16 }} />
                                  </IconButton>
                                </Tooltip>
                              </Box>
                            </TableCell>
                            <TableCell>
                              <Box sx={{ display: "flex", alignItems: "center", gap: 1 }}>
                                <Box
                                  sx={{
                                    width: 16,
                                    height: 16,
                                    borderRadius: "50%",
                                    bgcolor: p.color_hex || "#605DFF",
                                    border: "2px solid #fff",
                                    boxShadow: "0 0 0 1px #cbd5e1",
                                  }}
                                />
                                <Typography variant="caption" sx={{ fontFamily: "monospace", color: "#64748b" }}>
                                  {p.color_hex}
                                </Typography>
                              </Box>
                            </TableCell>
                            <TableCell>
                              <Typography variant="body2" sx={{ fontWeight: 600 }}>
                                {p.orden}
                              </Typography>
                            </TableCell>
                            <TableCell>
                              <Chip
                                label={p.is_active ? "Activa" : "Inactiva"}
                                size="small"
                                sx={{
                                  bgcolor: p.is_active ? "#dcfce7" : "#f1f5f9",
                                  color: p.is_active ? "#15803d" : "#64748b",
                                  fontWeight: 600,
                                }}
                              />
                            </TableCell>
                            <TableCell align="right">
                              <Box sx={{ display: "flex", justifyContent: "flex-end", alignItems: "center", gap: 0.5 }}>
                                <Tooltip title={p.is_active ? "Desactivar" : "Activar"}>
                                  <Switch
                                    checked={p.is_active}
                                    onChange={() => handleTogglePage(p.id)}
                                    color="primary"
                                    size="small"
                                  />
                                </Tooltip>
                                <Tooltip title="Editar detalles">
                                  <IconButton size="small" onClick={() => handleOpenPageDialog(p)}>
                                    <EditIcon sx={{ fontSize: 18 }} />
                                  </IconButton>
                                </Tooltip>
                                <Tooltip title="Dar de baja">
                                  <IconButton
                                    size="small"
                                    color="error"
                                    onClick={() => handleDeletePage(p.id, p.title)}
                                  >
                                    <DeleteOutlineIcon sx={{ fontSize: 18 }} />
                                  </IconButton>
                                </Tooltip>
                              </Box>
                            </TableCell>
                          </TableRow>
                        );
                      })}
                    </TableBody>
                  </Table>
                </TableContainer>
              </CardContent>
            </Card>
          )}

          {/* ================================================================= */}
          {/* TAB 1: EDICIONES & CONTENIDO                                       */}
          {/* ================================================================= */}
          {currentTab === 1 && (
            <Grid container spacing={3}>
              {/* Editions List */}
              <Grid size={{ xs: 12, md: 4 }}>
                <Card sx={{ borderRadius: 3, boxShadow: "0 4px 20px rgba(0,0,0,0.04)", border: "1px solid #e2e8f0" }}>
                  <CardContent sx={{ p: 2.5 }}>
                    <Typography variant="h6" sx={{ fontWeight: 700, color: "#1e293b", mb: 2 }}>
                      Ediciones por Fecha
                    </Typography>
                    {editions.length === 0 ? (
                      <Typography variant="body2" sx={{ color: "#94a3b8", textAlign: "center", py: 3 }}>
                        No hay ediciones generadas aún.
                      </Typography>
                    ) : (
                      <Box sx={{ display: "flex", flexDirection: "column", gap: 1 }}>
                        {editions.map((ed) => {
                          const isSelected = selectedEdition?.edition_date === ed.edition_date;
                          return (
                            <Box
                              key={ed.id}
                              onClick={() => handleSelectEdition(ed.edition_date)}
                              sx={{
                                p: 1.5,
                                borderRadius: 2,
                                cursor: "pointer",
                                border: isSelected ? "2px solid #605DFF" : "1px solid #f1f5f9",
                                bgcolor: isSelected ? "#f5f3ff" : "#fff",
                                "&:hover": { bgcolor: "#f8fafc" },
                                transition: "all 0.15s ease",
                              }}
                            >
                              <Box sx={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                                <Typography variant="subtitle2" sx={{ fontWeight: 700, color: "#0f172a" }}>
                                  📅 {ed.edition_date}
                                </Typography>
                                <Chip
                                  label={ed.status === "published" ? "Publicada" : "Borrador"}
                                  size="small"
                                  sx={{
                                    bgcolor: ed.status === "published" ? "#dcfce7" : "#fef9c3",
                                    color: ed.status === "published" ? "#15803d" : "#854d0e",
                                    fontWeight: 600,
                                    fontSize: "0.7rem",
                                    height: 20,
                                  }}
                                />
                              </Box>
                              <Typography variant="caption" sx={{ color: "#64748b", display: "block", mt: 0.5 }}>
                                Generada: {ed.generated_at ? new Date(ed.generated_at).toLocaleTimeString("es-MX", { hour: "2-digit", minute: "2-digit" }) : "Auto"}
                              </Typography>
                            </Box>
                          );
                        })}
                      </Box>
                    )}
                  </CardContent>
                </Card>
              </Grid>

              {/* Selected Edition Sections */}
              <Grid size={{ xs: 12, md: 8 }}>
                {selectedEdition ? (
                  <Card sx={{ borderRadius: 3, boxShadow: "0 4px 20px rgba(0,0,0,0.04)", border: "1px solid #e2e8f0" }}>
                    <CardContent sx={{ p: 3 }}>
                      <Box sx={{ display: "flex", justifyContent: "space-between", alignItems: "center", mb: 3, flexWrap: "wrap", gap: 1.5 }}>
                        <Box>
                          <Typography variant="h5" sx={{ fontWeight: 800, color: "#1e293b" }}>
                            Edición: {selectedEdition.edition_date}
                          </Typography>
                          <Typography variant="body2" sx={{ color: "#64748b" }}>
                            Estado actual:{" "}
                            <strong style={{ color: selectedEdition.status === "published" ? "#16a34a" : "#ca8a04" }}>
                              {selectedEdition.status === "published" ? "PUBLICADA EN LÍNEA" : "BORRADOR"}
                            </strong>
                          </Typography>
                        </Box>
                        <Box sx={{ display: "flex", gap: 1 }}>
                          <Button
                            variant="outlined"
                            onClick={() => handlePublishEdition(false)}
                            disabled={publishing}
                            sx={{ textTransform: "none" }}
                          >
                            Solo Publicar en Web
                          </Button>
                          <Button
                            variant="contained"
                            startIcon={<SendIcon />}
                            onClick={() => handlePublishEdition(true)}
                            disabled={publishing}
                            sx={{ textTransform: "none", bgcolor: "#16a34a", "&:hover": { bgcolor: "#15803d" } }}
                          >
                            Publicar y Despachar WhatsApp
                          </Button>
                        </Box>
                      </Box>

                      <Divider sx={{ mb: 3 }} />

                      <Typography variant="h6" sx={{ fontWeight: 700, mb: 2, color: "#334155" }}>
                        Secciones del Brief Diario
                      </Typography>

                      <Box sx={{ display: "flex", flexDirection: "column", gap: 2 }}>
                        {selectedEdition.sections && selectedEdition.sections.length > 0 ? (
                          selectedEdition.sections.map((sec: any) => (
                            <Box
                              key={sec.id || sec.section_key}
                              sx={{
                                p: 2,
                                borderRadius: 2,
                                border: "1px solid #e2e8f0",
                                bgcolor: "#f8fafc",
                              }}
                            >
                              <Box sx={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start", mb: 1 }}>
                                <Box>
                                  <Box sx={{ display: "flex", alignItems: "center", gap: 1 }}>
                                    <Chip
                                      label={sec.section_key.toUpperCase()}
                                      size="small"
                                      sx={{ bgcolor: "#605DFF", color: "#fff", fontWeight: 700, fontSize: "0.7rem" }}
                                    />
                                    <Typography variant="subtitle1" sx={{ fontWeight: 700, color: "#0f172a" }}>
                                      {sec.title}
                                    </Typography>
                                  </Box>
                                  <Typography variant="body2" sx={{ color: "#475569", mt: 0.5, fontStyle: "italic" }}>
                                    {sec.summary}
                                  </Typography>
                                </Box>
                                <Box sx={{ display: "flex", gap: 0.5 }}>
                                  <Button
                                    size="small"
                                    variant="outlined"
                                    startIcon={<EditIcon sx={{ fontSize: 15 }} />}
                                    onClick={() => handleOpenSectionDialog(sec)}
                                    sx={{ textTransform: "none" }}
                                  >
                                    Editar
                                  </Button>
                                  <Button
                                    size="small"
                                    variant="text"
                                    startIcon={<LaunchIcon sx={{ fontSize: 15 }} />}
                                    component={Link}
                                    href={`/brief/${selectedEdition.edition_date}/${sec.section_key}`}
                                    target="_blank"
                                    sx={{ textTransform: "none" }}
                                  >
                                    Ver Web
                                  </Button>
                                </Box>
                              </Box>

                              <Box
                                sx={{
                                  maxHeight: 140,
                                  overflowY: "auto",
                                  p: 1.5,
                                  bgcolor: "#fff",
                                  borderRadius: 1.5,
                                  border: "1px solid #e2e8f0",
                                  fontSize: "0.85rem",
                                  color: "#334155",
                                  whiteSpace: "pre-line",
                                }}
                              >
                                {sec.full_content || "Sin contenido disponible."}
                              </Box>
                            </Box>
                          ))
                        ) : (
                          <Alert severity="warning">
                            Esta edición no cuenta con secciones generadas. Revisa el worker de síntesis.
                          </Alert>
                        )}
                      </Box>
                    </CardContent>
                  </Card>
                ) : (
                  <Card sx={{ borderRadius: 3, p: 4, textAlign: "center", border: "1px solid #e2e8f0" }}>
                    <Typography variant="body1" sx={{ color: "#94a3b8" }}>
                      Selecciona una edición a la izquierda para inspeccionar y publicar sus secciones.
                    </Typography>
                  </Card>
                )}
              </Grid>
            </Grid>
          )}

          {/* ================================================================= */}
          {/* TAB 2: AUDIENCIA & SUSCRIPTORES                                   */}
          {/* ================================================================= */}
          {currentTab === 2 && (
            <Card sx={{ borderRadius: 3, boxShadow: "0 4px 20px rgba(0,0,0,0.04)", border: "1px solid #e2e8f0" }}>
              <CardContent sx={{ p: 3 }}>
                {/* Metric Cards */}
                <Grid container spacing={2} sx={{ mb: 3 }}>
                  <Grid size={{ xs: 12, sm: 4 }}>
                    <Paper sx={{ p: 2, borderRadius: 2, bgcolor: "#f8fafc", border: "1px solid #e2e8f0" }}>
                      <Typography variant="caption" sx={{ color: "#64748b", fontWeight: 600 }}>
                        AUDIENCIA TOTAL REGISTRADA
                      </Typography>
                      <Typography variant="h4" sx={{ fontWeight: 800, color: "#1e293b", mt: 0.5 }}>
                        {subscriberStats.total}
                      </Typography>
                    </Paper>
                  </Grid>
                  <Grid size={{ xs: 12, sm: 4 }}>
                    <Paper sx={{ p: 2, borderRadius: 2, bgcolor: "#f0fdf4", border: "1px solid #bbf7d0" }}>
                      <Typography variant="caption" sx={{ color: "#166534", fontWeight: 600 }}>
                        SUSCRIPCIONES ACTIVAS (OPT-IN)
                      </Typography>
                      <Typography variant="h4" sx={{ fontWeight: 800, color: "#15803d", mt: 0.5 }}>
                        {subscriberStats.subscribed}
                      </Typography>
                    </Paper>
                  </Grid>
                  <Grid size={{ xs: 12, sm: 4 }}>
                    <Paper sx={{ p: 2, borderRadius: 2, bgcolor: "#fef2f2", border: "1px solid #fecaca" }}>
                      <Typography variant="caption" sx={{ color: "#991b1b", fontWeight: 600 }}>
                        BAJAS / UNSUBSCRIBED (STOP)
                      </Typography>
                      <Typography variant="h4" sx={{ fontWeight: 800, color: "#b91c1c", mt: 0.5 }}>
                        {subscriberStats.unsubscribed}
                      </Typography>
                    </Paper>
                  </Grid>
                </Grid>

                <Alert severity="info" sx={{ mb: 3 }}>
                  <strong>Cumplimiento de Políticas WhatsApp & Meta:</strong> Los broadcasts solo se envían a contactos con estado <code>subscribed</code>. Si un suscriptor responde <code>STOP</code> por WhatsApp, el webhook de Kapso actualiza automáticamente su estado a <code>unsubscribed</code> para evitar bloqueos del número remitente.
                </Alert>

                <Box sx={{ display: "flex", justifyContent: "space-between", alignItems: "center", mb: 2, flexWrap: "wrap", gap: 1.5 }}>
                  <TextField
                    placeholder="Buscar por nombre o número telefónico..."
                    size="small"
                    value={subFilter}
                    onChange={(e) => setSubFilter(e.target.value)}
                    sx={{ width: { xs: "100%", sm: 350 } }}
                  />
                  <Button
                    variant="contained"
                    startIcon={<AddIcon />}
                    onClick={() => setSubDialogOpen(true)}
                    sx={{ textTransform: "none", bgcolor: "#605DFF", "&:hover": { bgcolor: "#4d4ad6" } }}
                  >
                    Registrar Suscriptor
                  </Button>
                </Box>

                <TableContainer component={Paper} elevation={0} sx={{ border: "1px solid #f1f5f9", borderRadius: 2 }}>
                  <Table>
                    <TableHead sx={{ bgcolor: "#f8fafc" }}>
                      <TableRow>
                        <TableCell sx={{ fontWeight: 700, color: "#475569" }}>Teléfono (E.164)</TableCell>
                        <TableCell sx={{ fontWeight: 700, color: "#475569" }}>Nombre</TableCell>
                        <TableCell sx={{ fontWeight: 700, color: "#475569" }}>Consentimiento</TableCell>
                        <TableCell sx={{ fontWeight: 700, color: "#475569" }}>Origen</TableCell>
                        <TableCell sx={{ fontWeight: 700, color: "#475569" }}>Fecha Consentimiento</TableCell>
                        <TableCell align="right" sx={{ fontWeight: 700, color: "#475569" }}>Acciones</TableCell>
                      </TableRow>
                    </TableHead>
                    <TableBody>
                      {filteredSubscribers.length === 0 ? (
                        <TableRow>
                          <TableCell colSpan={6} align="center" sx={{ py: 3, color: "#94a3b8" }}>
                            No se encontraron suscriptores con el criterio indicado.
                          </TableCell>
                        </TableRow>
                      ) : (
                        filteredSubscribers.map((s) => (
                          <TableRow key={s.id} hover>
                            <TableCell sx={{ fontFamily: "monospace", fontWeight: 600 }}>
                              {s.phone_number}
                            </TableCell>
                            <TableCell sx={{ fontWeight: 600, color: "#0f172a" }}>
                              {s.display_name || "Sin nombre"}
                            </TableCell>
                            <TableCell>
                              <Chip
                                label={s.consent_status.toUpperCase()}
                                size="small"
                                sx={{
                                  bgcolor: s.consent_status === "subscribed" ? "#dcfce7" : "#fee2e2",
                                  color: s.consent_status === "subscribed" ? "#15803d" : "#991b1b",
                                  fontWeight: 700,
                                  fontSize: "0.7rem",
                                }}
                              />
                            </TableCell>
                            <TableCell>
                              <Typography variant="body2" sx={{ color: "#64748b" }}>
                                {s.consent_source || "directo"}
                              </Typography>
                            </TableCell>
                            <TableCell>
                              <Typography variant="caption" sx={{ color: "#64748b" }}>
                                {s.consented_at ? new Date(s.consented_at).toLocaleDateString("es-MX") : "N/D"}
                              </Typography>
                            </TableCell>
                            <TableCell align="right">
                              <Button
                                size="small"
                                variant={s.consent_status === "subscribed" ? "outlined" : "contained"}
                                color={s.consent_status === "subscribed" ? "error" : "success"}
                                onClick={() => handleToggleSubscriber(s.id)}
                                sx={{ textTransform: "none", fontSize: "0.75rem" }}
                              >
                                {s.consent_status === "subscribed" ? "Dar de Baja" : "Reactivar"}
                              </Button>
                            </TableCell>
                          </TableRow>
                        ))
                      )}
                    </TableBody>
                  </Table>
                </TableContainer>
              </CardContent>
            </Card>
          )}

          {/* ================================================================= */}
          {/* TAB 3: CAMPAÑAS WHATSAPP (KAPSO BROADCASTS)                        */}
          {/* ================================================================= */}
          {currentTab === 3 && (
            <Card sx={{ borderRadius: 3, boxShadow: "0 4px 20px rgba(0,0,0,0.04)", border: "1px solid #e2e8f0" }}>
              <CardContent sx={{ p: 3 }}>
                <Box sx={{ display: "flex", justifyContent: "space-between", alignItems: "center", mb: 3, flexWrap: "wrap", gap: 1.5 }}>
                  <Box>
                    <Typography variant="h6" sx={{ fontWeight: 700, color: "#1e293b" }}>
                      Campañas Broadcast Kapso Platform
                    </Typography>
                    <Typography variant="body2" sx={{ color: "#64748b" }}>
                      Envíos masivos y programados de la plantilla <code>brief_informativo_diario</code> a los suscriptores activos.
                    </Typography>
                  </Box>
                  <Button
                    variant="contained"
                    startIcon={<CampaignIcon />}
                    onClick={() => setBroadcastDialogOpen(true)}
                    sx={{ textTransform: "none", bgcolor: "#16a34a", "&:hover": { bgcolor: "#15803d" } }}
                  >
                    Disparar / Programar Campaña
                  </Button>
                </Box>

                <Alert severity="info" sx={{ mb: 3 }}>
                  <strong>Webhook de Kapso:</strong> Registra la URL <code>https://monitoreo.causer.com.mx/api/webhooks/kapso</code> en el panel de Kapso para telemetría en tiempo real y bajas automáticas de usuarios al escribir <code>STOP</code>. Puedes configurar el <em>Secret</em> en la pestaña Configuración.
                </Alert>

                <TableContainer component={Paper} elevation={0} sx={{ border: "1px solid #f1f5f9", borderRadius: 2 }}>
                  <Table>
                    <TableHead sx={{ bgcolor: "#f8fafc" }}>
                      <TableRow>
                        <TableCell sx={{ fontWeight: 700, color: "#475569" }}>Fecha Edición</TableCell>
                        <TableCell sx={{ fontWeight: 700, color: "#475569" }}>Kapso ID</TableCell>
                        <TableCell sx={{ fontWeight: 700, color: "#475569" }}>Estado</TableCell>
                        <TableCell sx={{ fontWeight: 700, color: "#475569" }}>Destinatarios</TableCell>
                        <TableCell sx={{ fontWeight: 700, color: "#475569" }}>Métricas (Env / Ent / Leí)</TableCell>
                        <TableCell sx={{ fontWeight: 700, color: "#475569" }}>Programado / Enviado</TableCell>
                        <TableCell align="right" sx={{ fontWeight: 700, color: "#475569" }}>Acciones</TableCell>
                      </TableRow>
                    </TableHead>
                    <TableBody>
                      {broadcasts.length === 0 ? (
                        <TableRow>
                          <TableCell colSpan={7} align="center" sx={{ py: 3, color: "#94a3b8" }}>
                            No hay registros de campañas broadcast aún.
                          </TableCell>
                        </TableRow>
                      ) : (
                        broadcasts.map((b) => (
                          <TableRow key={b.id} hover>
                            <TableCell sx={{ fontWeight: 700, color: "#0f172a" }}>
                              {b.edition_date || "N/A"}
                            </TableCell>
                            <TableCell sx={{ fontFamily: "monospace", fontSize: "0.8rem", color: "#64748b" }}>
                              {b.kapso_broadcast_id ? b.kapso_broadcast_id.slice(0, 16) + "..." : "Local"}
                            </TableCell>
                            <TableCell>
                              <Chip
                                label={b.status.toUpperCase()}
                                size="small"
                                sx={{
                                  bgcolor:
                                    b.status === "completed"
                                      ? "#dcfce7"
                                      : b.status === "scheduled" || b.status === "processing"
                                      ? "#e0e7ff"
                                      : b.status === "failed"
                                      ? "#fee2e2"
                                      : "#f1f5f9",
                                  color:
                                    b.status === "completed"
                                      ? "#15803d"
                                      : b.status === "scheduled" || b.status === "processing"
                                      ? "#4338ca"
                                      : b.status === "failed"
                                      ? "#991b1b"
                                      : "#475569",
                                  fontWeight: 700,
                                  fontSize: "0.7rem",
                                }}
                              />
                            </TableCell>
                            <TableCell sx={{ fontWeight: 600 }}>
                              {b.total_recipients} contactos
                            </TableCell>
                            <TableCell>
                              <Typography variant="body2" sx={{ fontWeight: 600, color: "#334155" }}>
                                {b.sent_count} / {b.delivered_count} / {b.read_count}
                              </Typography>
                              {b.failed_count > 0 && (
                                <Typography variant="caption" sx={{ color: "#ef4444", display: "block" }}>
                                  {b.failed_count} fallidos
                                </Typography>
                              )}
                            </TableCell>
                            <TableCell>
                              <Typography variant="caption" sx={{ color: "#64748b", display: "block" }}>
                                {b.scheduled_for
                                  ? "Prog: " + new Date(b.scheduled_for).toLocaleString("es-MX")
                                  : b.sent_at
                                  ? "Env: " + new Date(b.sent_at).toLocaleString("es-MX")
                                  : "Inmediato"}
                              </Typography>
                            </TableCell>
                            <TableCell align="right">
                              <Button
                                size="small"
                                variant="outlined"
                                startIcon={reconcilingId === b.id ? <CircularProgress size={12} /> : <SyncIcon sx={{ fontSize: 14 }} />}
                                onClick={() => handleReconcile(b.id)}
                                disabled={reconcilingId === b.id}
                                sx={{ textTransform: "none", fontSize: "0.75rem" }}
                              >
                                Reconciliar
                              </Button>
                            </TableCell>
                          </TableRow>
                        ))
                      )}
                    </TableBody>
                  </Table>
                </TableContainer>
              </CardContent>
            </Card>
          )}
        </>
      )}

      {/* =================================================================== */}
      {/* DIALOG: CREATE / EDIT PAGE                                         */}
      {/* =================================================================== */}
      <Dialog open={pageDialogOpen} onClose={() => setPageDialogOpen(false)} maxWidth="sm" fullWidth>
        <DialogTitle sx={{ fontWeight: 700 }}>
          {editingPage ? "Editar Canal / Sección" : "Nueva Sección de Noticias"}
        </DialogTitle>
        <DialogContent dividers>
          <Box sx={{ display: "flex", flexDirection: "column", gap: 2, pt: 1 }}>
            <TextField
              label="Slug / Identificador (URL)"
              value={pageForm.section_key}
              onChange={(e) => setPageForm({ ...pageForm, section_key: e.target.value.toLowerCase().replace(/[^a-z0-9_-]/g, "") })}
              disabled={!!editingPage}
              helperText={editingPage ? "El slug no se puede modificar una vez creado para no romper URLs fijas de WhatsApp." : "Ejemplo: politica, economia, seguridad, tecnologia"}
              fullWidth
            />
            <TextField
              label="Título Público"
              value={pageForm.title}
              onChange={(e) => setPageForm({ ...pageForm, title: e.target.value })}
              helperText="Ejemplo: Política y Coyuntura Nacional"
              fullWidth
            />
            <TextField
              label="Subtítulo / Bajada"
              value={pageForm.subtitle}
              onChange={(e) => setPageForm({ ...pageForm, subtitle: e.target.value })}
              fullWidth
            />
            <Grid container spacing={2}>
              <Grid size={{ xs: 6 }}>
                <TextField
                  label="Color Hex"
                  value={pageForm.color_hex}
                  onChange={(e) => setPageForm({ ...pageForm, color_hex: e.target.value })}
                  fullWidth
                  InputProps={{
                    startAdornment: (
                      <Box
                        sx={{
                          width: 20,
                          height: 20,
                          borderRadius: "50%",
                          bgcolor: pageForm.color_hex || "#605DFF",
                          mr: 1,
                          border: "1px solid #cbd5e1",
                        }}
                      />
                    ),
                  }}
                />
              </Grid>
              <Grid size={{ xs: 6 }}>
                <TextField
                  label="Ícono Material"
                  value={pageForm.icon_name}
                  onChange={(e) => setPageForm({ ...pageForm, icon_name: e.target.value })}
                  helperText="article, security, trending_up, etc."
                  fullWidth
                />
              </Grid>
            </Grid>
            <Grid container spacing={2}>
              <Grid size={{ xs: 6 }}>
                <TextField
                  label="Orden de visualización"
                  type="number"
                  value={pageForm.orden}
                  onChange={(e) => setPageForm({ ...pageForm, orden: parseInt(e.target.value, 10) || 0 })}
                  fullWidth
                />
              </Grid>
              <Grid size={{ xs: 6 }} sx={{ display: "flex", alignItems: "center" }}>
                <FormControlLabel
                  control={
                    <Switch
                      checked={pageForm.is_active}
                      onChange={(e) => setPageForm({ ...pageForm, is_active: e.target.checked })}
                      color="primary"
                    />
                  }
                  label="Canal Activo"
                />
              </Grid>
            </Grid>
          </Box>
        </DialogContent>
        <DialogActions sx={{ p: 2 }}>
          <Button onClick={() => setPageDialogOpen(false)} sx={{ textTransform: "none" }}>
            Cancelar
          </Button>
          <Button
            variant="contained"
            onClick={handleSavePage}
            sx={{ textTransform: "none", bgcolor: "#605DFF", "&:hover": { bgcolor: "#4d4ad6" } }}
          >
            Guardar
          </Button>
        </DialogActions>
      </Dialog>

      {/* =================================================================== */}
      {/* DIALOG: EDIT SECTION CONTENT                                       */}
      {/* =================================================================== */}
      <Dialog open={sectionDialogOpen} onClose={() => setSectionDialogOpen(false)} maxWidth="md" fullWidth>
        <DialogTitle sx={{ fontWeight: 700 }}>
          Editar Contenido: {editingSection?.title} ({selectedEdition?.edition_date})
        </DialogTitle>
        <DialogContent dividers>
          <Box sx={{ display: "flex", flexDirection: "column", gap: 2.5, pt: 1 }}>
            <TextField
              label="Título de la Sección"
              value={sectionForm.title}
              onChange={(e) => setSectionForm({ ...sectionForm, title: e.target.value })}
              fullWidth
            />
            <TextField
              label="Resumen Ejecutivo (Bajada)"
              multiline
              rows={3}
              value={sectionForm.summary}
              onChange={(e) => setSectionForm({ ...sectionForm, summary: e.target.value })}
              helperText="Resumen conciso visible como destacado en el feed público."
              fullWidth
            />
            <TextField
              label="Contenido Completo (Markdown / Texto)"
              multiline
              rows={12}
              value={sectionForm.full_content}
              onChange={(e) => setSectionForm({ ...sectionForm, full_content: e.target.value })}
              helperText="Sintesis completa desarrollada por los analistas o Claude."
              fullWidth
            />
            <FormControlLabel
              control={
                <Switch
                  checked={sectionForm.published}
                  onChange={(e) => setSectionForm({ ...sectionForm, published: e.target.checked })}
                  color="success"
                />
              }
              label="Publicar esta sección en el portal web"
            />
          </Box>
        </DialogContent>
        <DialogActions sx={{ p: 2 }}>
          <Button onClick={() => setSectionDialogOpen(false)} sx={{ textTransform: "none" }}>
            Cancelar
          </Button>
          <Button
            variant="contained"
            startIcon={<SaveIcon />}
            onClick={handleSaveSection}
            sx={{ textTransform: "none", bgcolor: "#605DFF", "&:hover": { bgcolor: "#4d4ad6" } }}
          >
            Guardar Cambios
          </Button>
        </DialogActions>
      </Dialog>

      {/* =================================================================== */}
      {/* DIALOG: ADD SUBSCRIBER                                             */}
      {/* =================================================================== */}
      <Dialog open={subDialogOpen} onClose={() => setSubDialogOpen(false)} maxWidth="xs" fullWidth>
        <DialogTitle sx={{ fontWeight: 700 }}>Nuevo Suscriptor WhatsApp</DialogTitle>
        <DialogContent dividers>
          <Box sx={{ display: "flex", flexDirection: "column", gap: 2, pt: 1 }}>
            <TextField
              label="Número Telefónico (E.164)"
              placeholder="+525512345678"
              value={subForm.phone_number}
              onChange={(e) => setSubForm({ ...subForm, phone_number: e.target.value })}
              helperText="Debe incluir clave de país (+52 para México)."
              fullWidth
            />
            <TextField
              label="Nombre Completo / Destinatario"
              placeholder="Lic. Roberto Sánchez"
              value={subForm.display_name}
              onChange={(e) => setSubForm({ ...subForm, display_name: e.target.value })}
              fullWidth
            />
            <TextField
              label="Origen del Consentimiento"
              value={subForm.consent_source}
              onChange={(e) => setSubForm({ ...subForm, consent_source: e.target.value })}
              fullWidth
            />
          </Box>
        </DialogContent>
        <DialogActions sx={{ p: 2 }}>
          <Button onClick={() => setSubDialogOpen(false)} sx={{ textTransform: "none" }}>
            Cancelar
          </Button>
          <Button
            variant="contained"
            onClick={handleSaveSubscriber}
            sx={{ textTransform: "none", bgcolor: "#605DFF", "&:hover": { bgcolor: "#4d4ad6" } }}
          >
            Registrar
          </Button>
        </DialogActions>
      </Dialog>

      {/* =================================================================== */}
      {/* DIALOG: TRIGGER BROADCAST                                          */}
      {/* =================================================================== */}
      <Dialog open={broadcastDialogOpen} onClose={() => setBroadcastDialogOpen(false)} maxWidth="sm" fullWidth>
        <DialogTitle sx={{ fontWeight: 700 }}>Disparar Campaña WhatsApp (Kapso)</DialogTitle>
        <DialogContent dividers>
          <Box sx={{ display: "flex", flexDirection: "column", gap: 2.5, pt: 1 }}>
            <Alert severity="info">
              Se despachará la plantilla oficial <code>brief_informativo_diario</code> con botones directos hacia las secciones <strong>Política, Economía y Seguridad</strong>.
            </Alert>

            <FormControl fullWidth>
              <InputLabel>Fecha de la Edición</InputLabel>
              <Select
                value={broadcastForm.edition_date}
                label="Fecha de la Edición"
                onChange={(e) => setBroadcastForm({ ...broadcastForm, edition_date: e.target.value })}
              >
                {editions.map((ed) => (
                  <MenuItem key={ed.edition_date} value={ed.edition_date}>
                    {ed.edition_date} ({ed.status === "published" ? "Publicada" : "Borrador"})
                  </MenuItem>
                ))}
              </Select>
            </FormControl>

            <TextField
              label="Número de prueba individual (Opcional)"
              placeholder="+525512345678"
              value={broadcastForm.test_phone}
              onChange={(e) => setBroadcastForm({ ...broadcastForm, test_phone: e.target.value })}
              helperText="Si indicas un número, SOLO se enviará a este número para validar la plantilla y los botones."
              fullWidth
            />

            {!broadcastForm.test_phone && (
              <FormControlLabel
                control={
                  <Switch
                    checked={broadcastForm.send_immediately}
                    onChange={(e) => setBroadcastForm({ ...broadcastForm, send_immediately: e.target.checked })}
                    color="warning"
                  />
                }
                label="Enviar de inmediato (si no se activa, se programa a las 07:00 AM CDMX)"
              />
            )}
          </Box>
        </DialogContent>
        <DialogActions sx={{ p: 2 }}>
          <Button onClick={() => setBroadcastDialogOpen(false)} sx={{ textTransform: "none" }}>
            Cancelar
          </Button>
          <Button
            variant="contained"
            startIcon={triggering ? <CircularProgress size={16} /> : <SendIcon />}
            onClick={handleTriggerBroadcast}
            disabled={triggering || !broadcastForm.edition_date}
            sx={{ textTransform: "none", bgcolor: "#16a34a", "&:hover": { bgcolor: "#15803d" } }}
          >
            {triggering ? "Enviando a Kapso..." : broadcastForm.test_phone ? "Enviar Mensaje de Prueba" : "Confirmar Broadcast"}
          </Button>
        </DialogActions>
      </Dialog>

      {/* Snackbar alerts */}
      <Snackbar
        open={snackbar.open}
        autoHideDuration={5000}
        onClose={() => setSnackbar((prev) => ({ ...prev, open: false }))}
      >
        <Alert severity={snackbar.severity} onClose={() => setSnackbar((prev) => ({ ...prev, open: false }))}>
          {snackbar.message}
        </Alert>
      </Snackbar>
    </Box>
  );
}
