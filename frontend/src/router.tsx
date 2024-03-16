import { RouterProvider, createBrowserRouter } from "react-router-dom";
import App from "./App.tsx";
import OuterVisualEditor from "./routes/visual-editor/outer-visual-editor/OuterVisualEditor.tsx";
import OcelInfoViewer from "./routes/ocel-info/OcelInfoViewer.tsx";
import ErrorPage from "./ErrorPage.tsx";

// export const router = createBrowserRouter([
//   {
//     path: "/",
//     element: <App />,
//     errorElement: <ErrorPage />,
//     children: [
//       { path: "/beta", element: <OuterVisualEditor /> },
//       { path: "/ocel-info", element: <OcelInfoViewer /> },
//     ],
//   },
// ]);

const router = createBrowserRouter([
  {
    path: "/",
    element: <App />,
    errorElement: <ErrorPage />,
    children: [
      { path: "/beta", element: <OuterVisualEditor /> },
      { path: "/ocel-info", element: <OcelInfoViewer /> },
    ],
  },
]);

export const MainRouterProvider = () => <RouterProvider router={router} />;
