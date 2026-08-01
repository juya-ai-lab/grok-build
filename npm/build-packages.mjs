#!/usr/bin/env node
// Build the npm packages for @juya-ai-lab/grok-build from GitHub Release assets.
//
// Usage:
//   node npm/build-packages.mjs <version> <assetsDir> <outDir> <manifestPath>
//
// assetsDir must contain the six release assets produced by the release
// workflow (grok-<version>-<platform>[.exe]) plus their .sha256 files.
// The script writes six platform packages (one prebuilt binary each) and the
// main wrapper package, then writes a manifest listing every package dir in
// publish order: platform packages first, main package last.

import {
  chmodSync,
  copyFileSync,
  existsSync,
  mkdirSync,
  writeFileSync,
} from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

const SCOPE = "@juya-ai-lab";
const PACKAGE_NAME = "grok-build";
const LICENSE = "Apache-2.0";
const HOMEPAGE = "https://github.com/juya-ai-lab/grok-build";

const PLATFORMS = [
  { release: "linux-x86_64", os: "linux", arch: "x64", exe: "" },
  { release: "linux-aarch64", os: "linux", arch: "arm64", exe: "" },
  { release: "macos-x86_64", os: "darwin", arch: "x64", exe: "" },
  { release: "macos-aarch64", os: "darwin", arch: "arm64", exe: "" },
  { release: "windows-x86_64", os: "win32", arch: "x64", exe: ".exe" },
  { release: "windows-aarch64", os: "win32", arch: "arm64", exe: ".exe" },
];

const [, , version, assetsDir, outDir, manifestPath] = process.argv;

if (!version || !assetsDir || !outDir || !manifestPath) {
  console.error(
    "Usage: node npm/build-packages.mjs <version> <assetsDir> <outDir> <manifestPath>"
  );
  process.exit(1);
}

if (!/^[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z-]+(\.[0-9A-Za-z-]+)*)?$/.test(version)) {
  console.error(`Invalid npm version: ${version}`);
  process.exit(1);
}

const platformName = (p) => `${SCOPE}/${PACKAGE_NAME}-${p.os}-${p.arch}`;
const publishOrder = [];

for (const p of PLATFORMS) {
  const asset = `grok-${version}-${p.release}${p.exe}`;
  const assetPath = join(assetsDir, asset);
  if (!existsSync(assetPath)) {
    console.error(`Missing release asset: ${assetPath}`);
    process.exit(1);
  }

  const pkgDir = join(outDir, `${PACKAGE_NAME}-${p.os}-${p.arch}`);
  const binDir = join(pkgDir, "bin");
  mkdirSync(binDir, { recursive: true });

  const binName = p.exe ? "grok-build.exe" : "grok-build";
  const binPath = join(binDir, binName);
  copyFileSync(assetPath, binPath);
  chmodSync(binPath, 0o755);

  writeFileSync(
    join(pkgDir, "package.json"),
    `${JSON.stringify(
      {
        name: platformName(p),
        version,
        description: `grok-build binary for ${p.os}-${p.arch}`,
        license: LICENSE,
        os: [p.os],
        cpu: [p.arch],
        files: ["bin"],
      },
      null,
      2
    )}\n`
  );
  publishOrder.push(pkgDir);
}

const mainDir = join(outDir, PACKAGE_NAME);
const mainBinDir = join(mainDir, "bin");
mkdirSync(mainBinDir, { recursive: true });

const wrapperSrc = fileURLToPath(new URL("./wrapper.js", import.meta.url));
const readmeSrc = fileURLToPath(new URL("./README.md", import.meta.url));
copyFileSync(wrapperSrc, join(mainBinDir, "grok-build.js"));
copyFileSync(readmeSrc, join(mainDir, "README.md"));

const optionalDependencies = Object.fromEntries(
  PLATFORMS.map((p) => [platformName(p), version])
);

writeFileSync(
  join(mainDir, "package.json"),
  `${JSON.stringify(
    {
      name: `${SCOPE}/${PACKAGE_NAME}`,
      version,
      description: "grok-build terminal agent CLI (prebuilt binaries via optionalDependencies)",
      license: LICENSE,
      homepage: HOMEPAGE,
      repository: {
        type: "git",
        url: `git+${HOMEPAGE}.git`,
      },
      bin: {
        "grok-build": "bin/grok-build.js",
      },
      files: ["bin", "README.md"],
      optionalDependencies,
    },
    null,
    2
  )}\n`
);
publishOrder.push(mainDir);

writeFileSync(manifestPath, `${publishOrder.join("\n")}\n`);
console.log(`Built ${publishOrder.length} npm packages for version ${version}`);
