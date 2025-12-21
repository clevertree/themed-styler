import express from 'express';
import path from 'path';
import { fileURLToPath } from 'url';
import fs from 'fs';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const app = express();
const port = 8084;

// Root of the repo (themed-styler)
const repoRoot = path.join(__dirname, '../..');
const hookTranspilerRoot = path.join(repoRoot, '../hook-transpiler');

// 1. Unified WASM serving location
const wasmDir = path.join(repoRoot, 'wasm');
const hookWasmDir = path.join(hookTranspilerRoot, 'wasm');

app.use('/wasm', (req, res, next) => {
  const filename = req.path.split('/').pop();
  if (!filename.endsWith('.wasm')) return next();

  // Try themed-styler wasm first
  const stylerWasmPath = path.join(wasmDir, filename);
  if (fs.existsSync(stylerWasmPath)) {
    res.setHeader('Content-Type', 'application/wasm');
    return res.sendFile(stylerWasmPath);
  }

  // Try hook-transpiler wasm
  const hookWasmPath = path.join(hookWasmDir, filename);
  if (fs.existsSync(hookWasmPath)) {
    res.setHeader('Content-Type', 'application/wasm');
    return res.sendFile(hookWasmPath);
  }

  res.status(404).send('WASM not found');
});

// Serve JS glue from wasm dirs
app.use('/wasm', express.static(wasmDir));
app.use('/wasm', express.static(hookWasmDir));

// Serve static files
app.use(express.static(path.join(__dirname, 'public')));

// Serve packages dist for debugging if needed
app.use('/themed-styler/dist', express.static(path.join(repoRoot, 'dist')));
app.use('/hook-transpiler/dist', express.static(path.join(hookTranspilerRoot, 'dist')));

// Serve react from themed-styler node_modules
app.use('/react', express.static(path.join(repoRoot, 'node_modules/react/umd')));
app.use('/react-dom', express.static(path.join(repoRoot, 'node_modules/react-dom/umd')));

// Serve a test hook
app.get('/hooks/test-hook.jsx', (req, res) => {
  res.setHeader('Content-Type', 'text/javascript');
  res.send(`
    import React from 'react';
    export default function TestHook() {
      return (
        <div className="p-4 bg-blue-500 text-white rounded shadow-lg">
          <h1 className="text-2xl font-bold">Hello from Test Hook!</h1>
          <p className="mt-2">This hook was transpiled and rendered by HookRenderer.</p>
          <div className="mt-4 p-2 bg-white text-blue-800 rounded">
            Tailwind class 'bg-blue-500' should be active if themed-styler is working.
          </div>
        </div>
      );
    }
  `);
});

// Explicitly handle 404 for missing files to avoid serving index.html for everything
app.get('*', (req, res, next) => {
  // If it looks like a file (has extension), 404 it if not found yet
  if (req.path.includes('.')) {
    return res.status(404).send('404: Not Found');
  }
  next();
});

// Fallback for SPA (only for non-file paths)
app.get('*', (req, res) => {
  res.sendFile(path.join(__dirname, 'public', 'index.html'));
});

app.listen(port, () => {
  console.log(`Test server running at http://localhost:${port}`);
});
