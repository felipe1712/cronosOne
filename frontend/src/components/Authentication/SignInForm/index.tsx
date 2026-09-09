"use client";

import React, { useState } from "react";
import {
  Grid,
  Button,
  Box,
  Typography,
  FormControl,
  TextField,
  Alert,
  CircularProgress,
} from "@mui/material";
import Image from "next/image";
import { useRouter } from "next/navigation";
import { ApiService, setToken } from "@/lib/api";

const SignInForm: React.FC = () => {
  const router = useRouter();
  const [correo, setCorreo] = useState("");
  const [password, setPassword] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!correo || !password) {
      setError("Por favor ingrese correo y contraseña");
      return;
    }

    try {
      setLoading(true);
      setError(null);

      const response = await ApiService.login(correo, password);
      if (response && response.token) {
        setToken(response.token);
        if (typeof window !== "undefined") {
          localStorage.setItem("exposureiq_user", JSON.stringify(response.usuario));
        }
        router.push("/boletines");
      } else {
        setError("Respuesta de autenticación inválida del servidor");
      }
    } catch (err: any) {
      setError(err.message || "Error al iniciar sesión. Verifique sus credenciales.");
    } finally {
      setLoading(false);
    }
  };

  return (
    <Box
      className="auth-main-wrapper sign-in-area"
      sx={{
        py: { xs: "50px", md: "80px", lg: "100px" },
        minHeight: "100vh",
        display: "flex",
        alignItems: "center",
      }}
    >
      <Box
        sx={{
          maxWidth: { sm: "500px", md: "1150px" },
          mx: "auto !important",
          px: "16px",
          width: "100%",
        }}
      >
        <Grid
          container
          alignItems="center"
          columnSpacing={{ xs: 1, sm: 2, md: 4, lg: 5 }}
        >
          <Grid size={{ xs: 12, md: 6, lg: 6 }}>
            <Box
              sx={{
                display: { xs: "none", md: "block" },
                textAlign: "center",
              }}
            >
              <Image
                src="/images/sign-in.jpg"
                alt="ExposureIQ Portal"
                width={560}
                height={680}
                style={{
                  borderRadius: "20px",
                  objectFit: "cover",
                  boxShadow: "0 10px 30px rgba(0,0,0,0.08)",
                  width: "100%",
                  height: "auto",
                  maxHeight: "680px",
                }}
              />
            </Box>
          </Grid>

          <Grid size={{ xs: 12, md: 6, lg: 6 }}>
            <Box
              className="form-content"
              sx={{
                paddingLeft: { xs: "0", lg: "20px" },
                backgroundColor: "#fff",
                p: { xs: 3, sm: 4 },
                borderRadius: "16px",
                boxShadow: "0 4px 20px rgba(0,0,0,0.05)",
              }}
            >
              <Box
                className="logo"
                sx={{
                  mb: "20px",
                  display: "flex",
                  alignItems: "center",
                  gap: 1.5,
                }}
              >
                <Image
                  src="/images/logo-icon.svg"
                  alt="ExposureIQ Logo"
                  width={36}
                  height={36}
                />
                <Typography
                  variant="h5"
                  sx={{
                    fontWeight: 700,
                    color: "#1e293b",
                    letterSpacing: "-0.5px",
                  }}
                >
                  ExposureIQ
                </Typography>
              </Box>

              <Box className="title" sx={{ mb: "24px" }}>
                <Typography
                  variant="h1"
                  className="text-black"
                  sx={{
                    fontSize: { xs: "22px", sm: "26px" },
                    mb: "6px",
                    fontWeight: "700",
                  }}
                >
                  Monitoreo Ejecutivo
                </Typography>

                <Typography sx={{ fontWeight: "400", fontSize: "14px", color: "#64748b" }}>
                  Ingrese sus credenciales institucionales para acceder al sistema.
                </Typography>
              </Box>

              {error && (
                <Alert severity="error" sx={{ mb: 3 }} onClose={() => setError(null)}>
                  {error}
                </Alert>
              )}

              <Box component="form" onSubmit={handleSubmit}>
                <Box mb="18px">
                  <FormControl fullWidth>
                    <Typography
                      component="label"
                      htmlFor="email-input"
                      sx={{
                        fontWeight: "600",
                        fontSize: "13px",
                        mb: "8px",
                        display: "block",
                        color: "#334155",
                      }}
                    >
                      Correo Electrónico
                    </Typography>

                    <TextField
                      id="email-input"
                      placeholder="admin@exposureiq.internal"
                      variant="outlined"
                      size="small"
                      value={correo}
                      onChange={(e) => setCorreo(e.target.value)}
                      required
                      fullWidth
                    />
                  </FormControl>
                </Box>

                <Box mb="24px">
                  <FormControl fullWidth>
                    <Typography
                      component="label"
                      htmlFor="password-input"
                      sx={{
                        fontWeight: "600",
                        fontSize: "13px",
                        mb: "8px",
                        display: "block",
                        color: "#334155",
                      }}
                    >
                      Contraseña
                    </Typography>

                    <TextField
                      id="password-input"
                      placeholder="••••••••••••"
                      variant="outlined"
                      size="small"
                      type="password"
                      value={password}
                      onChange={(e) => setPassword(e.target.value)}
                      required
                      fullWidth
                    />
                  </FormControl>
                </Box>

                <Box mb="20px">
                  <Button
                    type="submit"
                    variant="contained"
                    disabled={loading}
                    sx={{
                      textTransform: "none",
                      borderRadius: "8px",
                      fontWeight: "600",
                      fontSize: "15px",
                      py: 1.2,
                      width: "100%",
                      backgroundColor: "#4f46e5",
                      "&:hover": { backgroundColor: "#4338ca" },
                    }}
                  >
                    {loading ? (
                      <Box sx={{ display: "flex", alignItems: "center", gap: 1 }}>
                        <CircularProgress size={20} color="inherit" />
                        <span>Verificando credenciales...</span>
                      </Box>
                    ) : (
                      "Iniciar Sesión"
                    )}
                  </Button>
                </Box>

                <Typography sx={{ fontSize: "12px", color: "#94a3b8", textAlign: "center" }}>
                  Plataforma Segura de Inteligencia de Riesgo y Alertas Tempranas.
                </Typography>
              </Box>
            </Box>
          </Grid>
        </Grid>
      </Box>
    </Box>
  );
};

export default SignInForm;
