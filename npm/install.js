#!/usr/bin/env node

const { execSync } = require("node:child_process");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");

const pkg = require("./package.json");
const version = pkg.version;
const repo = "2lab-ai/rubber-duck-mcp";
const binName = "rubber-duck-mcp";

const targets = {
  "darwin-arm64": "aarch64-apple-darwin",
  "darwin-x64": "x86_64-apple-darwin",
  "linux-arm64": "aarch64-unknown-linux-gnu",
  "linux-x64": "x86_64-unknown-linux-gnu",
  "win32-x64": "x86_64-pc-windows-msvc",
};

const platformKey = `${os.platform()}-${os.arch()}`;
const target = targets[platformKey];

if (!target) {
  console.error(`Unsupported platform: ${platformKey}`);
  process.exit(1);
}

const isWindows = os.platform() === "win32";
const archiveExt = isWindows ? "zip" : "tar.gz";
const archiveName = `${binName}-${target}.${archiveExt}`;
const downloadUrl = `https://github.com/${repo}/releases/download/v${version}/${archiveName}`;

const binRoot = path.join(__dirname, "bin");
const nativeDir = path.join(binRoot, "native");
fs.mkdirSync(nativeDir, { recursive: true });

const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), `${binName}-`));
const archivePath = path.join(tmpDir, archiveName);

function run(cmd) {
  execSync(cmd, { stdio: "inherit" });
}

console.log(`Fetching ${binName} ${version} for ${target}...`);

try {
  if (isWindows) {
    run(`powershell -Command "Invoke-WebRequest -Uri  -OutFile "`);
    run(`powershell -Command "Expand-Archive -Path  -DestinationPath  -Force"`);
  } else {
    run(`curl -fL ${downloadUrl} -o "${archivePath}"`);
    run(`tar -xzf "${archivePath}" -C "${nativeDir}"`);
  }

  const binPath = path.join(nativeDir, binName + (isWindows ? ".exe" : ""));
  if (!fs.existsSync(binPath)) {
    throw new Error("Binary not found after extraction");
  }
  if (!isWindows) {
    fs.chmodSync(binPath, 0o755);
  }

  console.log(`Installed ${binName} ${version} -> ${binPath}`);
} catch (err) {
  console.error(`Failed to install binary: ${err.message}`);
  process.exit(1);
}
