import React from "react";
import ReactDOM from "react-dom/client";
import { Toaster } from "react-hot-toast";
import {
  BackendProviderContext,
  DEFAULT_BACKEND_URL,
  getAPIServerBackendProvider,
} from "./BackendProviderContext.ts";
import "./index.css";
import { MainRouterProvider } from "./router.tsx";

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <BackendProviderContext.Provider value={getAPIServerBackendProvider(DEFAULT_BACKEND_URL)}>
      <Toaster position="bottom-left" />
      <MainRouterProvider />
    </BackendProviderContext.Provider>
  </React.StrictMode>,
);
