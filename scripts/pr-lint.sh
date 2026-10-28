#!/usr/bin/env bash
# Checks a PR's body and commit messages against docs/agents/workflow.md (PRs,
# Review) and the sections of .github/pull_request_template.md, and that a PR
# changing what Reference scenes render links its gallery images.
# Usage: pr-lint.sh <body-file> <changed-paths-file> [<commit-message-file>...]
# The changed-paths file lists the PR's paths, one per line.
# Prints one line per miss, naming the rule it breaks; exits non-zero on any.
set -uo pipefail

template="$(dirname "$0")/../.github/pull_request_template.md"
misses=0

miss() {
  misses=$((misses + 1))
  printf '%s\n' "$1"
}

# The body without carriage returns, HTML comments or trailing blanks, so a
# section that holds only its placeholder comment reads as empty. Lines in
# code fences are indented, so pasted output is never a heading or ticket line.
body=$(tr -d '\r' <"$1" | perl -0pe 's/<!--.*?-->//gs; s/[ \t]+$//mg' |
  perl -pe 'if (/^\s*```/) { $fenced = !$fenced; next } $_ = " $_" if $fenced')

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
  if ! grep -Eiq "(^|[^0-9])[0-9]+ $kind([^[:alpha:]]|$)|(^|[^[:alpha:]])$kind:? [0-9]+" <<<"$verification"; then
    uncounted+=("$kind")
  fi
done
if [ ${#uncounted[@]} -gt 0 ]; then
  miss "body: Verification lacks the review counts \"N found, N fixed, N declined, N deferred\" (missing: ${uncounted[*]}) (workflow.md, Review)"
fi
# "Sources checked" followed by something, on its line or below it.
if ! awk '
  /^[*_ -]*Sources checked/ { seen = 1; sub(/^[*_ -]*Sources checked[*_: ]*/, "") }
  seen && /[^[:space:]]/ { listed = 1 }
  END { exit !listed }' <<<"$verification"; then
  miss 'body: Verification lacks a "Sources checked" line listing each reference and its verdict, or None (workflow.md, Review)'
fi

# A PR touching the renderer, the cooker's textures or meshes, or ADR 0019 or
# 0020 links scripts/gallery.sh's before, after and difference images: links
# to an image whose text or file name names its kind.
if [ ! -r "${2:-}" ]; then
  miss 'paths: no changed-paths file to check'
elif grep -Eq '^(src/render/|src/cook/(texture|mesh)(\.rs$|/)|docs/adr/00(19|20)-)' "$2"; then
  images=$(section '## Images')
  unlinked=()
  for kind in before after difference; do
    if ! KIND="$kind" perl -ne '
      while (/(!?)\[([^\]]*)\]\(([^)\s]+)\)/g) {
        my ($embedded, $text, $url) = ($1, $2, $3);
        next unless $embedded || $url =~ /\.(png|jpe?g|gif|webp)(\?\S*)?$/i;
        my $name = $text . " " . ($url =~ s{.*/}{}r);
        $found = 1 if $name =~ /(?<![[:alpha:]])$ENV{KIND}(?![[:alpha:]])/i;
      }
      END { exit !$found }' <<<"$images"; then
      unlinked+=("$kind")
    fi
  done
  if [ ${#unlinked[@]} -gt 0 ]; then
    miss "body: a rendering PR's Images links scripts/gallery.sh's before, after and difference images, each named by its kind in its link text or file name (missing: ${unlinked[*]}) (workflow.md, PRs)"
  fi
fi

for file in "${@:3}"; do
  name=$(basename "$file")
  if [ ! -r "$file" ]; then
    miss "commit $name: no message to check"
    continue
  fi
  message=$(tr -d '\r' <"$file")
  subject=$(head -n 1 <<<"$message")
  length=$(perl -CA -e 'print length $ARGV[0]' "$subject")
  if [ "$length" -gt 72 ]; then
    miss "commit $name: subject is $length characters, over 72 (workflow.md, PRs)"
  fi
  if grep -Eiq '^co-authored-by:.*(claude|anthropic|copilot|openai|chatgpt|gemini|cursor|codex|devin)' <<<"$message"; then
    miss "commit $name: AI co-author trailer; commits name no AI co-author (workflow.md, PRs)"
  fi
  # A line that opens with the phrase, after any emoji, not prose naming it.
  if grep -Eiq '^[^[:alnum:]]*generated with \[?claude' <<<"$message"; then
    miss "commit $name: \"Generated with Claude\" line; commits carry no AI attribution (workflow.md, PRs)"
  fi
done

[ "$misses" -eq 0 ]
