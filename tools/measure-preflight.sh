#!/usr/bin/env bash
# measure-preflight.sh — time this repository's green-main gate against the
# platform's 30-minute preflight budget, phase by phase.
#
# Why this exists. On 2026-09-02 ticket 1107 was claimed three times and
# never started. Each claim ran the gate for 33.5, 32.1 and 31.5 minutes and
# was killed by the dispatcher's 30-minute bound (factory cli.ts:911,
# scriptTimeoutMs). The kill signals the whole process group, so `before`
# never reported which gate died, and the evidence log that would have said
# was deleted by factory's rotating 20-file cache before anyone read it.
#
# So: measure it ourselves, and keep the result somewhere factory cannot
# evict. BioMCP owns whether its gates fit the platform's budget. The
# platform owns honest timeout reporting. This script answers only the first.
#
# Usage:
#   tools/measure-preflight.sh                 # current cache conditions
#   tools/measure-preflight.sh --cold          # private sccache, cold compile
#   tools/measure-preflight.sh --force         # run even while the channel is busy
#
# Output: $HOME/.local/share/biomcp-preflight/<stamp>/ holding result.json,
# lint.log and test.log. That path is deliberately NOT factory's
# before-evidence directory, which keeps only the newest 20 files globally.

set -uo pipefail

BUDGET_SECONDS=$((30 * 60))
REPO=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
COLD=0
FORCE=0
for arg in "$@"; do
	case "$arg" in
	--cold) COLD=1 ;;
	--force) FORCE=1 ;;
	-h | --help)
		sed -n '2,28p' "$0"
		exit 0
		;;
	*)
		echo "measure-preflight: unknown argument $arg" >&2
		exit 2
		;;
	esac
done

# Refuse to compete with live factory work. A cold build saturates the CPU,
# and the thing being measured is sensitive to exactly that. Measuring under
# load is a legitimate second experiment; it needs a deliberate --force so it
# is never the accidental default, and the result records which it was.
if [ "$FORCE" -eq 0 ]; then
	if active=$(factory status 2>/dev/null | sed -n '/^active tickets:/,/^blocked tickets:/p' | grep -c 'running;'); then
		if [ "${active:-0}" -gt 0 ]; then
			echo "measure-preflight: $active factory run(s) are live; measuring now would" >&2
			echo "  contend for the same CPU and distort the result. Wait for a quiet" >&2
			echo "  channel, or pass --force to measure under representative load." >&2
			exit 3
		fi
	fi
fi

STAMP=$(date -u +%Y%m%dT%H%M%SZ)
OUT="$HOME/.local/share/biomcp-preflight/$STAMP"
mkdir -p "$OUT" || exit 1

HEAD_SHA=$(git -C "$REPO" rev-parse HEAD)
TREE=$(mktemp -d "${TMPDIR:-/tmp}/biomcp-preflight-XXXXXX") || exit 1
SCRATCH=""

cleanup() {
	git -C "$REPO" worktree remove --force "$TREE" >/dev/null 2>&1
	git -C "$REPO" worktree prune >/dev/null 2>&1
	rm -rf "$TREE"
	[ -n "$SCRATCH" ] && rm -rf "$SCRATCH"
}
trap cleanup EXIT

# A fresh worktree is what every ticket gets, so the compile target directory
# is cold by construction. --cold additionally gives sccache a private empty
# directory, which is the true floor: no compilation result is reused from any
# earlier run. Without it the shared sccache is warm, which is the condition
# production has actually been in.
if [ "$COLD" -eq 1 ]; then
	SCRATCH=$(mktemp -d "${TMPDIR:-/tmp}/biomcp-preflight-cache-XXXXXX") || exit 1
	export SCCACHE_DIR="$SCRATCH/sccache"
	mkdir -p "$SCCACHE_DIR"
	sccache --stop-server >/dev/null 2>&1 || true
fi

phase() { # phase <name> <logfile> -- <cmd...>
	local name=$1 log=$2
	shift 3
	local start end status
	start=$(date +%s.%N)
	"$@" >"$log" 2>&1
	status=$?
	end=$(date +%s.%N)
	PHASE_SECONDS=$(awk "BEGIN{printf \"%.1f\", $end - $start}")
	PHASE_STATUS=$status
	printf '  %-10s %8ss  exit %d\n' "$name" "$PHASE_SECONDS" "$status" >&2
	return 0
}

echo "measure-preflight: $STAMP" >&2
echo "  repo    $REPO @ ${HEAD_SHA:0:12}" >&2
echo "  mode    $([ "$COLD" -eq 1 ] && echo 'cold (private sccache)' || echo 'current cache conditions')" >&2
echo "  load    $(cut -d' ' -f1-3 /proc/loadavg 2>/dev/null || echo n/a)" >&2
echo "  budget  ${BUDGET_SECONDS}s" >&2

git -C "$REPO" worktree remove --force "$TREE" >/dev/null 2>&1
rmdir "$TREE" 2>/dev/null
phase worktree "$OUT/worktree.log" -- git -C "$REPO" worktree add --detach "$TREE" "$HEAD_SHA"
WORKTREE_SECONDS=$PHASE_SECONDS
WORKTREE_STATUS=$PHASE_STATUS
if [ "$WORKTREE_STATUS" -ne 0 ]; then
	echo "measure-preflight: could not create the worktree; see $OUT/worktree.log" >&2
	exit 1
fi

phase lint "$OUT/lint.log" -- "$TREE/sdlc/scripts/lint"
LINT_SECONDS=$PHASE_SECONDS
LINT_STATUS=$PHASE_STATUS

phase test "$OUT/test.log" -- "$TREE/sdlc/scripts/test"
TEST_SECONDS=$PHASE_SECONDS
TEST_STATUS=$PHASE_STATUS

TOTAL=$(awk "BEGIN{printf \"%.1f\", $WORKTREE_SECONDS + $LINT_SECONDS + $TEST_SECONDS}")
MARGIN=$(awk "BEGIN{printf \"%.1f\", $BUDGET_SECONDS - $TOTAL}")
FITS=$(awk "BEGIN{print ($TOTAL < $BUDGET_SECONDS) ? \"true\" : \"false\"}")

cat >"$OUT/result.json" <<JSON
{
  "measured_at": "$STAMP",
  "repo_head": "$HEAD_SHA",
  "mode": "$([ "$COLD" -eq 1 ] && echo cold || echo current_cache)",
  "forced_under_load": $([ "$FORCE" -eq 1 ] && echo true || echo false),
  "loadavg_at_start": "$(cut -d' ' -f1-3 /proc/loadavg 2>/dev/null || echo unknown)",
  "budget_seconds": $BUDGET_SECONDS,
  "phases": {
    "worktree": { "seconds": $WORKTREE_SECONDS, "exit": $WORKTREE_STATUS },
    "lint":     { "seconds": $LINT_SECONDS,     "exit": $LINT_STATUS },
    "test":     { "seconds": $TEST_SECONDS,     "exit": $TEST_STATUS }
  },
  "total_seconds": $TOTAL,
  "margin_seconds": $MARGIN,
  "fits_budget": $FITS
}
JSON

echo >&2
printf '  total   %8ss  margin %ss  fits %s\n' "$TOTAL" "$MARGIN" "$FITS" >&2
echo "  written $OUT/result.json" >&2

[ "$FITS" = "true" ] && [ "$LINT_STATUS" -eq 0 ] && [ "$TEST_STATUS" -eq 0 ] && exit 0
exit 1
