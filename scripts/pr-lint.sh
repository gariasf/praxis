#!/usr/bin/env bash
# Checks a PR's body and commit messages against docs/agents/workflow.md (PRs,
# Review) and the sections of .github/pull_request_template.md.
# Usage: pr-lint.sh <body-file> [<commit-message-file>...]
# Prints one line per miss, naming the rule it breaks; exits non-zero on any.
set -uo pipefail

template="$(dirname "$0")/../.github/pull_request_template.md"
misses=0

miss() {
  misses=$((misses + 1))
  printf '%s\n' "$1"
}

# The body without carriage returns, HTML comments or trailing blanks, so a
# section that holds only its placeholder comment reads as empty.
body=$(tr -d '\r' <"$1" | perl -0pe 's/<!--.*?-->//gs; s/[ \t]+$//mg')

# The lines under a "## " heading, up to the next heading.
section() {
  printf '%s\n' "$body" | awk -v heading="$1" '/^## / { inside = ($0 == heading); next } inside'
}

if ! grep -Eq '^(Closes #[0-9]+|Part [0-9]+ of #[0-9]+|Ticket: n/a \(.+\))$' <<<"$body"; then
  miss 'body: no ticket line; one ticket per PR needs "Closes #N", "Part k of #N" or "Ticket: n/a (reason)" on a line of its own (workflow.md, PRs)'
fi

while IFS= read -r heading; do
  if ! grep -Fxq "$heading" <<<"$body"; then
    miss "body: \"$heading\" is missing; the description follows the PR template (workflow.md, PRs)"
  elif ! section "$heading" | grep -q '[^[:space:]]'; then
    miss "body: \"$heading\" holds only its placeholder; fill it, or write n/a or None as it asks (workflow.md, PRs)"
  fi
done < <(tr -d '\r' <"$template" | grep '^## ')

verification=$(section '## Verification')
uncounted=()
for kind in found fixed declined deferred; do
  if ! grep -Eiq "(^|[^0-9])[0-9]+ $kind\b|\b$kind:? [0-9]+" <<<"$verification"; then
    uncounted+=("$kind")
  fi
done
if [ ${#uncounted[@]} -gt 0 ]; then
  miss "body: Verification lacks the review counts \"N found, N fixed, N declined, N deferred\" (missing: ${uncounted[*]}) (workflow.md, Review)"
fi
if ! grep -Eq '^[*_ -]*Sources checked' <<<"$verification"; then
  miss 'body: Verification lacks a "Sources checked" line listing each reference and its verdict, or None (workflow.md, Review)'
fi

for file in "${@:2}"; do
  name=$(basename "$file")
  message=$(tr -d '\r' <"$file")
  subject=$(head -n 1 <<<"$message")
  length=$(perl -CA -e 'print length $ARGV[0]' "$subject")
  if [ "$length" -gt 72 ]; then
    miss "commit $name: subject is $length characters, over 72 (workflow.md, PRs)"
  fi
  if grep -Eiq '^co-authored-by:.*(claude|anthropic|copilot|openai|chatgpt|gemini|cursor|codex|devin)' <<<"$message"; then
    miss "commit $name: AI co-author trailer; commits name no AI co-author (workflow.md, PRs)"
  fi
  if grep -Eiq 'generated with \[?claude' <<<"$message"; then
    miss "commit $name: \"Generated with Claude\" line; commits carry no AI attribution (workflow.md, PRs)"
  fi
done

[ "$misses" -eq 0 ]
