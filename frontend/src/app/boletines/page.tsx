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
} from "@mui/material";
import CloudUploadIcon from "@mui/icons-material/CloudUpload";
import RefreshIcon from "@mui/icons-material/Refresh";
import DescriptionIcon from "@mui/icons-material/Description";
import VisibilityIcon from "@mui/icons-material/Visibility";
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

  // Modal de detalle / síntesis
  const [selectedBoletin, setSelectedBoletin] = useState<any | null>(null);
  const [modalOpen, setModalOpen] = useState<boolean>(false);
  const [loadingDetail, setLoadingDetail] = useState<boolean>(false);

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

  useEffect(() => {
    fetchBoletines();
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

  const handleViewDetail = async (id: string) => {
    try {
      setLoadingDetail(true);
      setModalOpen(true);
      const detail = await ApiService.getBoletinDetail(id);
      setSelectedBoletin(detail);
    } catch (err: any) {
      setError("No se pudo cargar el detalle del boletín");
    } finally {
      setLoadingDetail(false);
    }
  };

  const getStatusChip = (estado: string) => {
    switch (estado) {
      case "sintesis_lista":
        return <Chip label="Síntesis Lista" color="success" size="small" />;
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
      <Box sx={{ display: "flex", justifyContent: "space-between", alignItems: "center", mb: 3 }}>
        <Box>
          <Typography variant="h5" sx={{ fontWeight: 700, color: "#1e293b" }}>
            Boletines Coparmex — Ingesta & Síntesis Ejecutiva
          </Typography>
          <Typography variant="body2" sx={{ color: "#64748b" }}>
            Carga manual del PDF diario, procesamiento con Surya OCR y generación del brief matutino con Claude API.
          </Typography>
        </Box>
        <Button
          variant="outlined"
          startIcon={<RefreshIcon />}
          onClick={fetchBoletines}
          disabled={loading}
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
                            <Button
                              size="small"
                              variant="outlined"
                              startIcon={<VisibilityIcon />}
                              onClick={() => handleViewDetail(b.id)}
                            >
                              Ver Brief
                            </Button>
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

      {/* Modal de Detalle y Síntesis */}
      <Dialog
        open={modalOpen}
        onClose={() => setModalOpen(false)}
        maxWidth="md"
        fullWidth
      >
        <DialogTitle sx={{ fontWeight: 700, borderBottom: "1px solid #e2e8f0" }}>
          Síntesis Ejecutiva & Secciones del Boletín
        </DialogTitle>
        <DialogContent sx={{ mt: 2 }}>
          {loadingDetail ? (
            <Box sx={{ display: "flex", justifyContent: "center", p: 4 }}>
              <CircularProgress />
            </Box>
          ) : selectedBoletin ? (
            <Box>
              {/* Bloque del Briefing de WhatsApp */}
              <Box sx={{ mb: 3, p: 2.5, backgroundColor: "#f0fdf4", border: "1px solid #bbf7d0", borderRadius: "8px" }}>
                <Typography variant="subtitle2" sx={{ fontWeight: 700, color: "#166534", mb: 1 }}>
                  📱 Mensaje generado para entrega vía WhatsApp (WAHA):
                </Typography>
                <Typography
                  component="pre"
                  sx={{
                    fontFamily: "inherit",
                    whiteSpace: "pre-wrap",
                    fontSize: "0.9rem",
                    color: "#1e293b",
                    m: 0,
                  }}
                >
                  {selectedBoletin.sintesis?.texto || "La síntesis ejecutiva aún está en proceso de generación."}
                </Typography>
              </Box>

              {/* Secciones Temáticas Extraídas */}
              <Typography variant="subtitle1" sx={{ fontWeight: 600, mb: 1 }}>
                Secciones Segmentadas ({selectedBoletin.secciones?.length || 0}):
              </Typography>
              {selectedBoletin.secciones?.map((s: any) => (
                <Box
                  key={s.id}
                  sx={{
                    p: 1.5,
                    mb: 1,
                    backgroundColor: "#f8fafc",
                    borderRadius: "6px",
                    border: "1px solid #e2e8f0",
                  }}
                >
                  <Typography variant="subtitle2" sx={{ fontWeight: 600, color: "#334155" }}>
                    #{s.orden} — {s.tema?.toUpperCase()} (Págs {s.pagina_inicio} a {s.pagina_fin})
                  </Typography>
                  <Typography variant="body2" sx={{ color: "#64748b", fontSize: "0.82rem" }}>
                    {s.contenido?.texto_completo?.slice(0, 200)}...
                  </Typography>
                </Box>
              ))}
            </Box>
          ) : null}
        </DialogContent>
        <DialogActions sx={{ borderTop: "1px solid #e2e8f0", p: 2 }}>
          <Button onClick={() => setModalOpen(false)}>Cerrar</Button>
        </DialogActions>
      </Dialog>
    </Box>
  );
}
