// File path: /styles/left-sidebar-menu.scss

"use client";

import React from "react";
import Link from "next/link";
import Image from "next/image";
import { usePathname } from "next/navigation";
import { styled } from "@mui/material/styles";
import ArrowForwardIosSharpIcon from "@mui/icons-material/ArrowForwardIosSharp";
import MuiAccordion, { AccordionProps } from "@mui/material/Accordion";
import MuiAccordionSummary, {
  AccordionSummaryProps,
} from "@mui/material/AccordionSummary";
import MuiAccordionDetails from "@mui/material/AccordionDetails";
import { Box, Typography } from "@mui/material";

const Accordion = styled((props: AccordionProps) => (
  <MuiAccordion disableGutters elevation={0} square {...props} />
))(({ theme }) => ({
  border: `1px solid ${theme.palette.divider}`,
  "&:not(:last-child)": {
    borderBottom: 0,
  },
  "&::before": {
    display: "none",
  },
}));

const AccordionSummary = styled((props: AccordionSummaryProps) => (
  <MuiAccordionSummary
    expandIcon={<ArrowForwardIosSharpIcon sx={{ fontSize: "0.9rem" }} />}
    {...props}
  />
))(({ theme }) => ({
  backgroundColor: theme.palette.mode === "dark" ? "#3a4252" : "#f6f7f9",
  flexDirection: "row-reverse",
  "& .MuiAccordionSummary-expandIconWrapper.Mui-expanded": {
    transform: "rotate(90deg)",
  },
  "& .MuiAccordionSummary-content": {},
}));

const AccordionDetails = styled(MuiAccordionDetails)(({ theme }) => ({
  padding: theme.spacing(2),
}));

interface LeftSidebarProps {
  toggleActive: () => void;
}

const LeftSidebarMenu: React.FC<LeftSidebarProps> = ({ toggleActive }) => {
  const pathname = usePathname();
  const [expanded, setExpanded] = React.useState<string | false>("panelExposureIQ");

  const handleChange =
    (panel: string) => (event: React.SyntheticEvent, newExpanded: boolean) => {
      setExpanded(newExpanded ? panel : false);
    };

  return (
    <Box className="left-sidebar-menu">
      <Box className="logo">
        <Link href="/boletines">
          <Image
            src="/images/logo-icon.svg"
            alt="ExposureIQ"
            width={26}
            height={26}
          />
          <Typography component="span" sx={{ fontWeight: 700, ml: 1, letterSpacing: "-0.5px" }}>
            ExposureIQ
          </Typography>
        </Link>
      </Box>

      <Box className="burger-menu" onClick={toggleActive}>
        <Typography component="span" className="top-bar"></Typography>
        <Typography component="span" className="middle-bar"></Typography>
        <Typography component="span" className="bottom-bar"></Typography>
      </Box>

      <Box className="sidebar-inner">
        <Box className="sidebar-menu">
          <Typography
            className="sub-title"
            sx={{
              display: "block",
              fontWeight: "700",
              color: "#605DFF !important",
              textTransform: "uppercase",
              fontSize: "0.75rem",
              letterSpacing: "1px",
              mb: 1,
            }}
          >
            INTELIGENCIA & OPERACIONES
          </Typography>

          <Accordion
            expanded={expanded === "panelExposureIQ"}
            onChange={handleChange("panelExposureIQ")}
            className="mat-accordion"
          >
            <AccordionSummary
              className="mat-summary"
              aria-controls="panelExposureIQ-content"
              id="panelExposureIQ-header"
            >
              <i className="material-symbols-outlined">shield</i>
              <Typography component="span" className="title">
                Monitoreo Ejecutivo
              </Typography>
            </AccordionSummary>

            <AccordionDetails className="mat-details">
              <ul className="sidebar-sub-menu">
                <li className="sidemenu-item">
                  <Link
                    href="/boletines"
                    className={`sidemenu-link ${
                      pathname === "/boletines" ? "active" : ""
                    }`}
                  >
                    Boletines Coparmex
                  </Link>
                </li>
                <li className="sidemenu-item">
                  <Link
                    href="/osint"
                    className={`sidemenu-link ${
                      pathname === "/osint" ? "active" : ""
                    }`}
                  >
                    Inteligencia & OSINT
                  </Link>
                </li>
                <li className="sidemenu-item">
                  <Link
                    href="/waha"
                    className={`sidemenu-link ${
                      pathname === "/waha" ? "active" : ""
                    }`}
                  >
                    Canal WhatsApp (WAHA)
                  </Link>
                </li>
                <li className="sidemenu-item">
                  <Link
                    href="/configuracion"
                    className={`sidemenu-link ${
                      pathname === "/configuracion" ? "active" : ""
                    }`}
                  >
                    Configuración
                  </Link>
                </li>
              </ul>
            </AccordionDetails>
          </Accordion>

          <Typography
            className="sub-title"
            sx={{
              display: "block",
              fontWeight: "500",
              textTransform: "uppercase",
              fontSize: "0.75rem",
              letterSpacing: "1px",
              mt: 3,
              mb: 1,
            }}
          >
            ACCESO
          </Typography>

          <ul className="sidebar-sub-menu" style={{ paddingLeft: 0, listStyle: "none" }}>
            <li className="sidemenu-item">
              <Link
                href="/authentication/sign-in/"
                className={`sidemenu-link ${
                  pathname === "/authentication/sign-in/" ? "active" : ""
                }`}
              >
                <i className="material-symbols-outlined" style={{ marginRight: 8, fontSize: "1.1rem" }}>
                  login
                </i>
                Iniciar Sesión
              </Link>
            </li>
          </ul>
        </Box>
      </Box>
    </Box>
  );
};

export default LeftSidebarMenu;
