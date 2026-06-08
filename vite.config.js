import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

export default defineConfig({
  plugins: [svelte()],
  server: { port: 1422, strictPort: true },
  cacheDir: "/private/tmp/ziploom-vite-cache",
  build: {
    target: "esnext",
    emptyOutDir: false,
    outDir: process.env.ZIPLOOM_OUTDIR || "dist",
  },
});
