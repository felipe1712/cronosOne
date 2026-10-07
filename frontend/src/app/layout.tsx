import "swiper/css";
import "swiper/css/bundle";
import 'remixicon/fonts/remixicon.css';
import 'react-datetime-picker/dist/DateTimePicker.css';
import 'react-calendar/dist/Calendar.css';
import 'react-clock/dist/Clock.css';
import '../../node_modules/boxicons/css/boxicons.min.css';
import '../../styles/front-pages.css';
import "../../styles/control-panel.css";
import "../../styles/left-sidebar-menu.css";
import "../../styles/top-navbar.css";
import "../../styles/crypto-dashboard.css";
import "../../styles/chat.css";
import "../../styles/horizontal-navbar.css";
import "../../styles/globals.css";

// globals dark Mode CSS
import "../../styles/dark.css";
// globals RTL Mode CSS
import "../../styles/rtl.css";

import * as React from "react";
import { AppRouterCacheProvider } from "@mui/material-nextjs/v14-appRouter";
import { ThemeProvider } from "@mui/material/styles";
import CssBaseline from "@mui/material/CssBaseline";
import theme from "@/theme";
import LayoutProvider from "@/providers/LayoutProvider";

export const metadata = {
  title: "ExposureIQ — Monitoreo Ejecutivo",
  description: "Servicio de Monitoreo Ejecutivo e Inteligencia de Riesgo para Aseguradoras",
};

export default function RootLayout(props: { children: React.ReactNode }) {
  return (
    <html lang="en">
      <head>
        <link
          rel="stylesheet"
          href="https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:opsz,wght,FILL,GRAD@24,400,0,0"
        />
        <script
          dangerouslySetInnerHTML={{
            __html: `
              window.addEventListener('error', function(e) {
                if (e && ((e.message && e.message.indexOf('ChunkLoadError') !== -1) || (e.error && e.error.name === 'ChunkLoadError'))) {
                  var key = 'next_chunk_error_reload';
                  var last = sessionStorage.getItem(key);
                  var now = Date.now();
                  if (last && (now - parseInt(last, 10)) < 15000) {
                    console.warn('[ExposureIQ] ChunkLoadError recurrente evitado para prevenir loop.');
                    return;
                  }
                  sessionStorage.setItem(key, now.toString());
                  var url = window.location.pathname + (window.location.search ? window.location.search + '&' : '?') + '_r=' + now;
                  window.location.replace(url);
                }
              });
            `,
          }}
        />
      </head>
      <body>
        <AppRouterCacheProvider options={{ enableCssLayer: true }}>
          <ThemeProvider theme={theme}>
            {/* CssBaseline kickstart an elegant, consistent, and simple baseline to build upon. */}
            <CssBaseline />

            <LayoutProvider>{props.children}</LayoutProvider>
          </ThemeProvider>
        </AppRouterCacheProvider>
      </body>
    </html>
  );
}
