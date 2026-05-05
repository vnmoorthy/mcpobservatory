#!/usr/bin/env node
"use strict";
const fs = require("fs");
const path = require("path");
const https = require("https");
const crypto = require("crypto");
const os = require("os");
const { execFileSync } = require("child_process");
const { pipeline } = require("stream");
const { promisify } = require("util");
const streamPipeline = promisify(pipeline);

const REPO = process.env.MCPOBS_REPO || "vnmoorthy/mcpobservatory";
const VERSION = require("./package.json").version;

if (process.env.MCPOBS_SKIP_DOWNLOAD === "1") {
  console.log("mcpobs: MCPOBS_SKIP_DOWNLOAD=1, skipping binary download");
  process.exit(0);
}

function detectTarget() {
  const p = process.platform, a = process.arch;
  if (p === "darwin" && a === "arm64") return ["aarch64-apple-darwin", "tar.gz", "mcpobs"];
  if (p === "darwin" && a === "x64")   return ["x86_64-apple-darwin", "tar.gz", "mcpobs"];
  if (p === "linux"  && a === "x64")   return ["x86_64-unknown-linux-gnu", "tar.gz", "mcpobs"];
  if (p === "linux"  && a === "arm64") return ["aarch64-unknown-linux-gnu", "tar.gz", "mcpobs"];
  if (p === "win32"  && a === "x64")   return ["x86_64-pc-windows-msvc", "zip", "mcpobs.exe"];
  throw new Error(`mcpobs: unsupported platform ${p}/${a}. file: https://github.com/${REPO}/issues`);
}

function get(url, hops = 5) {
  return new Promise((resolve, reject) => {
    https.get(url, { headers: { "user-agent": `mcpobs-npm/${VERSION}`, accept: "application/octet-stream" } }, (res) => {
      if (res.statusCode >= 300 && res.statusCode < 400 && res.headers.location) {
        if (hops <= 0) return reject(new Error("too many redirects"));
        res.resume();
        return resolve(get(res.headers.location, hops - 1));
      }
      if (res.statusCode !== 200) return reject(new Error(`HTTP ${res.statusCode} from ${url}`));
      resolve(res);
    }).on("error", reject);
  });
}

async function download(url, dest) {
  const res = await get(url);
  await streamPipeline(res, fs.createWriteStream(dest));
}

const sha256 = (p) => crypto.createHash("sha256").update(fs.readFileSync(p)).digest("hex");

function extract(archive, dest, ext) {
  if (ext === "tar.gz") return execFileSync("tar", ["-xzf", archive, "-C", dest], { stdio: "inherit" });
  if (process.platform === "win32") {
    return execFileSync("powershell.exe", ["-NoProfile", "-Command",
      `Expand-Archive -Path "${archive}" -DestinationPath "${dest}" -Force`], { stdio: "inherit" });
  }
  return execFileSync("unzip", ["-o", archive, "-d", dest], { stdio: "inherit" });
}

(async () => {
  const [target, ext, binName] = detectTarget();
  const base = `mcpobs-${VERSION}-${target}`;
  const archive = `${base}.${ext}`;
  const baseUrl = `https://github.com/${REPO}/releases/download/v${VERSION}`;
  const tmp = fs.mkdtempSync(path.join(os.tmpdir(), "mcpobs-"));
  const archivePath = path.join(tmp, archive);
  const shaPath = `${archivePath}.sha256`;

  console.log(`mcpobs: downloading ${baseUrl}/${archive}`);
  await download(`${baseUrl}/${archive}`, archivePath);

  let verified = false;
  try {
    await download(`${baseUrl}/${archive}.sha256`, shaPath);
    const expected = fs.readFileSync(shaPath, "utf8").trim().split(/\s+/)[0];
    const actual = sha256(archivePath);
    if (expected.toLowerCase() !== actual.toLowerCase()) {
      throw new Error(`mcpobs: sha256 mismatch (expected ${expected}, got ${actual})`);
    }
    verified = true;
    console.log("mcpobs: sha256 verified");
  } catch (e) {
    if (e.message.startsWith("mcpobs: sha256 mismatch")) throw e;
    console.warn(`mcpobs: warning: ${e.message}, continuing without sha verification`);
  }

  console.log("mcpobs: extracting");
  extract(archivePath, tmp, ext);

  const src = path.join(tmp, base, binName);
  if (!fs.existsSync(src)) throw new Error(`mcpobs: missing ${src} in archive`);

  const binDir = path.join(__dirname, "bin");
  fs.mkdirSync(binDir, { recursive: true });
  const dst = path.join(binDir, binName);
  fs.copyFileSync(src, dst);
  if (process.platform !== "win32") fs.chmodSync(dst, 0o755);

  try { fs.rmSync(tmp, { recursive: true, force: true }); } catch (_) {}
  console.log(`mcpobs: installed v${VERSION} (${target})${verified ? "" : " unverified"}`);
})().catch((e) => { console.error(e.message || e); process.exit(1); });
