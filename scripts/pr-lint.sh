#!/usr/bin/env bash
# Checks a PR's title, body, size and commit messages against
# docs/agents/workflow.md (Tickets, PRs, Review) and the sections of
# .github/pull_request_template.md, and that a PR touching rendering paths
# links before, after and difference images or counts its identical ones.
# Usage: pr-lint.sh <number> <title> <body-file> <changed-lines-file>
#        [<commit-message-file>...]
# The changed-lines file holds a line per path of the PR: its changed lines,
# a tab, the path.
# Prints one line per miss, naming the rule it breaks; exits non-zero on any.
set -uo pipefail

number="$1"
title="$2"
body_file="$3"
changed_lines_file="${4:-}"
template="$(dirname "$0")/../.github/pull_request_template.md"
misses=0

miss() {
  misses=$((misses + 1))
  printf '%s\n' "$1"
}

# The body without carriage returns, HTML comments or trailing blanks, so a
# section that holds only its placeholder comment reads as empty. Lines in
# code fences are indented, so pasted output is never a heading or ticket line.
body=$(tr -d '\r' <"$body_file" | perl -0pe 's/<!--.*?-->//gs; s/[ \t]+$//mg' |
  perl -pe 'if (/^\s*```/) { $fenced = !$fenced; next } $_ = " $_" if $fenced')

# The lines under a "## " heading, up to the next heading.
section() {
  printf '%s\n' "$body" | awk -v heading="$1" '/^## / { inside = ($0 == heading); next } inside'
}

if ! grep -Eq '^(Closes #[0-9]+|Part [0-9]+ of #[0-9]+|Ticket: n/a \(.+\))$' <<<"$body"; then
  miss 'body: no ticket line; one ticket per PR needs "Closes #N" on a line of its own (workflow.md, PRs), or there "Part k of #N" for a partial PR or "Ticket: n/a (reason)" for none (the PR template)'
fi

while IFS= read -r heading; do
  if ! grep -Fxq "$heading" <<<"$body"; then
    miss "body: \"$heading\" is missing; the description follows the PR template (workflow.md, PRs)"
  # The section is captured before grep reads it: piped, grep -q's early exit
  # leaves awk's later writes to a SIGPIPE, and pipefail's failure reads as an
  # empty section.
  elif ! grep -q '[^[:space:]]' <<<"$(section "$heading")"; then
    miss "body: \"$heading\" holds only its placeholder; fill it, or write n/a or None as it asks (workflow.md, PRs)"
  fi
done < <(tr -d '\r' <"$template" | grep '^## ')

verification=$(section '## Verification')
uncounted=()
for kind in found fixed declined deferred; do
  if ! grep -Eiq "(^|[^0-9])[0-9]+ $kind([^[:alpha:]]|$)|(^|[^[:alpha:]])$kind:? [0-9]+" <<<"$verification"; then
    uncounted+=("$kind")
  fi
done
if [ ${#uncounted[@]} -gt 0 ]; then
  miss "body: Verification lacks the review counts \"N found, N fixed, N declined, N deferred\" (missing: ${uncounted[*]}) (workflow.md, Review)"
fi
# Pasted output is what Verification's code fences hold, all fences together.
pasted_line_limit=30
pasted_lines=$(awk '/^[[:space:]]*```/ { fenced = !fenced; next } fenced { lines++ } END { print lines + 0 }' <<<"$verification")
if [ "$pasted_lines" -gt "$pasted_line_limit" ]; then
  miss "body: Verification's pasted output is $pasted_lines lines, over $pasted_line_limit (workflow.md, PRs)"
fi
# "Sources checked" followed by something, on its line or below it.
if ! awk '
  /^[*_ -]*Sources checked/ { seen = 1; sub(/^[*_ -]*Sources checked[*_: ]*/, "") }
  seen && /[^[:space:]]/ { listed = 1 }
  END { exit !listed }' <<<"$verification"; then
  miss 'body: Verification lacks a "Sources checked" line listing each reference and its verdict, or None (workflow.md, Review)'
fi

# Whether the text on standard input links an image whose link text or file
# name names the kind.
links_image() {
  KIND="$1" perl -ne '
    while (/(!?)\[([^\]]*)\]\(([^)\s]+)(?:\s+"[^"]*")?\)/g) {
      my ($embedded, $text, $url) = ($1, $2, $3);
      next unless $embedded || $url =~ /\.(png|jpe?g|gif|webp)(\?\S*)?$/i;
      my $name = $text . " " . ($url =~ s{.*/}{}r);
      $found = 1 if $name =~ /(?<![[:alpha:]])$ENV{KIND}(?![[:alpha:]])/i;
    }
    END { exit !$found }'
}

# The count line stands for the images only where its two counts are equal.
if [ ! -r "$changed_lines_file" ]; then
  miss 'paths: no changed-lines file to check'
else
  paths=$(cut -f 2- "$changed_lines_file")
  if grep -Eq '^(src/render/|src/cook/(texture|mesh)(\.rs$|/)|docs/adr/00(19|20)-)' <<<"$paths"; then
    images=$(section '## Images')
    unlinked=()
    for kind in before after difference; do
      links_image "$kind" <<<"$images" || unlinked+=("$kind")
    done
    if [ ${#unlinked[@]} -gt 0 ]; then
      linking="body: a rendering PR's Images links scripts/gallery.sh's before, after and difference images, each named by its kind in its link text or file name (missing: ${unlinked[*]})"
      if grep -Eq '^tests/goldens/|\.wgsl$' <<<"$paths"; then
        miss "$linking; a PR that moves a Golden or changes a .wgsl file links all three (workflow.md, PRs)"
      elif ! perl -ne '$counted = 1 if /(?<![0-9])([1-9][0-9]*) of \1(?![0-9]) gallery images identical/; END { exit !$counted }' <<<"$images"; then
        miss "$linking, or gives the gallery's count on one line when no image differs, as \"22 of 22 gallery images identical to the base's\" (workflow.md, PRs)"
      fi
    fi
  fi
  code_line_limit=800
  code_lines=$(awk -F '\t' -v uncounted='^(docs/|[^/]+[.]md$|tests/goldens/|Cargo[.]lock$|bench/(baselines|cycles)/|vendor/)' \
    '$2 !~ uncounted { lines += $1 } END { print lines + 0 }' "$changed_lines_file")
  if [ "$code_lines" -gt "$code_line_limit" ]; then
    miss "size: $code_lines changed lines of code, over $code_line_limit; a ticket that cannot fit splits along a seam (workflow.md, Tickets)"
  fi
fi

ai_tools='claude|anthropic|copilot|openai|chatgpt|gemini|cursor|codex|devin'
check_message() {
  local name="$1" message="$2"
  local subject length
  subject=$(head -n 1 <<<"$message")
  length=$(perl -CA -e 'print length $ARGV[0]' -- "$subject")
  if [ "$length" -gt 72 ]; then
    miss "$name: subject is $length characters, over 72 (workflow.md, PRs)"
  fi
  if grep -Eiq "^co-authored-by:.*($ai_tools)" <<<"$message"; then
    miss "$name: AI co-author trailer; commits name no AI co-author (workflow.md, PRs)"
  fi
  # A line that opens with the phrase, after any emoji, not prose naming it.
  if grep -Eiq "^[^[:alnum:]]*generated with \[?($ai_tools)" <<<"$message"; then
    miss "$name: \"Generated with\" line naming an AI tool; commits carry no AI attribution (workflow.md, PRs)"
  fi
}

# The squash merge commits the title, with the number appended, over the body.
check_message 'squash commit (title, number, body)' "$title (#$number)"$'\n\n'"$(tr -d '\r' <"$body_file")"

for file in "${@:5}"; do
  name=$(basename "$file")
  if [ ! -r "$file" ]; then
    miss "commit $name: no message to check"
    continue
  fi
  check_message "commit $name" "$(tr -d '\r' <"$file")"
done

[ "$misses" -eq 0 ]
