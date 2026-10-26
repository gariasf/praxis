#!/usr/bin/env bash
# Tests for scripts/pr-lint.sh on sample PR bodies and commit messages.
# Prints one line per failing case and a summary; exits non-zero on failure.
set -uo pipefail

lint="$(dirname "$0")/pr-lint.sh"
template="$(dirname "$0")/../.github/pull_request_template.md"
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
failures=0
cases=0

passing_body() {
  cat <<'EOF'
Closes #316

## What

A CI job checks PR bodies and commits.

## Verification

`scripts/pr-lint-test.sh`: 20 passed.

Review: 3 axes. 4 found, 2 fixed, 1 declined, 1 deferred.

Sources checked: None.

## Metrics

n/a

## Images

n/a

## Decide

None.
EOF
}

# Runs the lint on a body and commit messages, each given as a string.
run_lint() {
  local body="$1"
  shift
  printf '%s' "$body" >"$work/body.md"
  local commits=()
  local index=0
  local message
  for message in "$@"; do
    index=$((index + 1))
    printf '%s' "$message" >"$work/commit-$index"
    commits+=("$work/commit-$index")
  done
  output=$("$lint" "$work/body.md" "${commits[@]}" 2>&1)
  status=$?
}

expect_pass() {
  local name="$1"
  shift
  cases=$((cases + 1))
  run_lint "$@"
  if [ "$status" -ne 0 ] || [ -n "$output" ]; then
    failures=$((failures + 1))
    printf 'FAIL  %s: expected a pass, got status %s:\n%s\n' "$name" "$status" "$output"
  fi
}

# expect_fail <name> <expected output fragment> <body> [<commit>...]
expect_fail() {
  local name="$1"
  local fragment="$2"
  shift 2
  cases=$((cases + 1))
  run_lint "$@"
  if [ "$status" -eq 0 ] || [[ "$output" != *"$fragment"* ]]; then
    failures=$((failures + 1))
    printf 'FAIL  %s: expected a failure naming "%s", got status %s:\n%s\n' \
      "$name" "$fragment" "$status" "$output"
  fi
}

# The passing body with a sed script applied.
edited_body() {
  passing_body | sed "$1"
}

body=$(passing_body)
template_body=$(cat "$template")
subject_72=$(printf 'x%.0s' {1..72})
human_commit=$'Add the check\n\nWhy it matters.\n\nCo-Authored-By: A Person <person@example.com>'

expect_pass "a complete body and clean commits" "$body" "$human_commit" "$subject_72"
expect_pass "a partial PR" "$(edited_body 's/^Closes #316$/Part 2 of #310/')"
expect_pass "a process PR" "$(edited_body 's/^Closes #316$/Ticket: n\/a (process change)/')"
expect_pass "CRLF line endings" "$(passing_body | sed 's/$/\r/')"
expect_pass "a filled section that keeps its placeholder comment" \
  "$(edited_body '0,/^n\/a$/s//<!-- Otherwise `n\/a`. -->\nn\/a/')"
expect_pass "counts written kind first" \
  "$(edited_body 's/4 found, 2 fixed, 1 declined, 1 deferred/found 4, fixed 2, declined 1, deferred: 1/')"
expect_pass "a 72-character subject with non-ASCII letters" "$body" \
  "$(printf 'é%.0s' {1..72})"

expect_fail "the unfilled template" 'ticket line' "$template_body"
expect_fail "the unfilled template" '"## What" holds only its placeholder' "$template_body"
expect_fail "the unfilled template" '"## Decide" holds only its placeholder' "$template_body"
expect_fail "the unfilled template" 'review counts' "$template_body"
expect_fail "the unfilled template" 'Sources checked' "$template_body"
expect_fail "a ticket line without a number" 'ticket line' "$(edited_body 's/^Closes #316$/Closes #/')"
expect_fail "a ticket line inside a sentence" 'ticket line' \
  "$(edited_body 's/^Closes #316$/This Closes #316 eventually./')"
expect_fail "n/a without a reason" 'ticket line' "$(edited_body 's/^Closes #316$/Ticket: n\/a/')"
expect_fail "a missing section" '"## Images" is missing' "$(edited_body 's/^## Images$/## Pictures/')"
expect_fail "an empty section" '"## Metrics" holds only its placeholder' \
  "$(edited_body '0,/^n\/a$/s//<!-- Otherwise `n\/a`. -->/')"
expect_fail "counts missing one kind" 'review counts' \
  "$(edited_body 's/, 1 deferred/. Deferred: none/')"
expect_fail "counts outside Verification" 'review counts' \
  "$(edited_body '/ found, /d')"$'\n\n4 found, 2 fixed, 1 declined, 1 deferred.'
expect_fail "no Sources checked line" 'Sources checked' "$(edited_body '/^Sources checked/d')"
expect_fail "Sources checked outside Verification" 'Sources checked' \
  "$(edited_body '/^Sources checked/d')"$'\nSources checked: None.'
expect_fail "an AI co-author" 'AI co-author' "$body" \
  $'Add the check\n\nCo-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>'
expect_fail "an AI co-author in lower case" 'AI co-author' "$body" \
  $'Add the check\n\nco-authored-by: GitHub Copilot <copilot@github.com>'
expect_fail "a Generated with Claude line" 'Generated with Claude' "$body" \
  $'Add the check\n\n🤖 Generated with [Claude Code](https://claude.com/claude-code)'
expect_fail "a 73-character subject" 'subject is 73 characters' "$body" "x$subject_72"
if [ "$failures" -eq 0 ]; then
  printf 'ok    pr-lint  %s cases\n' "$cases"
else
  printf 'FAIL  pr-lint  %s of %s cases\n' "$failures" "$cases"
  exit 1
fi
