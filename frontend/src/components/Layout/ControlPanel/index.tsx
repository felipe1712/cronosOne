// File Path: /styles/control-panel.scss

"use client";

import React, { useState } from "react";
import IconButton from "@mui/material/IconButton";
import { Tooltip, Button } from "@mui/material";
import RTLMode from "./RTLMode";
import DarkMode from "./DarkMode";
import OnlySidebarDarkMode from "./OnlySidebarDarkMode";
import OnlyHeaderDarkMode from "./OnlyHeaderDarkMode";
import CompactSidebar from "./CompactSidebar";
import HorizontalLayout from "./HorizontalLayout";

const ControlPanel: React.FC = () => {
  const [isControlPanel, setControlPanel] = useState<boolean>(false);

  const handleToggleControlPanel = () => {
    setControlPanel(!isControlPanel);
  };

  return (
    <>
      <Tooltip title="Control Panel" placement="left" arrow>
        <IconButton
          onClick={handleToggleControlPanel}
          size="small"
          sx={{
            width: "30px",
            height: "30px",
            p: 0,
          }}
          className="t-settings-btn"
        >
          <i className="material-symbols-outlined text-body">settings</i>
        </IconButton>
      </Tooltip>

      <div
        className={`settings-sidebar bg-white transition ${
          isControlPanel ? "active" : ""
        }`}
      >
        <div className="settings-header bg-primary">
          <h4 className="text-white">Theme Settings</h4>
          <button
            className="close-btn text-white"
            type="button"
            onClick={handleToggleControlPanel}
          >
            <i className="material-symbols-outlined">close</i>
          </button>
        </div>

        <div className="settings-body">
          <RTLMode />

          <div className="border-bottom" style={{ margin: "15px 0" }}></div>

          <DarkMode />

          <div className="border-bottom" style={{ margin: "15px 0" }}></div>

          <HorizontalLayout />
          
          <div className="border-bottom" style={{ margin: "15px 0" }}></div>

          <CompactSidebar />

          <div className="border-bottom" style={{ margin: "15px 0" }}></div>

          <OnlySidebarDarkMode />

          <div className="border-bottom" style={{ margin: "15px 0" }}></div>

          <OnlyHeaderDarkMode />

          <div className="border-bottom" style={{ margin: "15px 0" }}></div>

          <a href="https://1.envato.market/QyqV6P" target="_blank">
            <Button
              variant="contained"
              color="primary"
              sx={{
                textTransform: "capitalize",
                color: "#fff !important",
              }}
            >
              Buy Trezo
            </Button>
          </a>
        </div>
      </div>
    </>
  );
};

export default ControlPanel;
