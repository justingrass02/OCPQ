import React from "react";
import ReactDOM from "react-dom/client";
import { Toaster } from "react-hot-toast";
import {
  API_WEB_SERVER_BACKEND_PROVIDER,
  BackendProviderContext,
} from "./BackendProviderContext.ts";
import "./index.css";
import { MainRouterProvider } from "./router.tsx";

// eslint-disable-next-line @typescript-eslint/no-non-null-assertion
ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <BackendProviderContext.Provider value={API_WEB_SERVER_BACKEND_PROVIDER}>
      <Toaster position="bottom-left" />
      <MainRouterProvider />
    </BackendProviderContext.Provider>
  </React.StrictMode>,
);
