#!/usr/bin/env bash
# Keep ticket numbers unique across main and the 1.0 line.
#
# The two lines share ticket history in sdlc/tickets/, so a number the
# 1.0 line reuses for a different file silently shadows the ticket
# main already filed under that number. The check enforces on the 1.0
# line only — the branch that must dodge main — and passes silently
# everywhere else, main included. Run from `make lint`.
#
# Written for the macOS-bundled Bash 3.2: no associative arrays.

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

# number<TAB>path lines for the numbered ticket files directly in
# sdlc/tickets/; archived and draft tickets never count. Main's files
# come first, one marker line apart, and awk holds both sides of the
# comparison.
ticket_number_of() {
  local name="$1"
  if [[ $name =~ ^([0-9]+)- ]]; then
    printf '%s' "$((10#${BASH_REMATCH[1]}))"
  fi
}

{
  while IFS= read -r path; do
    number="$(ticket_number_of "${path##*/}")"
    if [ -n "$number" ]; then
      printf '%s\t%s\n' "$number" "$path"
    fi
  done < <(git ls-tree --name-only "$MAIN_REF" -- "$TICKETS_DIR/")
  printf 'HEAD-TICKETS\n'
  for path in "$TICKETS_DIR"/*.md; do
    [ -f "$path" ] || continue
    number="$(ticket_number_of "${path##*/}")"
    if [ -n "$number" ]; then
      printf '%s\t%s\n' "$number" "$path"
    fi
  done
} | awk -v floor="$LINE_TICKET_FLOOR" -v main_ref="$MAIN_REF" '
  $0 == "HEAD-TICKETS" { in_head = 1; next }
  {
    tab = index($0, "\t")
    number = substr($0, 1, tab - 1) + 0
    path = substr($0, tab + 1)
    if (!in_head) {
      main_path[number] = path
      main_file[path] = 1
      next
    }
    if (number in main_path && main_path[number] != path) {
      printf "%s: ticket %d already exists on %s as %s\n", path, number, main_ref, main_path[number]
      failures++
    }
    if (!(path in main_file) && number < floor) {
      printf "%s: ticket %d is below %d on the 1.0 line\n", path, number, floor
      failures++
    }
  }
  END {
    if (failures > 0) {
      printf "ticket-number check: %d number(s) collide with %s or sit below %d\n", failures, main_ref, floor
      exit 1
    }
  }
'
