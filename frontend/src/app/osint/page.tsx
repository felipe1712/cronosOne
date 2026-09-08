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
} from "@mui/material";
import SecurityIcon from "@mui/icons-material/Security";
import CheckCircleIcon from "@mui/icons-material/CheckCircle";
import CancelIcon from "@mui/icons-material/Cancel";
import AddIcon from "@mui/icons-material/Add";
import RefreshIcon from "@mui/icons-material/Refresh";
import ToggleOnIcon from "@mui/icons-material/ToggleOn";
import ToggleOffIcon from "@mui/icons-material/ToggleOff";
import { ApiService } from "@/lib/api";

export default function OsintPage() {
  const [tabIndex, setTabIndex] = useState<number>(0);
  const [alertas, setAlertas] = useState<any[]>([]);
  const [entidades, setEntidades] = useState<any[]>([]);
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  // Modal para agregar entidad vigilada
  const [openAddModal, setOpenAddModal] = useState<boolean>(false);
  const [newTipo, setNewTipo] = useState<string>("marca");
  const [newValor, setNewValor] = useState<string>("");
  const [newDescripcion, setNewDescripcion] = useState<string>("");

  // Modal para validar / descartar alerta
  const [selectedAlerta, setSelectedAlerta] = useState<any | null>(null);
  const [validationNotes, setValidationNotes] = useState<string>("");
  const [openValidationModal, setOpenValidationModal] = useState<boolean>(false);
  const [actionType, setActionType] = useState<"validar" | "descartar">("validar");

  const loadData = async () => {
    try {
      setLoading(true);
      setError(null);
      const [alertasData, entidadesData] = await Promise.all([
        ApiService.getAlertas(),
        ApiService.getEntidades(),
      ]);
      setAlertas(alertasData);
      setEntidades(entidadesData);
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
      setSuccess("Entidad agregada al catálogo de vigilancia con éxito");
      setOpenAddModal(false);
      setNewValor("");
      setNewDescripcion("");
      loadData();
    } catch (err: any) {
      setError(err.message || "Error al registrar la entidad");
    }
  };

  const handleToggleEntidad = async (id: string) => {
    try {
      await ApiService.toggleEntidad(id);
      loadData();
    } catch (err: any) {
      setError(err.message || "Error al actualizar estado");
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
      setError(err.message || "Error al validar la alerta");
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

  return (
    <Box sx={{ p: 3 }}>
      {/* Encabezado */}
      <Box sx={{ display: "flex", justifyContent: "space-between", alignItems: "center", mb: 3 }}>
        <Box>
          <Typography variant="h5" sx={{ fontWeight: 700, color: "#1e293b" }}>
            Inteligencia OSINT & Exposición Digital (world-intel-mcp)
          </Typography>
          <Typography variant="body2" sx={{ color: "#64748b" }}>
            Vigilancia en tiempo real de redes, foros y marketplaces de Dark Web sobre datos de la aseguradora.
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

      {/* TAB 1: ENTIDADES VIGILADAS */}
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
                    <TableCell sx={{ fontWeight: 600, textAlign: "right" }}>Acción</TableCell>
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
                        <Tooltip title={e.activo ? "Desactivar vigilancia" : "Activar vigilancia"}>
                          <IconButton onClick={() => handleToggleEntidad(e.id)} color="primary">
                            {e.activo ? <ToggleOnIcon fontSize="large" color="success" /> : <ToggleOffIcon fontSize="large" />}
                          </IconButton>
                        </Tooltip>
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </TableContainer>
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
