#!/usr/bin/env bash
# Upstream trace tool for this privacy-trimmed fork.
#
# Modes:
#   refresh (default) — regenerate the git-derived blocks in UPSTREAM_TRACE.md
#       (status / upstream sync log / fork change log), delimited by
#       <!-- TRACE:<name>:BEGIN --> ... <!-- TRACE:<name>:END --> markers.
#       Judgement content (rationale, trim notes) lives in the manual sections
#       and is never touched.
#   --check           — read-only: verify every commit in the fork change log
#       has a matching entry in the manual「决策记录」section. A commit is
#       covered when the section contains a backticked token that is a prefix
#       of its full SHA (short-SHA lengths vary between clones), or its full
#       subject. CI-mechanical commits (chore: ... trace) are exempt.
#       Exit 1 when coverage is incomplete.
#   --fetch           — git fetch upstream --prune before doing anything.
#
# Usage:
#   scripts/upstream-trace.sh [--fetch] [--check]
set -euo pipefail

fetch=0
check=0
for arg in "$@"; do
  case "$arg" in
    --fetch) fetch=1 ;;
    --check) check=1 ;;
    *) echo "unknown argument: $arg" >&2; exit 2 ;;
  esac
done

if (( fetch )); then
  git fetch upstream --prune
fi

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

TRACE="UPSTREAM_TRACE.md"
VERSION_CRATE="crates/codegen/xai-grok-version/Cargo.toml"
UP="upstream/main"

[ -f "$TRACE" ] || { echo "错误: 缺少 $TRACE" >&2; exit 2; }
if ! git rev-parse --verify -q "$UP" >/dev/null 2>&1; then
  echo "错误: 缺少 $UP 上游 ref; 先运行: scripts/upstream-trace.sh --fetch" >&2
  exit 2
fi

# --- facts (shared by both modes) ---
local_head_full="$(git rev-parse HEAD)"
local_date="$(git log -1 --format=%cs)"
src_rev="$(cat SOURCE_REV)"
version="$(sed -n 's/^version = "\(.*\)"/\1/p' "$VERSION_CRATE")"
up_head_full="$(git rev-parse "$UP")"
up_date="$(git log -1 --format=%cs "$UP")"
up_subject="$(git log -1 --format=%s "$UP")"
up_src_rev="$(git show "$UP":SOURCE_REV 2>/dev/null || echo n/a)"
up_behind="$(git rev-list --count HEAD.."$UP")"
up_ahead="$(git rev-list --count "$UP"..HEAD)"
diff_files="$(git diff --name-only HEAD "$UP" | wc -l | tr -d ' ')"
if [[ "$up_src_rev" == "$src_rev" ]]; then
  verdict="无新基线：上游 SOURCE_REV 与本地一致；树差异 ${diff_files} 个文件，应为本地裁剪，需人工核对"
else
  verdict="上游有新基线：SOURCE_REV 变为 ${up_src_rev}，需按 AGENTS.md 评估合入/裁剪"
fi

# --- check mode: decision-record coverage (read-only) ---
if (( check )); then
  coverage="$(awk 'found{print} /^## 决策记录/{found=1}' "$TRACE")"
  [ -n "$coverage" ] || { echo "错误: 在 $TRACE 中找不到「决策记录」小节" >&2; exit 2; }
  tokens="$(printf '%s\n' "$coverage" | grep -o '`[^`]*`' | tr -d '`' || true)"
  total=0
  missing=0
  while IFS='|' read -r h d s; do
    total=$((total + 1))
    case "$s" in
      "chore: refresh upstream trace"* | "chore: update upstream trace"*) continue ;;
    esac
    covered=0
    while read -r tok; do
      [ -z "$tok" ] && continue
      case "$h" in "$tok"*) covered=1; break ;; esac
    done <<<"$tokens"
    if [ "$covered" -eq 0 ] && ! grep -qF "$s" <<<"$coverage"; then
      echo "缺少决策记录: ${h:0:8}  $s" >&2
      missing=1
    fi
  done < <(git log --format='%H|%cs|%s' "$UP..HEAD")
  if (( missing )); then
    echo "请在 $TRACE「决策记录」为上述提交补充一行（提交 / 原因 / 对应宗旨）" >&2
    exit 1
  fi
  echo "决策记录覆盖完整: 本 fork 共 ${total} 个提交均有对应记录（CI 机械刷新类提交豁免）"
  exit 0
fi

# --- refresh mode: build blocks ---
status_block=$(cat <<EOF
- 本地 HEAD: ${local_head_full:0:8} (${local_date})
- 版本: ${version}
- 上游基线 SOURCE_REV: ${src_rev}
- 上游 ${UP}: ${up_head_full:0:8} (${up_date}, ${up_subject})
- 上游 SOURCE_REV: ${up_src_rev}
- 落后上游 ${up_behind} 提交 / 本 fork 领先 ${up_ahead} 提交; 树差异 ${diff_files} 个文件
- 判定: ${verdict}
EOF
)

sync_block=$(cat <<EOF
| 上游提交 | 日期 | 内容 | SOURCE_REV |
|---|---|---|---|
EOF
)
while IFS='|' read -r h d s; do
  rev="$(git show "$h":SOURCE_REV 2>/dev/null || echo n/a)"
  sync_block+="
| ${h:0:8} | ${d} | ${s} | ${rev} |"
done < <(git log --format='%H|%cs|%s' "$UP")

fork_block=$(cat <<EOF
| 提交 | 日期 | 内容 |
|---|---|---|
EOF
)
while IFS='|' read -r h d s; do
  fork_block+="
| ${h:0:8} | ${d} | ${s} |"
done < <(git log --format='%H|%cs|%s' "$UP..HEAD")

# --- replace the marked blocks in-place (idempotent) ---
replace_block() {
  local marker="$1" content="$2"
  grep -q "<!-- TRACE:${marker}:BEGIN -->" "$TRACE" \
    || { echo "错误: $TRACE 缺少 <!-- TRACE:${marker}:BEGIN --> 标记" >&2; exit 2; }
  awk -v m="$marker" -v c="$content" '
    $0 == "<!-- TRACE:" m ":BEGIN -->" { print; print c; skip=1; next }
    $0 == "<!-- TRACE:" m ":END -->" { skip=0; print; next }
    !skip { print }
  ' "$TRACE" > "${TRACE}.tmp"
  mv "${TRACE}.tmp" "$TRACE"
}

replace_block status "$status_block"
replace_block sync "$sync_block"
replace_block fork "$fork_block"

echo "已更新 ${TRACE}: status / sync / fork 区块由 git 事实重新生成; 判断性内容请在「决策记录」手工补充"
