"use client";

import * as React from "react";
import { Box, Typography } from "@mui/material";

const Footer: React.FC = () => {
  return (
    <Box
      className="footer-area"
      sx={{
        textAlign: "center",
        bgcolor: "#fff",
        borderRadius: "7px 7px 0 0",
        padding: "15px 25px",
        mt: 4,
      }}
    >
      <Typography variant="body2" sx={{ color: "#64748b" }}>
        © {new Date().getFullYear()} <strong>ExposureIQ</strong> — Servicio de Monitoreo Ejecutivo e Inteligencia de Riesgo.
      </Typography>
    </Box>
  );
};

export default Footer;
