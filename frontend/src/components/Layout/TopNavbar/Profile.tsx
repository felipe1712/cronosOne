"use client";

import * as React from "react";
import {
  IconButton,
  Typography,
  Box,
  Tooltip,
  Avatar,
  Menu,
  MenuItem,
  ListItemIcon,
  Divider,
} from "@mui/material";
import Logout from "@mui/icons-material/Logout";
import KeyboardArrowDownIcon from "@mui/icons-material/KeyboardArrowDown";
import { useRouter } from "next/navigation";
import { removeToken } from "@/lib/api";

const Profile: React.FC = () => {
  const router = useRouter();
  const [anchorEl, setAnchorEl] = React.useState<null | HTMLElement>(null);
  const [userName, setUserName] = React.useState<string>("Olivia");
  const [userRole, setUserRole] = React.useState<string>("Dirección de Operaciones");
  const open = Boolean(anchorEl);

  React.useEffect(() => {
    if (typeof window !== "undefined") {
      try {
        const storedUser = localStorage.getItem("exposureiq_user");
        if (storedUser) {
          const parsed = JSON.parse(storedUser);
          if (parsed.nombre) setUserName(parsed.nombre);
          if (parsed.rol) setUserRole(parsed.rol.toUpperCase());
        }
      } catch {
        // Fallback a valores por defecto
      }
    }
  }, []);

  const handleClick = (event: React.MouseEvent<HTMLElement>) => {
    setAnchorEl(event.currentTarget);
  };

  const handleClose = () => {
    setAnchorEl(null);
  };

  const handleLogout = () => {
    handleClose();
    removeToken();
    router.push("/authentication/sign-in");
  };

  return (
    <>
      <Tooltip title="Perfil de Usuario">
        <IconButton
          onClick={handleClick}
          size="small"
          sx={{
            p: "4px 8px",
            borderRadius: "8px",
            "&:hover": { backgroundColor: "rgba(0,0,0,0.04)" },
          }}
          aria-controls={open ? "account-menu" : undefined}
          aria-haspopup="true"
          aria-expanded={open ? "true" : undefined}
        >
          <Avatar
            src="/images/admin.png"
            alt="Olivia"
            sx={{
              width: { xs: "34px", sm: "38px" },
              height: { xs: "34px", sm: "38px" },
              border: "2px solid #C2CDFF",
              mr: 1,
            }}
          />
          <Typography
            variant="subtitle2"
            sx={{
              fontWeight: "600",
              fontSize: "14px",
              display: { xs: "none", sm: "block" },
              color: "#1e293b",
              mr: 0.5,
            }}
          >
            Olivia
          </Typography>
          <KeyboardArrowDownIcon sx={{ fontSize: "18px", color: "#64748b" }} />
        </IconButton>
      </Tooltip>

      <Menu
        anchorEl={anchorEl}
        id="account-menu"
        open={open}
        onClose={handleClose}
        onClick={handleClose}
        PaperProps={{
          elevation: 3,
          sx: {
            borderRadius: "10px",
            minWidth: "220px",
            overflow: "visible",
            mt: 1.5,
          },
        }}
        transformOrigin={{ horizontal: "right", vertical: "top" }}
        anchorOrigin={{ horizontal: "right", vertical: "bottom" }}
      >
        <MenuItem sx={{ py: 1.5, px: 2, cursor: "default", "&:hover": { backgroundColor: "transparent" } }}>
          <Box>
            <Typography variant="subtitle2" sx={{ fontWeight: "700", color: "#1e293b" }}>
              {userName}
            </Typography>
            <Typography variant="caption" sx={{ color: "#64748b", display: "block" }}>
              {userRole}
            </Typography>
          </Box>
        </MenuItem>

        <Divider sx={{ my: 0.5 }} />

        <MenuItem
          onClick={handleLogout}
          sx={{
            py: 1,
            px: 2,
            color: "#dc2626",
            "&:hover": { backgroundColor: "#fef2f2" },
          }}
        >
          <ListItemIcon sx={{ color: "#dc2626", minWidth: "32px" }}>
            <Logout fontSize="small" />
          </ListItemIcon>
          <Typography variant="body2" sx={{ fontWeight: 600 }}>
            Cerrar Sesión
          </Typography>
        </MenuItem>
      </Menu>
    </>
  );
};

export default Profile;
