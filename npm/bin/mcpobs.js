#!/usr/bin/env node
// Thin bin wrapper. The real binary is dropped here by postinstall.js.
"use strict";

const { spawnSync } = require("child_process");
const path = require("path");
const fs = require("fs");

const binName = process.platform === "win32" ? "mcpobs.exe" : "mcpobs";
const binPath = path.join(__dirname, binName);

if (!fs.existsSync(binPath)) {
  console.error(
    `mcpobs: binary not found at ${binPath}. ` +
      `If you used --ignore-scripts, re-install without it: \`npm install -g mcpobs\`. ` +
      `Or grab a prebuilt directly: ` +
      `https://github.com/vnmoorthy/mcpobservatory/releases/latest`
  );
  process.exit(127);
}

const r = spawnSync(binPath, process.argv.slice(2), {
  stdio: "inherit",
  windowsHide: false,
});
if (r.error) {
  console.error(`mcpobs: failed to spawn ${binPath}: ${r.error.message}`);
  process.exit(1);
}
process.exit(r.status === null ? 1 : r.status);
