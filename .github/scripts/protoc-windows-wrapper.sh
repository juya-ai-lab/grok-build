#!/usr/bin/env bash
# Windows-compatible wrapper around the real protoc binary.
#
# xai-proto-build (CI-external crate code we must not change) invokes protoc with:
#   --dependency_out=/dev/stdout
#   --descriptor_set_out=/dev/null
# and then expects stdout dependency lines to start with the literal prefix
# "/dev/null:". Those Unix device paths do not exist on Windows, so the real
# protoc fails with: `/dev/stdout: No such file or directory`.
#
# This wrapper rewrites those args to temp/NUL paths, runs the real protoc, then
# rewrites the dependency-file prefix back to "/dev/null:" for the caller.
#
# Env:
#   PROTOC_REAL  absolute path to the real protoc(.exe)  (required)
set -euo pipefail

if [[ -z "${PROTOC_REAL:-}" ]]; then
  echo "protoc-windows-wrapper: PROTOC_REAL is not set" >&2
  exit 1
fi
if [[ ! -f "$PROTOC_REAL" ]]; then
  echo "protoc-windows-wrapper: PROTOC_REAL not found: $PROTOC_REAL" >&2
  exit 1
fi

# Portable temp paths (Git Bash on Windows understands these).
dep_out="${TMPDIR:-/tmp}/protoc-dep-$$.d"
desc_out="${TMPDIR:-/tmp}/protoc-desc-$$.pb"
# Prefer Windows NUL when available for descriptor sink.
if [[ "${OS:-}" == "Windows_NT" ]] || [[ "$(uname -s 2>/dev/null || true)" == MINGW* ]] \
  || [[ "$(uname -s 2>/dev/null || true)" == MSYS* ]] \
  || [[ "$(uname -s 2>/dev/null || true)" == CYGWIN* ]]; then
  desc_out="NUL"
fi

args=()
use_dep_rewrite=0
for arg in "$@"; do
  case "$arg" in
    --dependency_out=/dev/stdout)
      args+=("--dependency_out=${dep_out}")
      use_dep_rewrite=1
      ;;
    --descriptor_set_out=/dev/null)
      args+=("--descriptor_set_out=${desc_out}")
      ;;
    *)
      args+=("$arg")
      ;;
  esac
done

set +e
"$PROTOC_REAL" "${args[@]}"
status=$?
set -e

if [[ "$use_dep_rewrite" -eq 1 && -f "$dep_out" ]]; then
  # dependency_out format: "<descriptor_path>: deps..."
  # Caller requires the descriptor path to be exactly "/dev/null".
  # Use sed that works on both GNU and BSD (no -i needed; stream to stdout).
  sed '1s|^[^:]*:|/dev/null:|' "$dep_out"
  rm -f "$dep_out"
fi
if [[ "$desc_out" != "NUL" && -f "$desc_out" ]]; then
  rm -f "$desc_out"
fi

exit "$status"
