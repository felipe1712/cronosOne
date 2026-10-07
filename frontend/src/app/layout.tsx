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
        <meta httpEquiv="Cache-Control" content="no-cache, no-store, must-revalidate" />
        <meta httpEquiv="Pragma" content="no-cache" />
        <meta httpEquiv="Expires" content="0" />
        <link
          rel="stylesheet"
          href="https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:opsz,wght,FILL,GRAD@24,400,0,0"
        />
        <script
          dangerouslySetInnerHTML={{
            __html: `
              (function() {
                function handleChunkError(e) {
                  var isChunk = false;
                  if (e) {
                    var msg = e.message || (e.reason && (e.reason.message || e.reason.toString())) || (e.error && e.error.message) || '';
                    if (msg.indexOf('ChunkLoadError') !== -1 || msg.indexOf('Failed to load chunk') !== -1) {
                      isChunk = true;
                    }
                  }
                  if (!isChunk) return;

                  var key = 'next_chunk_error_last_reload';
                  var last = sessionStorage.getItem(key);
                  var now = Date.now();
                  if (last && (now - parseInt(last, 10)) < 10000) {
                    console.warn('[ExposureIQ] ChunkLoadError recurrente suprimido para evitar bucle de recarga.');
                    return;
                  }
                  sessionStorage.setItem(key, now.toString());

                  if ('caches' in window) {
                    try {
                      caches.keys().then(function(names) {
                        for (var i = 0; i < names.length; i++) caches.delete(names[i]);
                      });
                    } catch(err) {}
                  }

                  var target = window.location.origin + window.location.pathname;
                  var search = window.location.search || '';
                  search = search.replace(/[?&]_ts=[^&]*/g, '');
                  var sep = search ? (search.indexOf('?') === -1 ? '?' : '&') : '?';
                  window.location.replace(target + search + sep + '_ts=' + now);
                }

                window.addEventListener('error', handleChunkError);
                window.addEventListener('unhandledrejection', handleChunkError);
              })();
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
