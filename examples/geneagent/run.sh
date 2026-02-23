#!/usr/bin/env bash
set -euo pipefail

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$DIR"

PI_CMD="${PI_CMD:-pi}"
PI_MODEL="${PI_MODEL:-glm-4.7}"
PROMPT_FILE="prompt.md"

if ! command -v "$PI_CMD" >/dev/null 2>&1; then
  echo "Error: $PI_CMD not found in PATH" >&2
  exit 1
fi

START_NS=$(date +%s%N)
RESULT="$($PI_CMD --model "$PI_MODEL" -p "$(cat "$PROMPT_FILE")" 2>stderr.log)"
END_NS=$(date +%s%N)

printf '%s\n' "$RESULT" > output.md

ELAPSED_MS=$(( (END_NS - START_NS) / 1000000 ))
TOOL_CALLS=$(grep -ci 'biomcp' stderr.log 2>/dev/null || true)
OUTPUT_WORDS=$(wc -w < output.md)

cat > metrics.json <<JSON
{
  "elapsed_ms": $ELAPSED_MS,
  "tool_calls": ${TOOL_CALLS:-0},
  "output_words": $OUTPUT_WORDS
}
JSON

echo "Time: ${ELAPSED_MS}ms | Tools: ${TOOL_CALLS:-0} | Words: $OUTPUT_WORDS"
"$DIR/score.sh" output.md
