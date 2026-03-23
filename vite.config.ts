import { defineConfig } from "vite";

export default defineConfig({
  base: process.env.VITE_BASE_URL ?? "/",
  server: {
    port: 5173,
    host: "127.0.0.1"
  }
});

