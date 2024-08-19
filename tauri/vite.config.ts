import path from "path";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

export default defineConfig({
  server: {
    port: 4565,
    fs: {
      allow: [".","../frontend"]
    }
  },
  // build: {
  //   sourcemap: true,
  // },
  plugins: [react()],
  resolve: {
    alias: {
      "$": path.resolve(__dirname, "../frontend/src"),
      "@": path.resolve(__dirname, "../frontend/src"),
    },
  },
  optimizeDeps: {
    // exclude: ["monaco-editor","@monaco-editor/react"]
  },
});
