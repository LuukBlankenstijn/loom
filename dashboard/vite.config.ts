import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import path from "path";
import type { Plugin } from "vite";
import fs from "fs";

// Rewrite map-editor HTML to use /editor/ base path
function rewriteEditorHtml(html: string): string {
  return (
    html
      .replace('<base href="/" />', '<base href="/editor/" />')
      // Only replace hrefs that aren't already /editor/
      .replace(/href="\/(?!editor\/)/g, 'href="/editor/')
      .replace(/from '\/(?!editor\/)/g, "from '/editor/")
      .replace(/module_or_path: '\/(?!editor\/)/g, "module_or_path: '/editor/")
  );
}

// Plugin to serve map-editor WASM files (dev) and copy them (build)
function serveMapEditor(): Plugin {
  const mapEditorDist = path.resolve(__dirname, "../map-editor/dist");

  return {
    name: "serve-map-editor",
    // Production: copy files after build
    closeBundle() {
      const dest = path.resolve(__dirname, "dist/editor");

      if (!fs.existsSync(mapEditorDist)) {
        console.warn("map-editor/dist not found, skipping editor bundle");
        return;
      }

      // Copy all files
      fs.cpSync(mapEditorDist, dest, { recursive: true });

      // Rewrite paths in index.html
      const indexPath = path.join(dest, "index.html");
      if (fs.existsSync(indexPath)) {
        const html = fs.readFileSync(indexPath, "utf-8");
        fs.writeFileSync(indexPath, rewriteEditorHtml(html));
      }

      console.log("Bundled map-editor to dist/editor/");
    },
    // Development: serve files via middleware
    configureServer(server) {
      // Serve files from map-editor/dist at /editor/ path
      server.middlewares.use((req, res, next) => {
        const url = req.url?.split("?")[0] || "";

        // Handle /editor/ route - serve index.html
        if (url === "/editor" || url === "/editor/") {
          const indexPath = path.join(mapEditorDist, "index.html");
          if (fs.existsSync(indexPath)) {
            const html = rewriteEditorHtml(fs.readFileSync(indexPath, "utf-8"));
            res.setHeader("Content-Type", "text/html");
            res.end(html);
            return;
          }
        }

        // Serve static files from /editor/* path
        if (url.startsWith("/editor/")) {
          const filePath = url.replace("/editor/", "");
          const fullPath = path.join(mapEditorDist, filePath);

          if (fs.existsSync(fullPath) && fs.statSync(fullPath).isFile()) {
            const ext = path.extname(fullPath);
            const contentTypes: Record<string, string> = {
              ".html": "text/html",
              ".js": "application/javascript",
              ".wasm": "application/wasm",
              ".css": "text/css",
            };

            res.setHeader(
              "Content-Type",
              contentTypes[ext] || "application/octet-stream",
            );
            // WASM needs these headers for SharedArrayBuffer (if used)
            res.setHeader("Cross-Origin-Opener-Policy", "same-origin");
            res.setHeader("Cross-Origin-Embedder-Policy", "require-corp");
            fs.createReadStream(fullPath).pipe(res);
            return;
          }
        }

        next();
      });
    },
  };
}

export default defineConfig({
  plugins: [react(), tailwindcss(), serveMapEditor()],
  resolve: {
    alias: {
      "@client": path.resolve(__dirname, "../gen/ts"),
    },
  },
  server: {
    proxy: {
      "/api": {
        target: "http://localhost:8080",
        changeOrigin: true,
        rewrite: (path) => path.replace(/^\/api/, ""),
      },
    },
  },
});
