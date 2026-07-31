#!/usr/bin/env bash
# Install the protoc version pinned by repo bin/protoc (DotSlash manifest).
# Host tool: select asset by runner OS/arch (RUNNER_OS / RUNNER_ARCH), not cargo target.
#
# Portable across Linux / macOS (bash 3.2) / Windows Git Bash:
#   - no mapfile (bash 4+)
#   - strip CR so Windows Python/CRLF cannot poison curl URLs
#
# Side effects:
#   - writes binary under $RUNNER_TEMP/protoc-<version>/
#   - appends bin dir to $GITHUB_PATH (when set)
#   - appends PROTOC=... to $GITHUB_ENV (when set)
#   - prints absolute protoc path on stdout
set -euo pipefail

: "${RUNNER_OS:?RUNNER_OS is required}"
: "${RUNNER_ARCH:?RUNNER_ARCH is required}"
: "${RUNNER_TEMP:=/tmp}"

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
MANIFEST="${ROOT}/bin/protoc"
if [[ ! -f "$MANIFEST" ]]; then
  echo "missing $MANIFEST" >&2
  exit 1
fi

# Resolve version/url/digest/path via Python; write LF-only key=value (no mapfile).
META_FILE="${RUNNER_TEMP}/protoc-resolve-$$.env"
python3 - "$MANIFEST" "$META_FILE" <<'PY'
import json, os, re, sys
from pathlib import Path

manifest_path = Path(sys.argv[1])
out_path = Path(sys.argv[2])

raw = manifest_path.read_text(encoding="utf-8")
start = raw.find("{")
if start < 0:
    sys.exit("bin/protoc: no JSON object found")
manifest = json.loads(raw[start:])
platforms = manifest.get("platforms") or {}
if not platforms:
    sys.exit("bin/protoc: empty platforms")

key_map = {
    ("Linux", "X64"): "linux-x86_64",
    ("Linux", "ARM64"): "linux-aarch64",
    ("macOS", "ARM64"): "macos-aarch64",
    ("macOS", "X64"): "macos-x86_64",
    ("Windows", "X64"): "windows-x86_64",
    ("Windows", "ARM64"): "windows-x86_64",  # win64 under x64 emu
}
os_name = os.environ["RUNNER_OS"]
arch = os.environ["RUNNER_ARCH"]
ds_key = key_map.get((os_name, arch))
if not ds_key:
    sys.exit(f"unsupported runner for protoc: {os_name}-{arch}")


def first_url(entry: dict) -> str:
    for p in entry.get("providers") or []:
        if p.get("url"):
            return p["url"]
    raise SystemExit("no provider url in platform entry")


version = None
for entry in platforms.values():
    try:
        sample_url = first_url(entry)
    except SystemExit:
        continue
    m = re.search(r"/download/v([^/]+)/", sample_url)
    if m:
        version = m.group(1)
        break
if not version:
    sys.exit("bin/protoc: could not parse protoc version from provider URLs")

if ds_key in platforms:
    entry = platforms[ds_key]
    url = first_url(entry)
    digest = entry.get("digest") or ""
    relpath = entry.get("path") or "bin/protoc"
    print(f"manifest platform={ds_key}", file=sys.stderr)
else:
    asset_by_key = {
        "windows-x86_64": f"protoc-{version}-win64.zip",
        "macos-x86_64": f"protoc-{version}-osx-x86_64.zip",
        "macos-aarch64": f"protoc-{version}-osx-aarch_64.zip",
        "linux-x86_64": f"protoc-{version}-linux-x86_64.zip",
        "linux-aarch64": f"protoc-{version}-linux-aarch_64.zip",
    }
    asset = asset_by_key.get(ds_key)
    if not asset:
        sys.exit(f"no fallback asset mapping for {ds_key}")
    url = (
        f"https://github.com/protocolbuffers/protobuf/releases/"
        f"download/v{version}/{asset}"
    )
    digest = ""
    relpath = "bin/protoc.exe" if os_name == "Windows" else "bin/protoc"
    print(
        f"manifest has no {ds_key}; fallback same version v{version}",
        file=sys.stderr,
    )

# Always LF so bash on Windows does not pick up trailing CR.
with out_path.open("w", encoding="utf-8", newline="\n") as fh:
    fh.write(f"version={version}\n")
    fh.write(f"url={url}\n")
    fh.write(f"digest={digest}\n")
    fh.write(f"relpath={relpath}\n")
PY

version=""
url=""
expect_sha=""
zip_inner=""
while IFS= read -r line || [[ -n "$line" ]]; do
  # Strip CR for any accidental CRLF consumers.
  line="${line%$'\r'}"
  case "$line" in
    version=*) version="${line#version=}" ;;
    url=*) url="${line#url=}" ;;
    digest=*) expect_sha="${line#digest=}" ;;
    relpath=*) zip_inner="${line#relpath=}" ;;
  esac
done < "$META_FILE"
rm -f "$META_FILE"

if [[ -z "$version" || -z "$url" || -z "$zip_inner" ]]; then
  echo "failed to resolve protoc metadata from bin/protoc" >&2
  exit 1
fi

dest="${RUNNER_TEMP}/protoc-${version}"
# Normalize path separators for mixed Windows/bash environments.
dest="${dest//\\//}"
mkdir -p "$dest"
asset="$(basename "$url")"
echo "Installing protoc v${version} from bin/protoc -> ${url}" >&2

# Download via Python first (portable; avoids Git-Bash curl + CRLF URL issues on
# Windows). Fall back to curl if urllib is unavailable.
python3 - "$url" "${dest}/${asset}" <<'PY'
import sys
import urllib.request

url, dest = sys.argv[1], sys.argv[2]
# Guard against accidental CR from env/shell on Windows.
url = url.replace("\r", "").strip()
req = urllib.request.Request(url, headers={"User-Agent": "grok-build-ci"})
with urllib.request.urlopen(req, timeout=120) as resp, open(dest, "wb") as out:
    while True:
        chunk = resp.read(1024 * 1024)
        if not chunk:
            break
        out.write(chunk)
print(f"downloaded {url} -> {dest}", file=sys.stderr)
PY

# DotSlash digests the fetched artifact (the zip), not the extracted binary.
if [[ -n "$expect_sha" ]]; then
  if command -v sha256sum >/dev/null 2>&1; then
    got=$(sha256sum "${dest}/${asset}" | awk '{print $1}')
  else
    got=$(shasum -a 256 "${dest}/${asset}" | awk '{print $1}')
  fi
  got="${got%$'\r'}"
  if [[ "$got" != "$expect_sha" ]]; then
    echo "protoc zip digest mismatch: got $got want $expect_sha" >&2
    exit 1
  fi
  echo "Verified zip SHA256 $got (from bin/protoc)" >&2
else
  echo "No digest in bin/protoc for this host; skipped verify" >&2
fi

if command -v unzip >/dev/null 2>&1; then
  unzip -qo "${dest}/${asset}" -d "$dest"
else
  python3 -c "import sys, zipfile; zipfile.ZipFile(sys.argv[1]).extractall(sys.argv[2])" \
    "${dest}/${asset}" "$dest"
fi

protoc_bin="${dest}/${zip_inner}"
protoc_bin="${protoc_bin//\\//}"
if [[ ! -f "$protoc_bin" && -f "${dest}/bin/protoc.exe" ]]; then
  protoc_bin="${dest}/bin/protoc.exe"
fi
if [[ ! -f "$protoc_bin" && -f "${dest}/bin/protoc" ]]; then
  protoc_bin="${dest}/bin/protoc"
fi
if [[ ! -f "$protoc_bin" ]]; then
  echo "protoc binary not found after extract (looked for ${zip_inner})" >&2
  ls -la "$dest" "$dest/bin" 2>/dev/null || true
  exit 1
fi
chmod +x "$protoc_bin" 2>/dev/null || true

protoc_bin_env="${protoc_bin//\\//}"
bin_dir="$(dirname "$protoc_bin_env")"

# On Windows, xai-proto-build passes --dependency_out=/dev/stdout and
# --descriptor_set_out=/dev/null (Unix devices). Point PROTOC at a CI-only
# wrapper that rewrites those paths; do not change crate code.
protoc_for_build="$protoc_bin_env"
if [[ "${RUNNER_OS}" == "Windows" ]]; then
  wrap_src_dir="$(cd "$(dirname "$0")" && pwd)"
  wrap_src_dir="${wrap_src_dir//\\//}"
  cp -f "${wrap_src_dir}/protoc-windows-wrapper.sh" "${bin_dir}/protoc-windows-wrapper.sh"
  cp -f "${wrap_src_dir}/protoc-windows-wrapper.cmd" "${bin_dir}/protoc-windows-wrapper.cmd"
  chmod +x "${bin_dir}/protoc-windows-wrapper.sh" 2>/dev/null || true
  # Real binary for the wrapper; build scripts call the .cmd entrypoint.
  if [[ -n "${GITHUB_ENV:-}" ]]; then
    echo "PROTOC_REAL=${protoc_bin_env}" >> "$GITHUB_ENV"
  fi
  export PROTOC_REAL="$protoc_bin_env"
  protoc_for_build="${bin_dir}/protoc-windows-wrapper.cmd"
  echo "Windows: PROTOC -> wrapper (PROTOC_REAL=$protoc_bin_env)" >&2
fi

if [[ -n "${GITHUB_PATH:-}" ]]; then
  echo "$bin_dir" >> "$GITHUB_PATH"
fi
if [[ -n "${GITHUB_ENV:-}" ]]; then
  echo "PROTOC=${protoc_for_build}" >> "$GITHUB_ENV"
fi
if [[ -n "${GITHUB_OUTPUT:-}" ]]; then
  echo "path=${protoc_for_build}" >> "$GITHUB_OUTPUT"
  echo "version=${version}" >> "$GITHUB_OUTPUT"
fi

# Sanity: real binary --version (not the full codegen path rewrite).
"$protoc_bin" --version >&2
echo "$protoc_for_build"
