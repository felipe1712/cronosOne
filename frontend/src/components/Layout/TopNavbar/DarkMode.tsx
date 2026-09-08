"use client";

import React, { useState, useEffect } from "react";
import { Box } from "@mui/material";

const DarkMode: React.FC = () => {
  // Light/Dark Mode state
  const [isDarkMode, setIsDarkMode] = useState<boolean>(false);

  useEffect(() => {
    // Retrieve the user's preference from local storage
    const storedPreference = localStorage.getItem("theme");
    if (storedPreference === "dark") {
      setIsDarkMode(true);
    }
  }, []);

  const handleToggle = () => {
    setIsDarkMode(!isDarkMode);
  };

  useEffect(() => {
    // Update the user's preference in local storage
    localStorage.setItem("theme", isDarkMode ? "dark" : "light");

    // Update the class on the <html> element to apply the selected mode
    const htmlElement = document.querySelector("html");
    if (htmlElement) {
      if (isDarkMode) {
        htmlElement.classList.add("dark-theme");
      } else {
        htmlElement.classList.remove("dark-theme");
      }
    }
  }, [isDarkMode]);

  return (
    <>
      <Box
        className={`th-toggle-mode ${
          isDarkMode ? "active" : ""
        }`}
        onClick={handleToggle}
        sx={{
          width: "40px",
          height: "40px",
          display: "flex",
          justifyContent: "center",
          alignItems: "center",
        }}
      >
        <i className="ri-sun-line"></i>
        <i className="ri-moon-line"></i>
      </Box>
    </>
  );
};

export default DarkMode;
