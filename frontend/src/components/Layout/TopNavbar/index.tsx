// File path: /styles/top-navbar.scss

"use client";

import React, { useEffect } from "react";
import { AppBar, Toolbar, IconButton, Box } from "@mui/material";
import Image from "next/image";
import Link from "next/link";
import Tooltip from "@mui/material/Tooltip";
import Profile from "./Profile";

interface TopNavbarProps {
  toggleActive: () => void;
}

const TopNavbar: React.FC<TopNavbarProps> = ({ toggleActive }) => {
  useEffect(() => {
    let elementId = document.getElementById("navbar");
    const handleScroll = () => {
      if (window.scrollY > 100) {
        elementId?.classList.add("sticky");
      } else {
        elementId?.classList.remove("sticky");
      }
    };
    document.addEventListener("scroll", handleScroll);
    return () => {
      document.removeEventListener("scroll", handleScroll);
    };
  }, []);

  return (
    <div className="top-navbar-dark">
      <AppBar
        id="navbar"
        color="inherit"
        sx={{
          backgroundColor: "#fff",
          boxShadow: "0 1px 4px rgba(0,0,0,0.05)",
          borderRadius: "0 0 15px 15px",
          py: { xs: "12px", sm: "6px" },
          px: "0 !important",
          width: "initial",
          zIndex: "489",
        }}
        className="top-navbar"
      >
        <Box className="top-navbar-content">
          <Toolbar
            sx={{
              display: "flex",
              justifyContent: "space-between",
              alignItems: "center",
              px: { xs: 2, sm: 3 },
            }}
          >
            <Box
              sx={{
                display: "flex",
                alignItems: "center",
                gap: { xs: "10px", sm: "15px" },
              }}
            >
              <Box className="logos">
                <Link href="/boletines" className="logo">
                  <Image
                    src="/images/logo.svg"
                    alt="ExposureIQ"
                    width={110}
                    height={28}
                  />
                </Link>
                <Link href="/boletines" className="white-logo">
                  <Image
                    src="/images/white-logo.svg"
                    alt="ExposureIQ"
                    width={110}
                    height={28}
                  />
                </Link>
              </Box>

              <Tooltip title="Ocultar / Mostrar Menú" arrow>
                <IconButton
                  size="small"
                  edge="start"
                  color="inherit"
                  onClick={toggleActive}
                  className="top-burger"
                >
                  <i className="material-symbols-outlined">menu</i>
                </IconButton>
              </Tooltip>
            </Box>

            <Box
              sx={{
                display: "flex",
                alignItems: "center",
              }}
            >
              {/* Espacio exclusivo del usuario */}
              <Profile />
            </Box>
          </Toolbar>
        </Box>
      </AppBar>
    </div>
  );
};

export default TopNavbar;
