#!/usr/bin/env bash
# Keep ticket numbers unique across main and the 1.0 line.
#
# The two lines share ticket history in sdlc/tickets/, so a number the
# 1.0 line reuses for a different file silently shadows the ticket
# main already filed under that number. The check enforces on the 1.0
# line only — the branch that must dodge main — and passes silently
# everywhere else, main included. Run from `make lint`.

set -euo pipefail

MAIN_REF="origin/main"
LINE_REF="origin/biodata/biomcp-1.0"
TICKETS_DIR="sdlc/tickets"
# The 1.0 line files its own tickets from this number up; the numbers
# below it on the line are main's, carried by shared history.
LINE_TICKET_FLOOR=2001

cd "$(git rev-parse --show-toplevel)"

require_ref() {
  if ! git rev-parse --verify --quiet "$1" >/dev/null; then
    printf 'ticket-number check: %s is missing; fetch origin before linting\n' "$1" >&2
    exit 1
  fi
}

require_ref "$LINE_REF"

# Not the 1.0 line (main and branches off main): pass silently.
if ! git merge-base --is-ancestor "$LINE_REF" HEAD; then
  exit 0
fi

require_ref "$MAIN_REF"

# number<TAB>path for the numbered ticket files directly in
# sdlc/tickets/; archived and draft tickets never count.
head_ticket_lines() {
  local path name
  for path in "$TICKETS_DIR"/*.md; do
    [ -f "$path" ] || continue
    name="${path##*/}"
    if [[ $name =~ ^([0-9]+)- ]]; then
      printf '%d\t%s\n' "$((10#${BASH_REMATCH[1]}))" "$path"
    fi
  done
}

main_ticket_lines() {
  local path name
  while IFS= read -r path; do
    name="${path##*/}"
    if [[ $name =~ ^([0-9]+)- ]]; then
      printf '%d\t%s\n' "$((10#${BASH_REMATCH[1]}))" "$path"
    fi
  done < <(git ls-tree --name-only "$MAIN_REF" -- "$TICKETS_DIR/")
}

declare -A main_path_for_number=()
declare -A main_paths=()
while IFS=$'\t' read -r number path; do
  main_path_for_number["$number"]="$path"
  main_paths["$path"]=1
done < <(main_ticket_lines)

failures=0
while IFS=$'\t' read -r number path; do
  main_path="${main_path_for_number[$number]:-}"
  if [[ -n "$main_path" && "$main_path" != "$path" ]]; then
    printf '%s: ticket %s already exists on %s as %s\n' \
      "$path" "$number" "$MAIN_REF" "$main_path"
    failures=$((failures + 1))
  fi
  if [[ -z "${main_paths[$path]:-}" && "$number" -lt "$LINE_TICKET_FLOOR" ]]; then
    printf '%s: ticket %s is below %d on the 1.0 line\n' \
      "$path" "$number" "$LINE_TICKET_FLOOR"
    failures=$((failures + 1))
  fi
done < <(head_ticket_lines)

if [ "$failures" -gt 0 ]; then
  printf 'ticket-number check: %d number(s) collide with %s or sit below %d\n' \
    "$failures" "$MAIN_REF" "$LINE_TICKET_FLOOR"
  exit 1
fi
