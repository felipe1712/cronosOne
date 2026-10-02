"use client";

import React, { useState, useEffect, ReactNode } from "react";
import { usePathname, useRouter } from "next/navigation";
import LeftSidebarMenu from "@/components/Layout/LeftSidebarMenu";
import TopNavbar from "./../components/Layout/TopNavbar/index";
import Footer from "@/components/Layout/Footer";
import { Box, CircularProgress } from "@mui/material";

interface LayoutProviderProps {
  children: ReactNode;
}

const LayoutProvider: React.FC<LayoutProviderProps> = ({ children }) => {
  const [active, setActive] = useState<boolean>(false);
  const [isCheckingAuth, setIsCheckingAuth] = useState<boolean>(true);
  const pathname = usePathname();
  const router = useRouter();

  const toggleActive = () => {
    setActive(!active);
  };

  const isAuthPage = pathname.startsWith("/authentication");

  useEffect(() => {
    const handleChunkError = (event: ErrorEvent) => {
      const msg = event?.message || "";
      if (
        msg.includes("ChunkLoadError") ||
        msg.includes("Failed to load chunk") ||
        msg.includes("Loading chunk")
      ) {
        console.warn("ChunkLoadError detectado por nuevo build. Recargando página...", msg);
        window.location.reload();
      }
    };

    const handleRejection = (event: PromiseRejectionEvent) => {
      const reason = event?.reason?.message || String(event?.reason || "");
      if (
        reason.includes("ChunkLoadError") ||
        reason.includes("Failed to load chunk") ||
        reason.includes("Loading chunk")
      ) {
        console.warn("Rechazo de chunk detectado por nuevo build. Recargando página...", reason);
        window.location.reload();
      }
    };

    window.addEventListener("error", handleChunkError);
    window.addEventListener("unhandledrejection", handleRejection);
    return () => {
      window.removeEventListener("error", handleChunkError);
      window.removeEventListener("unhandledrejection", handleRejection);
    };
  }, []);

  useEffect(() => {
    if (!isAuthPage) {
      const token = typeof window !== "undefined" ? localStorage.getItem("exposureiq_token") : null;
      if (!token) {
        router.replace("/authentication/sign-in");
        return;
      }
    }
    setIsCheckingAuth(false);
  }, [pathname, isAuthPage, router]);

  // Si no está autenticado y no es página de autenticación, mostrar carga mientras redirige
  if (!isAuthPage && isCheckingAuth) {
    return (
      <Box
        sx={{
          display: "flex",
          justifyContent: "center",
          alignItems: "center",
          minHeight: "100vh",
          backgroundColor: "#f8fafc",
        }}
      >
        <CircularProgress />
      </Box>
    );
  }

  return (
    <div className={`main-wrapper-content ${active ? "active" : ""}`}>
      {!isAuthPage && (
        <>
          <TopNavbar toggleActive={toggleActive} />
          <LeftSidebarMenu toggleActive={toggleActive} />
        </>
      )}

      <div className="main-content">
        {children}

        {!isAuthPage && <Footer />}
      </div>
    </div>
  );
};

export default LayoutProvider;
