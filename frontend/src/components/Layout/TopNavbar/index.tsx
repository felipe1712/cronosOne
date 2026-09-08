// File path: /styles/top-navbar.scss

"use client";

import React, { useEffect } from "react";
import { AppBar, Toolbar, IconButton, Box } from "@mui/material";
import Image from "next/image";
import Link from "next/link";
import Tooltip from "@mui/material/Tooltip";
import SearchForm from "./SearchForm";
import Notifications from "./Notifications";
import Profile from "./Profile";
import FullscreenButton from "./FullscreenButton";
import AppsMenu from "./AppsMenu";
import ChooseLanguage from "./ChooseLanguage/index";
import ControlPanel from "../ControlPanel";
import DarkMode from "./DarkMode";
import HorizontalNavbar from "./HorizontalNavbar";

interface TopNavbarProps {
  toggleActive: () => void;
}

const TopNavbar: React.FC<TopNavbarProps> = ({ toggleActive }) => {
  useEffect(() => {
    let elementId = document.getElementById("navbar");
    document.addEventListener("scroll", () => {
      if (window.scrollY > 100) {
        elementId?.classList.add("sticky");
      } else {
        elementId?.classList.remove("sticky");
      }
    });
  });

  return (
    <>
      <div className="top-navbar-dark">
        <AppBar
          id="navbar"
          color="inherit"
          sx={{
            backgroundColor: "#fff",
            boxShadow: "initial",
            borderRadius: "0 0 15px 15px",
            py: { xs: "15px", sm: "3px" },
            px: "0 !important",
            width: "initial",
            zIndex: "489",
          }}
          className="top-navbar"
        >
          <Box className="top-navbar-content">
            <Toolbar
              sx={{
                display: { xs: "block", sm: "flex" },
                justifyContent: { xs: "center", sm: "space-between" },
              }}
            >
              <Box
                sx={{
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "center",
                  gap: { xs: "10px", sm: "5px", md: "15px" },
                }}
              >
                <Box className="logos">
                  <Link href="/boletines" className="logo">
                    <Image
                      src="/images/logo.svg"
                      alt="logo"
                      width={100}
                      height={26}
                    />
                  </Link>

                  <Link href="/boletines" className="white-logo">
                    <Image
                      src="/images/white-logo.svg"
                      alt="logo"
                      width={100}
                      height={26}
                    />
                  </Link>
                </Box>

                <Tooltip title="Hide/Show" arrow>
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

                {/* Search form */}
                <SearchForm />

                {/* AppsMenu */}
                <AppsMenu />
              </Box>

              <Box
                sx={{
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "center",
                  gap: { xs: "8px", sm: "8px", lg: "15px" },
                  mt: { xs: "10px", sm: "0px" },
                }}
              >
                {/* DarkMode */}
                <DarkMode />

                {/* ChooseLanguage */}
                <ChooseLanguage />

                {/* FullscreenButton */}
                <FullscreenButton />

                {/* Notifications */}
                <Notifications />

                {/* Profile */}
                <Profile />

                {/* ControlPanel */}
                <ControlPanel />
              </Box>
            </Toolbar>
          </Box>

          <HorizontalNavbar />
        </AppBar>
      </div>
    </>
  );
};

export default TopNavbar;
