import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App.tsx";
import "./index.css";
import { RouterProvider, createBrowserRouter } from "react-router-dom";
import ErrorPage from "./ErrorPage.tsx";
import { Toaster } from "react-hot-toast";
import VisualEditorOuter from "./routes/visual-editor/VisualEditor.tsx";
import OcelInfoViewer from "./routes/ocel-info/OcelInfoViewer.tsx";
const router = createBrowserRouter([
  {
    path: "/",
    element: <App />,
    errorElement: <ErrorPage />,
    children: [
      { path: "/beta", element: <VisualEditorOuter /> },
      { path: "/ocel-info", element: <OcelInfoViewer /> },
    ],
  },
]);

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <Toaster />
    <RouterProvider router={router} />
  </React.StrictMode>,
);
