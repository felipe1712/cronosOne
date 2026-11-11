"use client";

import { useEffect } from "react";
import { useRouter } from "next/navigation";
import { Box, CircularProgress, Typography } from "@mui/material";

export default function BriefRootPage() {
  const router = useRouter();

  useEffect(() => {
    router.replace("/brief/primeras_planas");
  }, [router]);

  return (
    <Box sx={{ display: "flex", flexDirection: "column", alignItems: "center", justifyContent: "center", py: 12 }}>
      <CircularProgress size={40} sx={{ color: "#0284c7" }} />
      <Typography variant="body2" sx={{ mt: 2, color: "#64748b" }}>
        Cargando canal informativo...
      </Typography>
    </Box>
  );
}
