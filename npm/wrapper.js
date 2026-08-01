#!/usr/bin/env node
"use strict";

const { spawn } = require("node:child_process");
const path = require("node:path");

const platformPackage = `@juya-ai-lab/grok-build-${process.platform}-${process.arch}`;
const binaryName = process.platform === "win32" ? "grok-build.exe" : "grok-build";

let binaryPath;
try {
  binaryPath = require.resolve(path.posix.join(platformPackage, "bin", binaryName));
} catch {
  console.error(
    `[grok-build] no prebuilt binary for ${process.platform}-${process.arch}.\n` +
      "Supported platforms: linux-x64, linux-arm64, darwin-x64, darwin-arm64, win32-x64, win32-arm64.\n" +
      'Reinstall with "npm i -g @juya-ai-lab/grok-build" so the matching platform package is installed.'
  );
  process.exit(1);
}

const child = spawn(binaryPath, process.argv.slice(2), { stdio: "inherit" });

child.on("error", (error) => {
  console.error(`[grok-build] failed to start binary: ${error.message}`);
  process.exit(1);
});

child.on("exit", (code, signal) => {
  if (signal) {
    process.kill(process.pid, signal);
    return;
  }
  process.exit(code === null ? 0 : code);
});
