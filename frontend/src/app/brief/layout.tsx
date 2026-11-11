"use client";

import React, { useEffect, useState } from "react";
import Link from "next/link";
import { usePathname } from "next/navigation";
import {
  AppBar,
  Toolbar,
  Typography,
  Container,
  Box,
  Button,
  Chip,
  IconButton,
  Tooltip,
} from "@mui/material";
import { ApiService } from "@/lib/api";

export default function BriefLayout({ children }: { children: React.ReactNode }) {
  const pathname = usePathname();
  const [sections, setSections] = useState<any[]>([
    { section_key: "primeras_planas", title: "Primeras Planas", icon: "newspaper", color: "#7c3aed" },
    { section_key: "politica", title: "Política y Gobierno", icon: "account_balance", color: "#0284c7" },
    { section_key: "economia", title: "Economía y Finanzas", icon: "trending_up", color: "#16a34a" },
    { section_key: "seguridad", title: "Seguridad Nacional", icon: "security", color: "#dc2626" },
  ]);

  useEffect(() => {
    ApiService.getPublicSections()
      .then((data) => {
        if (Array.isArray(data) && data.length > 0) {
          setSections(data);
        }
      })
      .catch(() => {
        // Fallback predeterminado
      });
  }, []);

  const todayStr = new Intl.DateTimeFormat("es-MX", {
    weekday: "long",
    year: "numeric",
    month: "long",
    day: "numeric",
  }).format(new Date());

  const capitalize = (s: string) => s.charAt(0).toUpperCase() + s.slice(1);

  return (
    <Box sx={{ minHeight: "100vh", backgroundColor: "#f8fafc", display: "flex", flexDirection: "column" }}>
      {/* Top Banner Branding */}
      <Box sx={{ backgroundColor: "#0f172a", color: "#f8fafc", py: 1, px: 2, borderBottom: "1px solid #1e293b" }}>
        <Container maxWidth="lg" sx={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
          <Box sx={{ display: "flex", alignItems: "center", gap: 1.5 }}>
            <Box
              sx={{
                width: 8,
                height: 8,
                borderRadius: "50%",
                backgroundColor: "#22c55e",
                boxShadow: "0 0 10px #22c55e",
              }}
            />
            <Typography variant="caption" sx={{ fontWeight: 600, letterSpacing: 0.5, color: "#94a3b8" }}>
              EDICIÓN MATUTINA OFICIAL · {capitalize(todayStr)}
            </Typography>
          </Box>
          <Box sx={{ display: "flex", alignItems: "center", gap: 1 }}>
            <a
              href="/authentication/sign-in"
              style={{
                fontSize: "0.75rem",
                color: "#cbd5e1",
                textDecoration: "none",
                display: "flex",
                alignItems: "center",
                gap: "4px",
              }}
            >
              <span className="material-symbols-outlined" style={{ fontSize: "14px" }}>
                lock
              </span>
              Portal de Redacción
            </a>
          </Box>
        </Container>
      </Box>

      {/* Main Header & Section Navigation */}
      <AppBar
        position="sticky"
        elevation={0}
        sx={{
          backgroundColor: "#ffffff",
          borderBottom: "1px solid #e2e8f0",
          color: "#0f172a",
        }}
      >
        <Container maxWidth="lg">
          <Toolbar disableGutters sx={{ py: 1.5, display: "flex", flexWrap: "wrap", justifyContent: "space-between", gap: 2 }}>
            {/* Logo */}
            <a href="/brief/politica" style={{ textDecoration: "none", color: "inherit", display: "flex", alignItems: "center", gap: "10px" }}>
              <Box
                sx={{
                  backgroundColor: "#0284c7",
                  color: "#ffffff",
                  p: 0.8,
                  borderRadius: 1.5,
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "center",
                }}
              >
                <span className="material-symbols-outlined" style={{ fontSize: "22px" }}>
                  newspaper
                </span>
              </Box>
              <Box>
                <Typography variant="h6" sx={{ fontWeight: 800, lineHeight: 1.1, letterSpacing: -0.5 }}>
                  ExposureIQ
                </Typography>
                <Typography variant="caption" sx={{ color: "#64748b", fontWeight: 500, fontSize: "0.7rem" }}>
                  Brief Informativo & Monitoreo
                </Typography>
              </Box>
            </a>

            {/* Navigation Pills (Stable URLs) */}
            <Box sx={{ display: "flex", alignItems: "center", gap: 1, overflowX: "auto", py: 0.5 }}>
              {sections.map((sec) => {
                const isActive = pathname.includes(sec.section_key);
                return (
                  <a
                    key={sec.section_key}
                    href={`/brief/${sec.section_key}`}
                    style={{ textDecoration: "none" }}
                  >
                    <Chip
                      clickable
                      icon={
                        <span
                          className="material-symbols-outlined"
                          style={{
                            fontSize: "18px",
                            color: isActive ? "#ffffff" : sec.color || "#0284c7",
                          }}
                        >
                          {sec.icon === "newspaper"
                            ? "newspaper"
                            : sec.icon === "trending-up"
                            ? "trending_up"
                            : sec.icon === "shield-alert"
                            ? "security"
                            : "account_balance"}
                        </span>
                      }
                      label={sec.section_key === "primeras_planas" ? "Primeras Planas" : sec.title.split(" ")[0]}
                      sx={{
                        fontWeight: 700,
                        fontSize: "0.85rem",
                        px: 1,
                        py: 2.2,
                        borderRadius: "10px",
                        backgroundColor: isActive ? sec.color || "#0284c7" : "#f1f5f9",
                        color: isActive ? "#ffffff" : "#334155",
                        border: isActive ? `1px solid ${sec.color || "#0284c7"}` : "1px solid #e2e8f0",
                        transition: "all 0.2s ease-in-out",
                        "&:hover": {
                          backgroundColor: isActive ? sec.color || "#0284c7" : "#e2e8f0",
                        },
                      }}
                    />
                  </a>
                );
              })}
            </Box>
          </Toolbar>
        </Container>
      </AppBar>

      {/* Content Area */}
      <Box sx={{ flex: 1, py: 4 }}>
        <Container maxWidth="lg">{children}</Container>
      </Box>

      {/* Clean Public Footer */}
      <Box sx={{ backgroundColor: "#ffffff", borderTop: "1px solid #e2e8f0", py: 3, mt: "auto" }}>
        <Container maxWidth="lg" sx={{ display: "flex", flexWrap: "wrap", justifyContent: "space-between", alignItems: "center", gap: 2 }}>
          <Typography variant="body2" sx={{ color: "#64748b" }}>
            © {new Date().getFullYear()} ExposureIQ. Servicio de Inteligencia Estratégica y Regulatoria.
          </Typography>
          <Box sx={{ display: "flex", gap: 2 }}>
            <Typography variant="caption" sx={{ color: "#94a3b8" }}>
              URLs Estables para WhatsApp
            </Typography>
            <Typography variant="caption" sx={{ color: "#94a3b8" }}>
              ·
            </Typography>
            <Typography variant="caption" sx={{ color: "#94a3b8" }}>
              Navegación Semanal
            </Typography>
          </Box>
        </Container>
      </Box>
    </Box>
  );
}
