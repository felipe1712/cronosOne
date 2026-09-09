"use client";

import React, { useState, useEffect } from "react";
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
  Alert,
  Tabs,
  Tab,
  Select,
  MenuItem,
  FormControl,
  InputLabel,
  IconButton,
  Tooltip,
  Switch,
  Divider,
} from "@mui/material";
import SecurityIcon from "@mui/icons-material/Security";
import CheckCircleIcon from "@mui/icons-material/CheckCircle";
import CancelIcon from "@mui/icons-material/Cancel";
import AddIcon from "@mui/icons-material/Add";
import RefreshIcon from "@mui/icons-material/Refresh";
import ToggleOnIcon from "@mui/icons-material/ToggleOn";
import ToggleOffIcon from "@mui/icons-material/ToggleOff";
import DeleteOutlineIcon from "@mui/icons-material/DeleteOutline";
import PublicIcon from "@mui/icons-material/Public";
import LanguageIcon from "@mui/icons-material/Language";
import HubIcon from "@mui/icons-material/Hub";
import PlayArrowIcon from "@mui/icons-material/PlayArrow";
import { ApiService } from "@/lib/api";

export default function OsintPage() {
  const [tabIndex, setTabIndex] = useState<number>(0);
  const [alertas, setAlertas] = useState<any[]>([]);
  const [entidades, setEntidades] = useState<any[]>([]);
  const [fuentes, setFuentes] = useState<any[]>([]);
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  // Modal para agregar entidad vigilada
  const [openAddModal, setOpenAddModal] = useState<boolean>(false);
  const [newTipo, setNewTipo] = useState<string>("marca");
  const [newValor, setNewValor] = useState<string>("");
  const [newDescripcion, setNewDescripcion] = useState<string>("");

  // Modal para eliminar entidad vigilada
  const [entityToDelete, setEntityToDelete] = useState<any | null>(null);
  const [openDeleteModal, setOpenDeleteModal] = useState<boolean>(false);
  const [deletingEntity, setDeletingEntity] = useState<boolean>(false);

  // Modal para validar / descartar alerta
  const [selectedAlerta, setSelectedAlerta] = useState<any | null>(null);
  const [validationNotes, setValidationNotes] = useState<string>("");
  const [openValidationModal, setOpenValidationModal] = useState<boolean>(false);
  const [actionType, setActionType] = useState<"validar" | "descartar">("validar");

  const loadData = async () => {
    try {
      setLoading(true);
      setError(null);
      const [alertasData, entidadesData, fuentesData] = await Promise.all([
        ApiService.getAlertas(),
        ApiService.getEntidades(),
        ApiService.getFuentesOsint().catch(() => []),
      ]);
      setAlertas(alertasData);
      setEntidades(entidadesData);
      setFuentes(fuentesData);
    } catch (err: any) {
      setError(err.message || "Error consultando datos de OSINT");
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadData();
  }, []);

  const handleCreateEntidad = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!newValor.trim()) return;

    try {
      await ApiService.createEntidad(newTipo, newValor.trim(), newDescripcion.trim());
      setSuccess("Entidad agregada al catálogo de vigilancia con éxito.");
      setOpenAddModal(false);
      setNewValor("");
      setNewDescripcion("");
      loadData();
    } catch (err: any) {
      setError(err.message || "Error al registrar la entidad.");
    }
  };

  const handleToggleEntidad = async (id: string) => {
    try {
      await ApiService.toggleEntidad(id);
      loadData();
    } catch (err: any) {
      setError(err.message || "Error al actualizar estado.");
    }
  };

  const handleOpenDeleteModal = (entidad: any) => {
    setEntityToDelete(entidad);
    setOpenDeleteModal(true);
  };

  const handleConfirmDelete = async () => {
    if (!entityToDelete) return;
    try {
      setDeletingEntity(true);
      await ApiService.deleteEntidad(entityToDelete.id);
      setSuccess(`Entidad "${entityToDelete.valor}" eliminada del catálogo exitosamente.`);
      setOpenDeleteModal(false);
      setEntityToDelete(null);
      loadData();
    } catch (err: any) {
      setError(err.message || "Error al eliminar la entidad.");
    } finally {
      setDeletingEntity(false);
    }
  };

  const handleToggleFuente = async (id: string) => {
    try {
      await ApiService.toggleFuenteOsint(id);
      const updated = await ApiService.getFuentesOsint();
      setFuentes(updated);
      setSuccess("Estado de la fuente OSINT actualizado con éxito.");
    } catch (err: any) {
      setError(err.message || "Error al actualizar la fuente OSINT.");
    }
  };

  const openActionModal = (alerta: any, action: "validar" | "descartar") => {
    setSelectedAlerta(alerta);
    setActionType(action);
    setValidationNotes("");
    setOpenValidationModal(true);
  };

  const handleConfirmValidation = async () => {
    if (!selectedAlerta) return;
    try {
      await ApiService.validarAlerta(selectedAlerta.id, actionType, validationNotes);
      setSuccess(
        actionType === "validar"
          ? "Alerta validada y encolada automáticamente para notificación inmediata por WhatsApp."
          : "Alerta descartada como falso positivo."
      );
      setOpenValidationModal(false);
      loadData();
    } catch (err: any) {
      setError(err.message || "Error al validar la alerta.");
    }
  };

  const getSeveridadChip = (sev: string) => {
    switch (sev) {
      case "critica":
        return <Chip label="CRÍTICA" color="error" size="small" sx={{ fontWeight: 700 }} />;
      case "alta":
        return <Chip label="ALTA" color="warning" size="small" sx={{ fontWeight: 600 }} />;
      case "media":
        return <Chip label="MEDIA" color="info" size="small" />;
      default:
        return <Chip label="BAJA" size="small" />;
    }
  };

  const getEstadoAlertaChip = (est: string) => {
    switch (est) {
      case "validada":
        return <Chip label="Validada (Enviada)" color="success" size="small" />;
      case "descartada":
        return <Chip label="Descartada" color="default" size="small" />;
      default:
        return <Chip label="Por Validar (Analista)" color="warning" size="small" />;
    }
  };

  const getFuenteTipoChip = (tipo: string) => {
    switch (tipo) {
      case "mcp_service":
        return <Chip label="MCP Service" color="primary" size="small" variant="outlined" />;
      case "scraper":
        return <Chip label="Scraper / Crawler" color="secondary" size="small" variant="outlined" />;
      case "rss":
        return <Chip label="Feed RSS / DOF" color="info" size="small" variant="outlined" />;
      case "api":
        return <Chip label="API / Social" color="success" size="small" variant="outlined" />;
      default:
        return <Chip label={tipo} size="small" variant="outlined" />;
    }
  };

  return (
    <Box sx={{ p: 3 }}>
      {/* Encabezado */}
      <Box sx={{ display: "flex", justifyContent: "space-between", alignItems: "center", mb: 3 }}>
        <Box>
          <Typography variant="h5" sx={{ fontWeight: 700, color: "#1e293b" }}>
            Inteligencia OSINT & Exposición Digital (world-intel-mcp)
          </Typography>
          <Typography variant="body2" sx={{ color: "#64748b" }}>
            Vigilancia en tiempo real de fuentes abiertas, redes y marketplaces de Dark Web sobre datos de la aseguradora.
          </Typography>
        </Box>
        <Button variant="outlined" startIcon={<RefreshIcon />} onClick={loadData} disabled={loading}>
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

      {/* Tabs */}
      <Box sx={{ borderBottom: 1, borderColor: "divider", mb: 3 }}>
        <Tabs value={tabIndex} onChange={(_, v) => setTabIndex(v)}>
          <Tab label={`Bandeja de Alertas (${alertas.length})`} />
          <Tab label={`Entidades Vigiladas (${entidades.length})`} />
          <Tab label={`Fuentes OSINT (world-intel-mcp) (${fuentes.length})`} />
        </Tabs>
      </Box>

      {/* TAB 0: ALERTAS */}
      {tabIndex === 0 && (
        <Card sx={{ borderRadius: "12px", boxShadow: "0 2px 6px rgba(0,0,0,0.04)" }}>
          <CardContent>
            <Box sx={{ display: "flex", justifyContent: "space-between", alignItems: "center", mb: 2 }}>
              <Typography variant="h6" sx={{ fontWeight: 600 }}>
                Hallazgos y Alertas Detectadas
              </Typography>
              <Typography variant="caption" sx={{ color: "#64748b" }}>
                🔒 Las alertas validadas se despachan prioritariamente vía WAHA al Director de Operaciones.
              </Typography>
            </Box>

            {loading ? (
              <Box sx={{ display: "flex", justifyContent: "center", p: 4 }}>
                <CircularProgress />
              </Box>
            ) : alertas.length === 0 ? (
              <Box sx={{ textAlign: "center", p: 4, color: "#94a3b8" }}>
                <SecurityIcon sx={{ fontSize: 48, mb: 1 }} />
                <Typography>No hay alertas detectadas por el momento.</Typography>
              </Box>
            ) : (
              <TableContainer component={Paper} elevation={0}>
                <Table size="small">
                  <TableHead sx={{ backgroundColor: "#f8fafc" }}>
                    <TableRow>
                      <TableCell sx={{ fontWeight: 600 }}>Severidad</TableCell>
                      <TableCell sx={{ fontWeight: 600 }}>Fuente</TableCell>
                      <TableCell sx={{ fontWeight: 600 }}>Título / Hallazgo</TableCell>
                      <TableCell sx={{ fontWeight: 600 }}>Estado</TableCell>
                      <TableCell sx={{ fontWeight: 600 }}>Fecha</TableCell>
                      <TableCell sx={{ fontWeight: 600, textAlign: "right" }}>Acción Analista</TableCell>
                    </TableRow>
                  </TableHead>
                  <TableBody>
                    {alertas.map((a) => (
                      <TableRow key={a.id} hover>
                        <TableCell>{getSeveridadChip(a.severidad)}</TableCell>
                        <TableCell sx={{ fontWeight: 500 }}>{a.fuente}</TableCell>
                        <TableCell sx={{ maxWidth: 300 }}>
                          <Typography variant="body2" sx={{ fontWeight: 600, color: "#1e293b" }}>
                            {a.titulo}
                          </Typography>
                          <Typography variant="caption" sx={{ color: "#64748b" }}>
                            {a.descripcion}
                          </Typography>
                        </TableCell>
                        <TableCell>{getEstadoAlertaChip(a.estado)}</TableCell>
                        <TableCell>{new Date(a.creado_en).toLocaleString()}</TableCell>
                        <TableCell sx={{ textAlign: "right" }}>
                          {a.estado === "por_validar" ? (
                            <Box sx={{ display: "flex", gap: 1, justifyContent: "flex-end" }}>
                              <Button
                                size="small"
                                variant="contained"
                                color="error"
                                startIcon={<CheckCircleIcon />}
                                onClick={() => openActionModal(a, "validar")}
                              >
                                Validar
                              </Button>
                              <Button
                                size="small"
                                variant="outlined"
                                color="inherit"
                                startIcon={<CancelIcon />}
                                onClick={() => openActionModal(a, "descartar")}
                              >
                                Descartar
                              </Button>
                            </Box>
                          ) : (
                            <Typography variant="caption" sx={{ color: "#94a3b8" }}>
                              Procesada
                            </Typography>
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
      )}

      {/* TAB 1: ENTIDADES VIGILADAS (CON BOTÓN DE ELIMINAR) */}
      {tabIndex === 1 && (
        <Card sx={{ borderRadius: "12px", boxShadow: "0 2px 6px rgba(0,0,0,0.04)" }}>
          <CardContent>
            <Box sx={{ display: "flex", justifyContent: "space-between", alignItems: "center", mb: 2 }}>
              <Box>
                <Typography variant="h6" sx={{ fontWeight: 600 }}>
                  Catálogo de Entidades Vigiladas
                </Typography>
                <Typography variant="body2" sx={{ color: "#64748b" }}>
                  Marcas, dominios de correo, nombres de directivos y expresiones regulares de números de póliza.
                </Typography>
              </Box>
              <Button
                variant="contained"
                startIcon={<AddIcon />}
                onClick={() => setOpenAddModal(true)}
              >
                Nueva Entidad
              </Button>
            </Box>

            <TableContainer component={Paper} elevation={0}>
              <Table size="small">
                <TableHead sx={{ backgroundColor: "#f8fafc" }}>
                  <TableRow>
                    <TableCell sx={{ fontWeight: 600 }}>Tipo</TableCell>
                    <TableCell sx={{ fontWeight: 600 }}>Valor de Búsqueda / Regla</TableCell>
                    <TableCell sx={{ fontWeight: 600 }}>Descripción</TableCell>
                    <TableCell sx={{ fontWeight: 600 }}>Estado</TableCell>
                    <TableCell sx={{ fontWeight: 600, textAlign: "right" }}>Acciones</TableCell>
                  </TableRow>
                </TableHead>
                <TableBody>
                  {entidades.map((e) => (
                    <TableRow key={e.id} hover>
                      <TableCell>
                        <Chip label={e.tipo.toUpperCase()} size="small" variant="outlined" />
                      </TableCell>
                      <TableCell sx={{ fontFamily: "monospace", fontWeight: 600 }}>
                        {e.valor}
                      </TableCell>
                      <TableCell>{e.descripcion || "-"}</TableCell>
                      <TableCell>
                        {e.activo ? (
                          <Chip label="Activo" color="success" size="small" />
                        ) : (
                          <Chip label="Inactivo" color="default" size="small" />
                        )}
                      </TableCell>
                      <TableCell sx={{ textAlign: "right" }}>
                        <Box sx={{ display: "flex", justifyContent: "flex-end", alignItems: "center", gap: 0.5 }}>
                          <Tooltip title={e.activo ? "Desactivar vigilancia" : "Activar vigilancia"}>
                            <IconButton onClick={() => handleToggleEntidad(e.id)} color="primary" size="small">
                              {e.activo ? <ToggleOnIcon fontSize="large" color="success" /> : <ToggleOffIcon fontSize="large" />}
                            </IconButton>
                          </Tooltip>

                          {/* Botón de Eliminar */}
                          <Tooltip title="Eliminar entidad">
                            <IconButton
                              onClick={() => handleOpenDeleteModal(e)}
                              color="error"
                              size="small"
                              sx={{
                                "&:hover": { backgroundColor: "#fee2e2" },
                              }}
                            >
                              <DeleteOutlineIcon fontSize="medium" />
                            </IconButton>
                          </Tooltip>
                        </Box>
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </TableContainer>
          </CardContent>
        </Card>
      )}

      {/* TAB 2: ADMINISTRADOR DE FUENTES OSINT (WORLD-INTEL-MCP) */}
      {tabIndex === 2 && (
        <Card sx={{ borderRadius: "12px", boxShadow: "0 2px 6px rgba(0,0,0,0.04)" }}>
          <CardContent>
            <Box sx={{ display: "flex", justifyContent: "space-between", alignItems: "center", mb: 3 }}>
              <Box>
                <Typography variant="h6" sx={{ fontWeight: 600 }}>
                  Administrador de Fuentes de Inteligencia (world-intel-mcp)
                </Typography>
                <Typography variant="body2" sx={{ color: "#64748b" }}>
                  Selecciona qué canales, foros y fuentes de datos se consultan durante los escaneos de exposición de la aseguradora.
                </Typography>
              </Box>
              <Chip
                icon={<HubIcon />}
                label="Protocolo MCP Conectado"
                color="success"
                variant="outlined"
                sx={{ fontWeight: 600 }}
              />
            </Box>

            <Grid container spacing={2.5}>
              {fuentes.map((f) => (
                <Grid size={{ xs: 12, md: 6, lg: 4 }} key={f.id}>
                  <Card
                    sx={{
                      borderRadius: "10px",
                      border: "1px solid",
                      borderColor: f.activo ? "#c7d2fe" : "#e2e8f0",
                      backgroundColor: f.activo ? "#fdfdff" : "#f8fafc",
                      p: 2,
                      height: "100%",
                      display: "flex",
                      flexDirection: "column",
                      justifyContent: "space-between",
                      transition: "all 0.2s ease-in-out",
                      "&:hover": {
                        boxShadow: "0 4px 12px rgba(0,0,0,0.06)",
                      },
                    }}
                  >
                    <Box>
                      <Box sx={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start", mb: 1 }}>
                        <Box sx={{ display: "flex", alignItems: "center", gap: 1 }}>
                          <PublicIcon sx={{ color: f.activo ? "#4f46e5" : "#94a3b8" }} />
                          <Typography variant="subtitle2" sx={{ fontWeight: 700, color: "#1e293b" }}>
                            {f.nombre}
                          </Typography>
                        </Box>
                        <Switch
                          checked={f.activo}
                          onChange={() => handleToggleFuente(f.id)}
                          color="primary"
                        />
                      </Box>

                      <Box sx={{ mb: 1.5, display: "flex", gap: 1, alignItems: "center" }}>
                        {getFuenteTipoChip(f.tipo)}
                        <Typography variant="caption" sx={{ fontFamily: "monospace", color: "#64748b" }}>
                          {f.clave}
                        </Typography>
                      </Box>

                      <Typography variant="body2" sx={{ color: "#475569", fontSize: "0.85rem", mb: 2 }}>
                        {f.descripcion}
                      </Typography>
                    </Box>

                    <Box sx={{ pt: 1, borderTop: "1px solid #e2e8f0", display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                      <Typography variant="caption" sx={{ color: "#64748b" }}>
                        Hallazgos totales: <strong>{f.total_hallazgos || 0}</strong>
                      </Typography>
                      <Chip
                        label={f.activo ? "Monitoreando" : "Inactivo"}
                        color={f.activo ? "success" : "default"}
                        size="small"
                        sx={{ fontSize: "0.72rem" }}
                      />
                    </Box>
                  </Card>
                </Grid>
              ))}
            </Grid>
          </CardContent>
        </Card>
      )}

      {/* Modal para Crear Entidad */}
      <Dialog open={openAddModal} onClose={() => setOpenAddModal(false)} maxWidth="sm" fullWidth>
        <DialogTitle sx={{ fontWeight: 700 }}>Agregar Nueva Entidad Vigilada</DialogTitle>
        <form onSubmit={handleCreateEntidad}>
          <DialogContent>
            <FormControl fullWidth size="small" sx={{ mb: 2, mt: 1 }}>
              <InputLabel>Tipo de Entidad</InputLabel>
              <Select
                value={newTipo}
                label="Tipo de Entidad"
                onChange={(e) => setNewTipo(e.target.value)}
              >
                <MenuItem value="marca">Marca / Nombre Comercial</MenuItem>
                <MenuItem value="dominio">Dominio Web / Correo</MenuItem>
                <MenuItem value="ejecutivo">Nombre de Ejecutivo Clave</MenuItem>
                <MenuItem value="patron_poliza">Patrón Regex de Póliza</MenuItem>
              </Select>
            </FormControl>

            <TextField
              label="Valor a vigilar"
              fullWidth
              size="small"
              required
              value={newValor}
              onChange={(e) => setNewValor(e.target.value)}
              placeholder="ej. Seguros Alianza, alianzaseguros.com.mx, POL-[0-9]{8}"
              sx={{ mb: 2 }}
            />

            <TextField
              label="Descripción / Justificación"
              fullWidth
              size="small"
              multiline
              rows={2}
              value={newDescripcion}
              onChange={(e) => setNewDescripcion(e.target.value)}
            />
          </DialogContent>
          <DialogActions sx={{ p: 2 }}>
            <Button onClick={() => setOpenAddModal(false)}>Cancelar</Button>
            <Button type="submit" variant="contained">Guardar Entidad</Button>
          </DialogActions>
        </form>
      </Dialog>

      {/* Modal de Confirmación de Eliminación de Entidad */}
      <Dialog open={openDeleteModal} onClose={() => setOpenDeleteModal(false)} maxWidth="xs" fullWidth>
        <DialogTitle sx={{ fontWeight: 700, color: "#dc2626" }}>
          Confirmar Eliminación de Entidad
        </DialogTitle>
        <DialogContent>
          <Typography variant="body2" sx={{ color: "#334155", mb: 1 }}>
            ¿Estás seguro de que deseas eliminar permanentemente esta entidad del catálogo de vigilancia?
          </Typography>
          {entityToDelete && (
            <Box sx={{ p: 1.5, backgroundColor: "#fee2e2", borderRadius: "6px", border: "1px solid #fca5a5" }}>
              <Typography variant="subtitle2" sx={{ fontWeight: 700, color: "#991b1b" }}>
                {entityToDelete.valor}
              </Typography>
              <Typography variant="caption" sx={{ color: "#7f1d1d" }}>
                Tipo: {entityToDelete.tipo.toUpperCase()}
              </Typography>
            </Box>
          )}
          <Typography variant="caption" sx={{ display: "block", mt: 1.5, color: "#64748b" }}>
            Las alertas previas generadas por esta entidad conservarán su evidencia histórica de auditoría.
          </Typography>
        </DialogContent>
        <DialogActions sx={{ p: 2 }}>
          <Button onClick={() => setOpenDeleteModal(false)} disabled={deletingEntity}>
            Cancelar
          </Button>
          <Button
            variant="contained"
            color="error"
            onClick={handleConfirmDelete}
            disabled={deletingEntity}
          >
            {deletingEntity ? <CircularProgress size={20} color="inherit" /> : "Eliminar Entidad"}
          </Button>
        </DialogActions>
      </Dialog>

      {/* Modal de Validación de Alerta */}
      <Dialog open={openValidationModal} onClose={() => setOpenValidationModal(false)} maxWidth="sm" fullWidth>
        <DialogTitle sx={{ fontWeight: 700 }}>
          {actionType === "validar" ? "Confirmar y Despachar Alerta Urgente" : "Descartar Alerta"}
        </DialogTitle>
        <DialogContent>
          <Typography variant="body2" sx={{ mb: 2, color: "#475569" }}>
            {actionType === "validar"
              ? "Al validar esta alerta, se insertará inmediatamente en la cola de salida para que el cron de n8n la entregue por WhatsApp al Director de Operaciones."
              : "¿Deseas descartar este hallazgo como falso positivo?"}
          </Typography>

          <TextField
            label="Notas del analista (opcional)"
            fullWidth
            size="small"
            multiline
            rows={3}
            value={validationNotes}
            onChange={(e) => setValidationNotes(e.target.value)}
            placeholder="Comentarios de análisis técnico, mitigación o contexto..."
          />
        </DialogContent>
        <DialogActions sx={{ p: 2 }}>
          <Button onClick={() => setOpenValidationModal(false)}>Cancelar</Button>
          <Button
            variant="contained"
            color={actionType === "validar" ? "error" : "inherit"}
            onClick={handleConfirmValidation}
          >
            {actionType === "validar" ? "Validar y Notificar WAHA" : "Confirmar Descarte"}
          </Button>
        </DialogActions>
      </Dialog>
    </Box>
  );
}
