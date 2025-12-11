#!/usr/bin/env node

const { spawnSync } = require("node:child_process");
const path = require("node:path");
const os = require("node:os");

const bin = path.join(__dirname, "native", `rubber-duck-mcp${os.platform() === "win32" ? ".exe" : ""}`);

const result = spawnSync(bin, process.argv.slice(2), {
  stdio: "inherit",
});

if (result.error) {
  console.error(result.error);
  process.exit(1);
}

process.exit(result.status === null ? 1 : result.status);
