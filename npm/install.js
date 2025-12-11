#!/usr/bin/env node

const { execSync } = require("node:child_process");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const { setTimeout: delay } = require("node:timers/promises");

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
const archiveExt = isWindows ? "zip" : "tar.xz";
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

async function waitForRelease(url) {
  const maxMs = 10 * 60 * 1000; // 10 minutes
  const intervalMs = 10 * 1000; // 10 seconds
  const deadline = Date.now() + maxMs;
  while (true) {
    try {
      execSync(`curl -sfI ${url}`, { stdio: "ignore" });
      return;
    } catch (err) {
      if (Date.now() >= deadline) {
        throw new Error(`Timed out waiting for release asset: ${url}`);
      }
      console.log(`Waiting for release asset to appear... (${url})`);
      await delay(intervalMs);
    }
  }
}

async function main() {
  console.log(`Fetching ${binName} ${version} for ${target}...`);
  await waitForRelease(downloadUrl);

  try {
    if (isWindows) {
      run(
        `powershell -Command "Invoke-WebRequest -Uri '${downloadUrl}' -OutFile '${archivePath}'"`
      );
      run(
        `powershell -Command "Expand-Archive -Path '${archivePath}' -DestinationPath '${nativeDir}' -Force"`
      );
    } else {
      run(`curl -fL ${downloadUrl} -o "${archivePath}"`);
      run(`tar -xJf "${archivePath}" -C "${nativeDir}" --strip-components=1`);
    }

    const ext = isWindows ? ".exe" : "";
    const finalBinPath = path.join(nativeDir, binName + ext);

    function findBinary(start) {
      for (const entry of fs.readdirSync(start, { withFileTypes: true })) {
        const full = path.join(start, entry.name);
        if (entry.isFile() && entry.name === binName + ext) {
          return full;
        }
        if (entry.isDirectory()) {
          const found = findBinary(full);
          if (found) return found;
        }
      }
      return null;
    }

    const located = findBinary(nativeDir);
    if (!located) {
      throw new Error("Binary not found after extraction");
    }

    if (located !== finalBinPath) {
      fs.copyFileSync(located, finalBinPath);
    }

    if (!isWindows) {
      fs.chmodSync(finalBinPath, 0o755);
    }

    console.log(`Installed ${binName} ${version} -> ${finalBinPath}`);
  } catch (err) {
    console.error(`Failed to install binary: ${err.message}`);
    process.exit(1);
  }
}

main().catch((err) => {
  console.error(`Install failed: ${err.message}`);
  process.exit(1);
});
