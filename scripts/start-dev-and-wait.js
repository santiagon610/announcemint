#!/usr/bin/env node
/**
 * Starts the Vite dev server in the background and exits only when the server
 * is ready to serve the app. Used as part of beforeDevCommand so that when
 * Tauri opens the window, the first load gets a full page instead of a blank.
 *
 * Usage: node scripts/start-dev-and-wait.js
 * (Run from project root; expects npm run dev to start Vite on port 5173.)
 */

import { spawn } from "child_process";
import http from "http";

const DEV_URL = "http://localhost:5173";
const POLL_INTERVAL_MS = 300;
const MAX_WAIT_MS = 120000; // 2 minutes
const EXTRA_DELAY_MS = 2000; // extra time for first Vite compile after server responds

function fetch(url) {
  return new Promise((resolve, reject) => {
    const req = http.get(url, { timeout: 5000 }, (res) => {
      let data = "";
      res.on("data", (chunk) => { data += chunk; });
      res.on("end", () => resolve({ status: res.statusCode, data }));
    });
    req.on("error", reject);
    req.on("timeout", () => {
      req.destroy();
      reject(new Error("timeout"));
    });
  });
}

async function waitForServer() {
  const start = Date.now();
  while (Date.now() - start < MAX_WAIT_MS) {
    try {
      const { status, data } = await fetch(DEV_URL);
      if (status === 200 && data.includes('id="app"')) {
        return true;
      }
    } catch {
      // ignore and retry
    }
    await new Promise((r) => setTimeout(r, POLL_INTERVAL_MS));
  }
  return false;
}

async function main() {
  const isWindows = process.platform === "win32";
  const npm = isWindows ? "npm.cmd" : "npm";
  const child = spawn(npm, ["run", "dev"], {
    stdio: "ignore",
    detached: true,
    cwd: process.cwd(),
    env: process.env,
    shell: isWindows,
  });
  child.unref();

  const ready = await waitForServer();
  if (!ready) {
    console.error("Dev server did not become ready in time.");
    process.exit(1);
  }

  await new Promise((r) => setTimeout(r, EXTRA_DELAY_MS));
  process.exit(0);
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
