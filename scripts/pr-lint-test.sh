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

# The PR's changed paths, one per line, for the next cases.
changed_paths='docs/agents/workflow.md'

# Runs the lint on a body and commit messages, each given as a string.
run_lint() {
  local body="$1"
  shift
  printf '%s' "$body" >"$work/body.md"
  printf '%s\n' "$changed_paths" >"$work/paths"
  local commits=()
  local index=0
  local message
  for message in "$@"; do
    index=$((index + 1))
    printf '%s' "$message" >"$work/commit-$index"
    commits+=("$work/commit-$index")
  done
  output=$("$lint" "$work/body.md" "$work/paths" ${commits[@]+"${commits[@]}"} 2>&1)
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

# The passing body with a Perl substitution applied to it whole (`m` flag for
# line anchors; without `g`, only the first match changes).
edited_body() {
  passing_body | perl -0pe "$1"
}

body=$(passing_body)
template_body=$(cat "$template")
subject_72=$(printf 'x%.0s' {1..72})
human_commit=$'Add the check\n\nWhy it matters.\n\nCo-Authored-By: A Person <person@example.com>'
fence='```'

expect_pass "a complete body and clean commits" "$body" "$human_commit" "$subject_72"
expect_pass "a partial PR" "$(edited_body 's/^Closes #316$/Part 2 of #310/m')"
expect_pass "a process PR" "$(edited_body 's/^Closes #316$/Ticket: n\/a (process change)/m')"
expect_pass "CRLF line endings" "$(edited_body 's/\n/\r\n/g')"
expect_pass "a filled section that keeps its placeholder comment" \
  "$(edited_body 's/^n\/a$/<!-- Otherwise `n\/a`. -->\nn\/a/m')"
expect_pass "counts written kind first" \
  "$(edited_body 's/4 found, 2 fixed, 1 declined, 1 deferred/found 4, fixed 2, declined 1, deferred: 1/')"
expect_pass "a heading in pasted output" \
  "$(edited_body "s/^(Review: )/$fence\\n## Metrics\\n$fence\\n\\n\$1/m")"
expect_pass "sources listed below their line" \
  "$(edited_body 's/^Sources checked: None\.$/Sources checked:\n\n- ADR 0015: confirmed./m')"
expect_pass "a commit body naming the attribution rule" "$body" \
  $'Add the check\n\nCommits carry no co-author or Generated with Claude line.'
expect_pass "a 72-character subject with non-ASCII letters" "$body" \
  "$(printf 'é%.0s' {1..72})"

expect_fail "the unfilled template: ticket line" 'ticket line' "$template_body"
expect_fail "the unfilled template: What" '"## What" holds only its placeholder' "$template_body"
expect_fail "the unfilled template: Decide" '"## Decide" holds only its placeholder' "$template_body"
expect_fail "the unfilled template: counts" 'review counts' "$template_body"
expect_fail "the unfilled template: sources" 'Sources checked' "$template_body"
expect_fail "a ticket line without a number" 'ticket line' "$(edited_body 's/^Closes #316$/Closes #/m')"
expect_fail "a ticket line inside a sentence" 'ticket line' \
  "$(edited_body 's/^Closes #316$/This Closes #316 eventually./m')"
expect_fail "a ticket line in pasted output" 'ticket line' \
  "$(edited_body "s/^Closes #316$/$fence\\nCloses #316\\n$fence/m")"
expect_fail "n/a without a reason" 'ticket line' "$(edited_body 's/^Closes #316$/Ticket: n\/a/m')"
expect_fail "a missing section" '"## Images" is missing' "$(edited_body 's/^## Images$/## Pictures/m')"
expect_fail "an empty section" '"## Metrics" holds only its placeholder' \
  "$(edited_body 's/^n\/a$/<!-- Otherwise `n\/a`. -->/m')"
expect_fail "counts missing one kind" 'review counts' \
  "$(edited_body 's/, 1 deferred/. Deferred: none/')"
expect_fail "counts outside Verification" 'review counts' \
  "$(edited_body 's/^.* found, .*\n//m')"$'\n\n4 found, 2 fixed, 1 declined, 1 deferred.'
expect_fail "no Sources checked line" 'Sources checked' "$(edited_body 's/^Sources checked.*\n//m')"
expect_fail "an empty Sources checked line" 'Sources checked' \
  "$(edited_body 's/^Sources checked: None\.$/Sources checked:/m')"
expect_fail "Sources checked outside Verification" 'Sources checked' \
  "$(edited_body 's/^Sources checked.*\n//m')"$'\nSources checked: None.'
expect_fail "an AI co-author" 'AI co-author' "$body" \
  $'Add the check\n\nCo-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>'
expect_fail "an AI co-author in lower case" 'AI co-author' "$body" \
  $'Add the check\n\nco-authored-by: GitHub Copilot <copilot@github.com>'
expect_fail "a Generated with Claude line" 'Generated with Claude' "$body" \
  $'Add the check\n\n🤖 Generated with [Claude Code](https://claude.com/claude-code)'
expect_fail "a 73-character subject" 'subject is 73 characters' "$body" "x$subject_72"

raw=https://raw.githubusercontent.com/gariasf/praxis/010f64c0e7dd10ba7790c98060ffdad8a6ccf9d0/pr-369
gallery_table="| Scene | Before | After | Difference |
|---|---|---|---|
| town | ![]($raw/town-frame-59-before.png) | ![]($raw/town-frame-59-after.png) | ![]($raw/town-frame-59-difference.png) |"
gallery_links="- \`shadows\`: [before]($raw/shadows.png), [after]($raw/shadows-2.png), [difference]($raw/shadows-3.png)."
# The passing body with its Images section holding the given text.
images_body() {
  passing_body | IMAGES="$1" perl -0pe 's/^## Images\n\nn\/a$/## Images\n\n$ENV{IMAGES}/m'
}

changed_paths=$'docs/guides/index.html\nscripts/gallery.sh'
expect_pass "a docs PR without images" "$body"
changed_paths=$'src/render/main_pass.rs\nREADME.md'
expect_pass "a rendering PR with images in a table" "$(images_body "$gallery_table")"
expect_pass "a rendering PR with images named by their links" "$(images_body "$gallery_links")"
expect_fail "a rendering PR without images" 'missing: before after difference' "$body"
expect_fail "a rendering PR without a difference image" 'missing: difference' \
  "$(images_body "${gallery_table/-difference.png/.png}")"
expect_fail "images outside the Images section" 'missing: before after difference' \
  "$body"$'\n\n'"$gallery_table"
expect_fail "kinds named in prose, not links" 'missing: before after difference' \
  "$(images_body 'Before and after match; the difference image is empty.')"
for changed_paths in src/render/shaders/lit.wgsl src/cook/texture.rs src/cook/mesh.rs \
  docs/adr/0019-clustered-forward-main-pass-with-4x-msaa.md \
  docs/adr/0020-ray-traced-probe-gi-and-hard-ray-traced-shadows.md; do
  expect_fail "a PR touching $changed_paths without images" 'missing: before' "$body"
done
changed_paths='src/cook/fetch.rs'
expect_pass "a cooker change outside textures and meshes" "$body"
changed_paths='docs/agents/workflow.md'

cases=$((cases + 1))
printf '%s' "$body" >"$work/body.md"
if "$lint" "$work/body.md" "$work/paths" "$work/absent" >/dev/null 2>&1; then
  failures=$((failures + 1))
  printf 'FAIL  a missing commit file: expected a failure\n'
fi

cases=$((cases + 1))
if "$lint" "$work/body.md" "$work/absent" >/dev/null 2>&1; then
  failures=$((failures + 1))
  printf 'FAIL  a missing changed-paths file: expected a failure\n'
fi

if [ "$failures" -eq 0 ]; then
  printf 'ok    pr-lint  %s cases\n' "$cases"
else
  printf 'FAIL  pr-lint  %s of %s cases\n' "$failures" "$cases"
  exit 1
fi
